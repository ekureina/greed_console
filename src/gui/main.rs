use super::campaign::CampaignGui;
use super::state::AppState;
use super::tabs::CampaignTabViewer;
use crate::google::GetOriginsAndClassesError;
use crate::gui::util::{error_log_and_notify, info_log_and_notify};
use crate::model::classes::{Class, ClassCache};
use crate::model::save::{Save, SaveWithPath};

use eframe::egui;
use eframe::glow::Context;
use eframe::Storage;
use egui::emath::Numeric;
use egui_dock::{DockState, Style};
use egui_notify::Toasts;
use log::info;
use rfd::{FileDialog, MessageDialog, MessageDialogResult};
use self_update::cargo_crate_version;
use tokio::join;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::OsString;
use std::rc::Rc;
use std::time::Duration;

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

static COPYRIGHT_NOTICE: &str = "
A console and digital character sheet for campaigns under the greed ruleset.
Copyright (C) 2023 Claire Moore

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
";

pub struct GuiGreedApp {
    dock_state: DockState<CampaignGui>,
    tab_viewer: CampaignTabViewer,
    app_state: AppState,
    new_campaign_name_entry: String,
    random_campaign_name_entry: String,
    class_cache_rc: Rc<RefCell<ClassCache>>,
    rule_refresh_runtime: Runtime,
    rule_refresh_handle: RefCell<Option<JoinHandle<Result<ClassCache, GetOriginsAndClassesError>>>>,
    toasts: Toasts,
    random_level: f64,
}

impl GuiGreedApp {
    pub fn new(
        cc: &eframe::CreationContext,
        class_cache: ClassCache,
        campaigns: &[OsString],
    ) -> GuiGreedApp {
        info!("Starting up app!");
        let app_state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppState::new()
        };

        let class_cache_rc = Rc::new(RefCell::new(class_cache));

        let rule_refresh_runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name("rules-refresh-worker")
            .enable_all()
            .build()
            .unwrap();

        let mut toasts = Toasts::default();

        let mut campaign_guis: Vec<CampaignGui> = campaigns
            .iter()
            .map(SaveWithPath::from_path)
            .filter_map(|result| {
                result
                    .map_err(|err| error_log_and_notify(&mut toasts, format!("{err}")))
                    .ok()
            })
            .map(|save| CampaignGui::new_refreshable(save, class_cache_rc.clone()))
            .collect();

        campaign_guis
            .iter_mut()
            .for_each(CampaignGui::refresh_campaign);

        let dock_state = DockState::new(campaign_guis);

