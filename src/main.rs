#![warn(nonstandard_style)]
#![warn(deprecated_in_future)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use crate::gui::player;

use eframe::NativeOptions;
use model::classes::ClassCache;

mod google;
mod gui;
mod model;

fn main() {
    env_logger::init();

    let native_options = NativeOptions::default();

    eframe::run_native(
        "Greed Console",
        native_options,
        Box::new(|cc| {
            let class_cache = if let Some(cache) =
                eframe::get_value::<ClassCache>(cc.storage.unwrap(), "class_cache")
            {
                cache
            } else {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                let (races, classes) = rt.block_on(google::get_races_and_classes());
                ClassCache::new(races, classes)
            };
            Box::new(player::GuiGreedApp::new(cc, class_cache))
        }),
    );
}
