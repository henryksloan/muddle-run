use crate::net::PlayerConnections;
use bevy::{
    ecs::{Res, ResMut},
    log,
};
use mr_shared_lib::{
    framebuffer::FrameNumber,
    messages::{PlayerInput, PlayerNetId},
    net::ConnectionState,
    player::{PlayerDirectionUpdate, PlayerUpdates},
    GameTime, SimulationTime, SIMULATIONS_PER_SECOND,
};
use std::collections::HashMap;

pub const SERVER_UPDATES_LIMIT: u16 = 64;
pub const MAX_LAG_COMPENSATION_MSEC: u16 = 200;

pub struct DeferredUpdates<T> {
    updates: HashMap<PlayerNetId, Vec<T>>,
}

impl<T> Default for DeferredUpdates<T> {
    fn default() -> Self {
        Self {
            updates: HashMap::new(),
        }
    }
}

impl<T> DeferredUpdates<T> {
    pub fn push(&mut self, player_net_id: PlayerNetId, update: T) {
        let player_updates = self.updates.entry(player_net_id).or_default();
        player_updates.push(update);
    }

    pub fn drain(&mut self) -> HashMap<PlayerNetId, Vec<T>> {
        std::mem::take(&mut self.updates)
    }
}

pub fn process_player_input_updates(
    time: Res<GameTime>,
    player_connections: Res<PlayerConnections>,
    connection_states: Res<HashMap<u32, ConnectionState>>,
    mut simulation_time: ResMut<SimulationTime>,
    mut updates: ResMut<PlayerUpdates>,
    mut deferred_updates: ResMut<DeferredUpdates<PlayerInput>>,
) {
    let lag_compensated_frames =
        (MAX_LAG_COMPENSATION_MSEC as f32 / (1000.0 / SIMULATIONS_PER_SECOND as f32)) as u16;
    let min_frame_number = time.frame_number - FrameNumber::new(lag_compensated_frames);

    let deferred_updates = deferred_updates.drain();
    for (player_net_id, mut player_updates) in deferred_updates {
        let player_connection = player_connections.get_value(player_net_id).unwrap();
        let player_connection_state = connection_states.get(&player_connection).unwrap();
        let player_frame_number = player_connection_state
            .incoming_acknowledgments()
            .0
            // A player has just connected, and it's got only the initial empty update, so it's fine.
            .unwrap_or(time.frame_number);

        let player_update = player_updates
            .first()
            .expect("Expected at least one update for a player hash map entry");
        let updates = updates.get_direction_mut(
            player_net_id,
            player_update.frame_number,
            SERVER_UPDATES_LIMIT,
        );

        // A client might be able to send several messages with the same unacknowledged updates
        // between runs of this system.
        player_updates.dedup_by_key(|update| update.frame_number);

        let mut updates_iter = player_updates.iter().peekable();
        while let Some(player_update) = updates_iter.next() {
            let next_player_update = updates_iter.peek();
            log::trace!(
                "Player ({}) update for frame {}",
                player_net_id.0,
                player_update.frame_number.value()
            );

            let duplicate_updates_from =
                std::cmp::max(player_update.frame_number, min_frame_number);
            let duplicate_updates_to = next_player_update.map_or(player_frame_number, |update| {
                update.frame_number - FrameNumber::new(1)
            });

            let update_to_insert = Some(PlayerDirectionUpdate {
                direction: player_update.direction,
                is_processed_client_input: None,
            });

            for frame_number in duplicate_updates_from..=duplicate_updates_to {
                let existing_update = updates.get(frame_number);
                // We don't want to allow re-writing updates.
                if existing_update.is_none() && updates.can_insert(frame_number) {
                    simulation_time.server_frame =
                        std::cmp::min(simulation_time.server_frame, frame_number);
                    simulation_time.player_frame = simulation_time.server_frame;
                    updates.insert(
                        frame_number,
                        Some(PlayerDirectionUpdate {
                            direction: player_update.direction,
                            is_processed_client_input: None,
                        }),
                    );
                } else if existing_update != Some(&update_to_insert) {
                    // TODO: is just discarding old updates good enough?
                    log::warn!(
                        "Ignoring player {:?} input for frame {} which differs from the existing one (current: {})",
                        player_net_id,
                        frame_number,
                        time.frame_number
                    );
                }
            }
        }
    }
}
