#![warn(nonstandard_style)]
#![warn(deprecated_in_future)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use crate::gui::player;

use eframe::NativeOptions;

mod google;
mod gui;
mod model;

fn main() {
    env_logger::init();

    let native_options = NativeOptions::default();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let (races, classes) = rt.block_on(google::get_races_and_classes());

    eframe::run_native(
        "Greed Console",
        native_options,
        Box::new(|cc| Box::new(player::GuiGreedApp::new(cc, races, classes))),
    );
}
