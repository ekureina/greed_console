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

use crate::cli::Args;
use crate::gui::main::GuiGreedApp;

use clap::Parser;
use eframe::NativeOptions;
use gui::state::AppState;
use log::{error, info, warn, LevelFilter};
use log4rs::{
    append::rolling_file::{
        policy::compound::{
            roll::delete::DeleteRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Root},
    Config,
};
use model::classes::ClassCache;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use self_update::cargo_crate_version;
use std::path::Path;
use tokio::runtime::Runtime;

mod cli;
mod google;
mod gui;
mod model;
mod util;

fn main() {
    let file_logger = RollingFileAppender::builder()
        .build(
            Path::new(&eframe::storage_dir("Greed Console").unwrap())
                .join("app.log")
                .to_str()
                .unwrap(),
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(10 * 1024 * 1024)),
                Box::new(DeleteRoller::new()),
            )),
        )
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("file_logger", Box::new(file_logger)))
        .build(
            Root::builder()
                .appender("file_logger")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    if update_app() {
        return;
    }

    let args = Args::parse();

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
        Box::new(move |cc| {
            let app_state = if let Some(storage) = cc.storage {
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
            } else {
                AppState::new()
            };
            cc.egui_ctx.style_mut(|style| {
                for (_, font_id) in style.text_styles.iter_mut() {
                    font_id.size *= app_state.get_font_scaling();
                }
            });

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let class_cache = if let Some(cache) =
                eframe::get_value::<ClassCache>(cc.storage.unwrap(), "class_cache")
            {
                let current_rules_update_time = rt.block_on(google::get_update_time());
                match current_rules_update_time {
                    Ok(current_rules_update_time) => {
                        if cache.get_cache_update_time() == Some(current_rules_update_time) {
                            cache
                        } else if !app_state.skip_rules_update_confirmation() {
                            match MessageDialog::new()
                                .set_title("Update Rules?")
                                .set_description("Greed Rules Have been updated")
                                .set_level(MessageLevel::Info)
                                .set_buttons(MessageButtons::YesNo)
                                .show()
                            {
                                MessageDialogResult::Yes => conditionally_get_new_cache(&rt, cache),
                                _ => cache,
                            }
                        } else {
                            conditionally_get_new_cache(&rt, cache)
                        }
                    }
                    Err(err) => {
                        error!("Error fetching current rules update time: {err}");
                        cache
                    }
                }
            } else {
                rt.block_on(google::get_origins_and_classes()).unwrap()
            };
            Box::new(GuiGreedApp::new(class_cache, &args.campaigns, app_state))
        }),
    )
    .unwrap();
}

fn conditionally_get_new_cache(rt: &Runtime, old_cache: ClassCache) -> ClassCache {
    match rt.block_on(google::get_origins_and_classes()) {
        Ok(new_cache) => {
            info!("Got new cache!");
            new_cache
        }
        Err(err) => {
            error!("Error getting new cache, using existing cache: {err}");
            MessageDialog::new()
                .set_title("Error")
                .set_description("Error getting new rules from Google")
                .set_level(MessageLevel::Warning)
                .set_buttons(MessageButtons::Ok)
                .show();
            old_cache
        }
    }
}

fn update_app() -> bool {
    match self_update::backends::github::Update::configure()
        .no_confirm(true)
        .repo_owner("ekureina")
        .repo_name("greed_console")
        .current_version(cargo_crate_version!())
        .bin_name("greed_console")
        .show_output(false)
        .build()
    {
        Ok(update) => match update.get_latest_release() {
            Ok(release) => {
                if self_update::version::bump_is_greater(cargo_crate_version!(), &release.version)
                    .unwrap_or(false)
                {
                    match MessageDialog::new()
                        .set_level(rfd::MessageLevel::Info)
                        .set_title(format!(
                            "New version found: ({}). Update to latest version?",
                            release.version
                        ))
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                    {
                        MessageDialogResult::Yes => match update.update_extended() {
                            Ok(_) => {
                                MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Info)
                                    .set_title("Updated App!")
                                    .set_description("Please restart app to play updated version")
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .show();
                                true
                            }
                            Err(err) => {
                                error!("Unable to update app: {err}");
                                false
                            }
                        },
                        MessageDialogResult::No => false,
                        _ => unreachable!(),
                    }
                } else {
                    false
                }
            }
            Err(err) => {
                error!("Error Finding the latest release: {err}");
                false
            }
        },
        Err(err) => {
            error!("Error Finding the latest release: {err}");
            false
        }
    }
}