        let mut app = GuiGreedApp {
            dock_state,
            tab_viewer: CampaignTabViewer::new(),
            app_state,
            new_campaign_name_entry: String::new(),
            random_campaign_name_entry: String::new(),
            class_cache_rc,
            rule_refresh_runtime,
            rule_refresh_handle: RefCell::new(None),
            toasts,
            random_level: 0.0,
        };
        app.update_app(false);
        app
    }

    fn menu_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").resizable(false).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let about_response = ui.button("About");
                let about_popup_id = ui.make_persistent_id("about_popup_id");
                if about_response.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(about_popup_id));
                }
                egui::popup_below_widget(ui, about_popup_id, &about_response, |ui| {
                    ui.set_min_width(450.0);
                    ui.label(COPYRIGHT_NOTICE);
                });

                ui.menu_button("Campaign", |ui| {
                    self.campaign_menu(ui);
                });

                if self.dock_state.find_active_focused().is_some() {
                    if ui.button("Next Battle").clicked() {
                        self.perform_on_all_guis_mut(&CampaignGui::next_battle);
                    }

                    if ui.button("Next Turn").clicked() {
                        self.perform_on_all_guis_mut(&CampaignGui::next_turn);
                    }
                }

                self.refresh_rules(ui, frame);

                if ui.button("Update App").clicked() && self.update_app(true) {
                    frame.close();
                }

                ui.hyperlink_to("Greed Rulset", "https://docs.google.com/document/d/1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4/edit?usp=sharing");
            });
        });
    }

    fn campaign_menu(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(200.0);
        ui.menu_button("New", |ui| {
            ui.text_edit_singleline(&mut self.new_campaign_name_entry);
            if !self.new_campaign_name_entry.is_empty() && ui.button("Create").clicked() {
                let mut campaign_gui = CampaignGui::new_refreshable(
                    SaveWithPath::new(Save::new(self.new_campaign_name_entry.clone())),
                    self.class_cache_rc.clone(),
                );
                campaign_gui.refresh_campaign();
                self.new_campaign_name_entry.clear();
                self.dock_state.push_to_first_leaf(campaign_gui);
            }
        });

        self.random_campaign_submenu(ui);

        if ui.button("Open").clicked() {
            self.open_campaign();
        }

        if !self.app_state.is_campaign_history_empty() {
            ui.menu_button("Recent Campaigns", |ui| {
                let mut invalid_paths = vec![];
                let mut distintness_decider = HashSet::new();
                for (pos, path) in self
                    .app_state
                    .get_campaign_path_history()
                    .clone()
                    .into_iter()
                    .enumerate()
                {
                    if Save::from_file(path.clone()).is_err()
                        || !distintness_decider.insert(path.clone())
                    {
                        invalid_paths.push(pos);
                    } else if ui.button(path.clone().to_string_lossy()).clicked() {
                        self.app_state.use_path_more_recently(pos);
                        self.open_new_save(&path);
                    }
                }

                // Offset because items will be removed
                for (offset, pos) in invalid_paths.into_iter().enumerate() {
                    self.app_state.remove_entry(pos - offset);
                }
            });
        }

        if let Some((_, campaign_gui)) = self.dock_state.find_active_focused() {
            if ui.button("Save").clicked() {
                info_log_and_notify(&mut self.toasts, "Attempting to save campaign!");
                let open_file_picker = if campaign_gui.get_path().is_some() {
                    campaign_gui
                        .save()
                        .unwrap()
                        .map_err(|err| {
                            error_log_and_notify(
                                &mut self.toasts,
                                format!("Failed to save campaign: {err}",),
                            );
                        })
                        .is_err()
                } else {
                    true
                };

                if open_file_picker {
                    self.save_as();
                }
            }
        }

        if self.dock_state.find_active_focused().is_some() && ui.button("Save As...").clicked() {
            self.save_as();
        }
    }

    fn random_campaign_submenu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("New Random Campaign", |ui| {
            ui.text_edit_singleline(&mut self.random_campaign_name_entry);
            ui.horizontal(|ui| {
                ui.label("Level:");
                ui.add(egui::Slider::new(
                    &mut self.random_level,
                    0.0..=self.class_cache_rc.borrow().get_classes().len().to_f64(),
                ));
            });
            if !self.random_campaign_name_entry.is_empty() && ui.button("Create").clicked() {
                let mut campaign = CampaignGui::new_refreshable(
                    SaveWithPath::new(Save::new(self.random_campaign_name_entry.clone())),
                    self.class_cache_rc.clone(),
                );
                self.random_campaign_name_entry.clear();
                self.random_campaign(&mut campaign);
                self.dock_state.push_to_first_leaf(campaign);
            }
        });
        if self.dock_state.find_active_focused().is_some() {
            ui.menu_button("Randomize Campaign", |ui| {
                ui.label("Level:");
                ui.add(egui::Slider::new(
                    &mut self.random_level,
                    0.0..=self.class_cache_rc.borrow().get_classes().len().to_f64(),
                ));
                if ui.button("Randomize").clicked() {
                    let mut new_gui = None;
                    if let Some((_, campaign_gui)) = self.dock_state.find_active_focused() {
                        new_gui = Some(campaign_gui.clone());
                    }
                    if let Some(campaign_gui) = &mut new_gui {
                        campaign_gui.clear_campaign();
                        self.random_campaign(campaign_gui);
                        if let Some((_, active_gui)) = self.dock_state.find_active_focused() {
                            *active_gui = campaign_gui.clone();
                        }
                    };
                }
            });
        }
    }

    fn refresh_rules(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if ui.button("Refresh Rules").clicked() {
            info_log_and_notify(&mut self.toasts, "Started refreshing rules!");
            self.rule_refresh_handle = RefCell::new(Some(
                self.rule_refresh_runtime
                    .spawn(crate::google::get_origins_and_classes()),
            ));
        }
        if self.rule_refresh_handle.borrow().is_some() {
            info!("Refreshing rules!");
            if self
                .rule_refresh_handle
                .borrow()
                .as_ref()
                .unwrap()
                .is_finished()
            {
                info_log_and_notify(&mut self.toasts, "Rules refreshed...");
                let refresh_handle = self.rule_refresh_handle.replace(None);
                match self
                    .rule_refresh_runtime
                    .block_on(async { join!(refresh_handle.unwrap()) })
                    .0
                    .unwrap()
                {
                    Ok(class_cache) => {
                        *self.class_cache_rc.borrow_mut() = class_cache;
                        if let Some(storage) = frame.storage_mut() {
                            eframe::set_value(
                                storage,
                                "class_cache",
                                &*self.class_cache_rc.borrow(),
                            );
                        }
                        info_log_and_notify(&mut self.toasts, "Campaign updated to new rules.");
                        if let Some((_, campaign_gui)) = self.dock_state.find_active_focused() {
                            campaign_gui.refresh_campaign();
                        }
                    }
                    Err(err) => {
                        error_log_and_notify(
                            &mut self.toasts,
                            format!("Error refreshing rules: {err}"),
                        );
                    }
                }
            }
        }
    }

    fn main_panel(&mut self, ctx: &egui::Context) {
        egui_dock::DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tab_viewer);
    }

    fn save_as(&mut self) -> bool {
        let dialog = FileDialog::new();
        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        let dialog = if let Some(path) = self.app_state.get_most_recent_campaign_path() {
            if let Some(converted_path) = path.to_str() {
                dialog.set_file_name(converted_path)
            } else {
                dialog
            }
        } else {
            dialog
        };

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let dialog = dialog
            .set_title("Save Campaign As")
            .add_filter("Greed Campaign", &["ron"]);

        dialog
            .save_file()
            .and_then(|picked_file| {
                self.dock_state.find_active_focused().map(|(_, campaign)| {
                    match campaign.save_to(picked_file.clone()) {
                        Ok(()) => {
                            info_log_and_notify(
                                &mut self.toasts,
                                format!(
                                    "Successfully saved file to {}",
                                    picked_file.to_string_lossy()
                                ),
                            );

                            if let Some(path) = campaign.set_path(picked_file) {
                                self.app_state.add_new_path_to_history(path);
                            }
                            true
                        }
                        Err(err) => {
                            error_log_and_notify(
                                &mut self.toasts,
                                format!("Error while saving to file {picked_file:?}: {err}"),
                            );
                            false
                        }
                    }
                })
            })
            .unwrap_or(false)
    }

    fn open_campaign(&mut self) {
        let dialog = FileDialog::new();

        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        let dialog = if let Some(path) = self.app_state.get_most_recent_campaign_path() {
            if let Some(converted_path) = path.to_str() {
                dialog.set_file_name(converted_path)
            } else {
                dialog
            }
        } else {
            dialog
        };

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        let dialog = dialog
            .set_title("Open Campaign")
            .add_filter("Greed Campaign", &["ron"]);

        if let Some(picked_file) = dialog.pick_file() {
            self.open_new_save(&picked_file.into_os_string());
        }
    }

    fn open_new_save(&mut self, new_save_path: &OsString) {
        let current_save_saved =
            if let Some((_, campaign_gui)) = self.dock_state.find_active_focused() {
                campaign_gui.save().map_or_else(
                    || true,
                    |result| {
                        result
                            .map_err(|err| {
                                error_log_and_notify(
                                    &mut self.toasts,
                                    format!("Unable to save current save: {err}",),
                                );
                            })
                            .is_ok()
                    },
                )
            } else {
                true
            };
        if current_save_saved {
            if let Ok(new_save) = SaveWithPath::from_path(new_save_path).map_err(|err| {
                error_log_and_notify(
                    &mut self.toasts,
                    format!(
                        "Error loading save file at '{}': {err}",
                        new_save_path.to_string_lossy()
                    ),
                );
            }) {
                let mut campaign_gui =
                    CampaignGui::new_refreshable(new_save, self.class_cache_rc.clone());
                campaign_gui.refresh_campaign();
                self.dock_state.push_to_first_leaf(campaign_gui);
            }
        }
    }

    fn perform_on_all_guis_mut<T>(&mut self, gui_action: &dyn Fn(&mut CampaignGui) -> T) -> Vec<T> {
        let mut results = Vec::with_capacity(self.dock_state.main_surface().num_tabs());
        for node in self.dock_state.main_surface_mut().iter_mut() {
            if let egui_dock::node::Node::Leaf { tabs, .. } = node {
                for gui in tabs {
                    results.push(gui_action(gui));
                }
            }
        }
        results
    }

    fn random_campaign(&self, campaign: &mut CampaignGui) {
        let class_cache = self.class_cache_rc.borrow();
        let origins = class_cache.get_origins();
        let classes = class_cache.get_classes();
        let origin = fastrand::choice(origins).unwrap().clone();
        let level: usize = unsafe {
            if origin.get_name() == "Human" {
                (self.random_level + 1.0)
                    .to_f64()
                    .clamp(0.0, classes.len().to_f64())
            } else {
                self.random_level
            }
            .to_int_unchecked()
        };
        campaign.change_origin(Some(origin));

        let mut character_classes: Vec<Class> = Vec::with_capacity(level);

        for _ in 0..level {
            let available_classes = classes
                .iter()
                .filter(|class| {
                    !character_classes
                        .iter()
                        .any(|current_class| class.get_name() == current_class.get_name())
                        && class.get_class_available(&character_classes)
                })
                .collect::<Vec<_>>();

            if let Some(class) = fastrand::choice(available_classes) {
                character_classes.push((*class).clone());
            } else {
                break;
            }
        }

        for class in character_classes {
            campaign.add_new_class(class);
        }

        campaign.refresh_campaign();
    }

    fn update_app(&mut self, requested: bool) -> bool {
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
                    if self_update::version::bump_is_greater(
                        cargo_crate_version!(),
                        &release.version,
                    )
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
                                        .set_description(
                                            "Please restart app to play updated version",
                                        )
                                        .set_buttons(rfd::MessageButtons::Ok)
                                        .show();
                                    return true;
                                }
                                Err(err) => {
                                    error_log_and_notify(
                                        &mut self.toasts,
                                        format!("Unable to update app: {err}"),
                                    );
                                    return false;
                                }
                            },
                            MessageDialogResult::No => return false,
                            _ => unreachable!(),
                        }
                    } else if requested {
                        info_log_and_notify(&mut self.toasts, "No update Available");
                        return false;
                    }
                }
                Err(err) => {
                    error_log_and_notify(
                        &mut self.toasts,
                        format!("Error Finding the latest release: {err}"),
                    );
                    return false;
                }
            },
            Err(err) => {
                error_log_and_notify(
                    &mut self.toasts,
                    format!("Error Finding the latest release: {err}"),
                );
                return false;
            }
        }
        true
    }
}

