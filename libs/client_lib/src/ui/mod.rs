use bevy_egui::egui::{self, Ui};
use mr_shared_lib::game::components::{PlayerDirection, Position};

pub mod debug_ui;
pub mod overlay_ui;

pub trait MuddleInspectable {
    fn inspect(&self, ui: &mut Ui);

    fn inspect_mut(&mut self, ui: &mut Ui) {
        self.inspect(ui);
    }
}

impl MuddleInspectable for PlayerDirection {
    fn inspect(&self, ui: &mut Ui) {
        let first_value = self
            .buffer
            .last()
            .and_then(|v| v.as_ref())
            .map(|v| format!("[{: <5.2};{: >5.2}]", v.x, v.y))
            .unwrap_or_else(|| "[None]".to_owned());

        egui::CollapsingHeader::new(format!(
            "Direction {}  -  ([{}; {}] ({}/{})",
            first_value,
            self.buffer.start_frame().value(),
            self.buffer.end_frame().value(),
            self.buffer.limit(),
            self.buffer.len(),
        ))
        .id_source("player direction buffer")
        .default_open(false)
        .show(ui, |ui| {
            egui::ScrollArea::from_max_height(200.0).show(ui, |ui| {
                for (frame_number, value) in self.buffer.iter().rev() {
                    let value = value
                        .map(|v| format!("[{: <5.2};{: >5.2}]", v.x, v.y))
                        .unwrap_or_else(|| "[None]".to_owned());
                    ui.label(format!("{}: {}", frame_number.value(), value));
                }
            });
        });
    }
}

impl MuddleInspectable for Position {
    fn inspect(&self, ui: &mut Ui) {
        let first_value = self
            .buffer
            .last()
            .map(|v| format!("[{: <5.2};{: >5.2}]", v.x, v.y))
            .unwrap_or_else(|| "[None]".to_owned());

        egui::CollapsingHeader::new(format!(
            "Position {}  -  ([{}; {}] ({}/{})",
            first_value,
            self.buffer.start_frame().value(),
            self.buffer.end_frame().value(),
            self.buffer.limit(),
            self.buffer.len(),
        ))
        .id_source("position buffer")
        .default_open(false)
        .show(ui, |ui| {
            egui::ScrollArea::from_max_height(200.0).show(ui, |ui| {
                for (frame_number, value) in self.buffer.iter().rev() {
                    ui.label(format!(
                        "{}: [{: <5.2};{: >5.2}]",
                        frame_number.value(),
                        value.x,
                        value.y
                    ));
                }
            });
        });
    }
}
