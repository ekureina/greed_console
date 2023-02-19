#![warn(nonstandard_style)]
#![warn(deprecated_in_future)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code)]

use crate::gui::player;

use eframe::NativeOptions;
use model::classes::ClassCache;

mod google;
mod gui;
mod model;

fn main() {
    env_logger::init();

    let img_bytes = include_bytes!(concat!(env!("OUT_DIR"), "greed_console_icon")).to_vec();
    let icon_data = eframe::IconData {
        rgba: img_bytes,
        width: env!("GREED_CONSOLE_ICON_WIDTH").parse().unwrap(),
        height: env!("GREED_CONSOLE_ICON_HEIGHT").parse().unwrap(),
    };

    let native_options = NativeOptions {
        icon_data: Some(icon_data),
        ..Default::default()
    };

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
                let (origins, classes) = rt.block_on(google::get_origins_and_classes());
                ClassCache::new(origins, classes)
            };
            Box::new(player::GuiGreedApp::new(cc, class_cache))
        }),
    );
}