impl eframe::App for GuiGreedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        frame.set_window_title("Greed Console");

        self.toasts.show(ctx);

        self.menu_panel(ctx, frame);

        self.main_panel(ctx);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        info!("Saving! AppState: {:?}", self.app_state);
        eframe::set_value(storage, eframe::APP_KEY, &self.app_state);
        let stored_cache = eframe::get_value::<ClassCache>(storage, "class_cache");
        let current_cache = self.class_cache_rc.borrow();
        if stored_cache.is_none() || stored_cache.is_some_and(|cache| cache != *current_cache) {
            info!("Saving! AppState: {:?}", current_cache);
            eframe::set_value(storage, "class_cache", &*current_cache);
        }
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn on_close_event(&mut self) -> bool {
        let tabs_to_close = Rc::new(RefCell::new(Vec::new()));
        let tabs_to_close_clone = tabs_to_close.clone();
        let campaign_results =
            self.perform_on_all_guis_mut(&move |campaign_gui: &mut CampaignGui| {
                if campaign_gui.save_is_dirty() {
                    match MessageDialog::new()
                        .set_level(rfd::MessageLevel::Info)
                        .set_title("Save Campaign?")
                        .set_buttons(rfd::MessageButtons::YesNoCancel)
                        .show()
                    {
                        MessageDialogResult::Yes => {
                            if campaign_gui.get_path().is_some() {
                                campaign_gui.save();
                                tabs_to_close_clone
                                    .borrow_mut()
                                    .push(campaign_gui.get_save().get_campaign_name());
                                true
                            } else {
                                false
                            }
                        }
                        MessageDialogResult::No => {
                            tabs_to_close_clone
                                .borrow_mut()
                                .push(campaign_gui.get_save().get_campaign_name());
                            true
                        }
                        MessageDialogResult::Cancel => false,
                        _ => {
                            unreachable!()
                        }
                    }
                } else {
                    tabs_to_close_clone
                        .borrow_mut()
                        .push(campaign_gui.get_save().get_campaign_name());
                    true
                }
            });

        self.tab_viewer.set_tabs_to_close(&tabs_to_close.borrow());

        campaign_results.is_empty() || !campaign_results.iter().any(|result| !result)
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        info!("App Shutting down!");
    }
}
