#![warn(nonstandard_style)]
#![warn(deprecated_in_future)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/*
 * A console and digital character sheet for campaigns under the greed ruleset.
 * Copyright (C) 2023 Claire Moore
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
