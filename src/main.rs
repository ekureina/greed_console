#![warn(nonstandard_style)]
#![warn(deprecated_in_future)]
#![warn(clippy::all, clippy::pedantic)]

use crate::gui::player::PlayerGuiGreedApp;

use eframe::NativeOptions;

mod gui;
mod model;

fn main() {
    env_logger::init();

    let native_options = NativeOptions::default();

    eframe::run_native(
        "Greed Console",
        native_options,
        Box::new(|cc| Box::new(PlayerGuiGreedApp::new(cc))),
    );
}
