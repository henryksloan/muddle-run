use crate::MaterialHandles;
use bevy::{ecs::SystemParam, prelude::*};
use mr_shared_lib::game::{level_objects::*, spawn::Spawner};
use std::marker::PhantomData;

#[derive(SystemParam)]
pub struct SpawnerPbrDeps<'a> {
    meshes: ResMut<'a, Assets<Mesh>>,
    materials: ResMut<'a, Assets<StandardMaterial>>,
    material_handles: Res<'a, MaterialHandles>,
}

#[derive(Default)]
pub struct PlayerSpawner;

impl<'a> Spawner<'a> for PlayerSpawner {
    type Dependencies = SpawnerPbrDeps<'a>;
    type Input = ();

    fn spawn(
        commands: &mut Commands,
        _deps: &mut Self::Dependencies,
        _input: &Self::Input,
    ) -> Entity {
        // commands.spawn(PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
        //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        //     ..Default::default()
        // });
        commands.current_entity().unwrap()
    }
}

#[derive(Default)]
pub struct PlaneSpawner;

impl<'a> Spawner<'a> for PlaneSpawner {
    type Dependencies = SpawnerPbrDeps<'a>;
    type Input = PlaneDesc;

    fn spawn(
        commands: &mut Commands,
        _deps: &mut Self::Dependencies,
        _input: &Self::Input,
    ) -> Entity {
        // commands.spawn(PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::Plane { size: PLANE_SIZE })),
        //     material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        //     ..Default::default()
        // });
        commands.current_entity().unwrap()
    }
}
