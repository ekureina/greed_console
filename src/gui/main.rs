use super::campaign::CampaignGui;
use super::state::AppState;
use super::tabs::CampaignTabViewer;
use crate::google::GetOriginsAndClassesError;
use crate::gui::util::error_log_and_notify;
use crate::model::classes::ClassCache;
use crate::model::save::{Save, SaveWithPath};

use eframe::egui;
use eframe::glow::Context;
use eframe::Storage;
use egui_dock::{Style, Tree};
use egui_file::FileDialog;
use egui_notify::Toasts;
use log::info;
use tokio::join;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::PathBuf;
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
    tree: Tree<CampaignGui>,
    tab_viewer: CampaignTabViewer,
    app_state: AppState,
    new_campaign_name_entry: String,
    file_dialog: Option<FileDialog>,
    class_cache_rc: Rc<RefCell<ClassCache>>,
    rule_refresh_runtime: Runtime,
    rule_refresh_handle: RefCell<Option<JoinHandle<Result<ClassCache, GetOriginsAndClassesError>>>>,
    show_save_on_quit_dialog: bool,
    allowed_to_quit: bool,
    toasts: Toasts,
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

        let tree = Tree::new(campaign_guis);

        GuiGreedApp {
            tree,
            tab_viewer: CampaignTabViewer {},
            app_state,
            new_campaign_name_entry: String::new(),
            file_dialog: None,
            class_cache_rc,
            rule_refresh_runtime,
            rule_refresh_handle: RefCell::new(None),
            show_save_on_quit_dialog: false,
            allowed_to_quit: false,
            toasts,
        }
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

                self.refresh_rules(ui, frame);

                ui.hyperlink_to("Greed Rulset", "https://docs.google.com/document/d/1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4/edit?usp=sharing");
            });
        });
    }

    fn campaign_menu(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(200.0);
        ui.menu_button("New", |ui| {
            ui.text_edit_singleline(&mut self.new_campaign_name_entry);
            if ui.button("Create").clicked() {
                let mut campaign_gui = CampaignGui::new_refreshable(
                    SaveWithPath::new(Save::new(self.new_campaign_name_entry.clone())),
                    Rc::new(RefCell::new(ClassCache::new(vec![], vec![]))),
                );
                campaign_gui.refresh_campaign();
                self.new_campaign_name_entry.clear();
                self.tree.push_to_first_leaf(campaign_gui);
            }
        });
        if ui.button("Open").clicked() {
            self.open_open_dialog();
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

        if let Some((_, campaign_gui)) = self.tree.find_active() {
            if ui.button("Save").clicked() {
                info!("Attempting to save campaign!");
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
                    self.open_save_as_dialog();
                }
            }
        }

        if self.tree.find_active().is_some() && ui.button("Save As...").clicked() {
            self.open_save_as_dialog();
        }
    }

    fn refresh_rules(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if ui.button("Refresh Rules").clicked() {
            info!("Started refreshing rules!");
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
                info!("Rules refreshed...");
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
                        info!("Campaign updated to new rules.");
                        if let Some((_, campaign_gui)) = self.tree.find_active() {
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
        egui_dock::DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tab_viewer);
    }

    fn open_save_as_dialog(&mut self) {
        let mut dialog = FileDialog::save_file(
            self.app_state
                .get_most_recent_campaign_path()
                .map(PathBuf::from),
        );
        dialog.open();
        self.file_dialog = Some(dialog);
    }

    fn open_open_dialog(&mut self) {
        let mut dialog = FileDialog::open_file(
            self.app_state
                .get_most_recent_campaign_path()
                .map(PathBuf::from),
        );
        dialog.open();
        self.file_dialog = Some(dialog);
    }

    fn open_new_save(&mut self, new_save_path: &OsString) {
        let current_save_saved = if let Some((_, campaign_gui)) = self.tree.find_active() {
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
                self.tree.push_to_first_leaf(campaign_gui);
            }
        }
    }

    fn display_dialog_boxes(&mut self, ctx: &egui::Context) {
        if let Some(dialog) = &mut self.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    match dialog.dialog_type() {
                        egui_file::DialogType::OpenFile => {
                            self.app_state.add_new_path_to_history(file.clone());
                            self.open_new_save(&file.as_os_str().to_owned());
                        }
                        egui_file::DialogType::SaveFile => {
                            if let Some((_, campaign_gui)) = &mut self.tree.find_active() {
                                match campaign_gui.save_to(file.clone()) {
                                    Ok(()) => {
                                        info!(
                                            "Successfully saved file to {}",
                                            file.to_string_lossy()
                                        );

                                        if let Some(path) = campaign_gui.set_path(file) {
                                            self.app_state.add_new_path_to_history(path);
                                        }
                                    }
                                    Err(err) => {
                                        error_log_and_notify(
                                            &mut self.toasts,
                                            format!("Error while saving to file {file:?}: {err}"),
                                        );
                                    }
                                }
                            }
                        }
                        egui_file::DialogType::SelectFolder => {
                            error_log_and_notify(
                                &mut self.toasts,
                                "Unreachable File dialog reached, need to handle!",
                            );
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for GuiGreedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        frame.set_window_title("Greed Console");

        self.toasts.show(ctx);

        self.menu_panel(ctx, frame);

        self.display_dialog_boxes(ctx);

        self.main_panel(ctx);

        if self.show_save_on_quit_dialog {
            egui::Window::new("Save Campaign?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_save_on_quit_dialog = false;
                        }

                        if ui.button("Save").clicked() {
                            self.allowed_to_quit = true;
                            if let Some((_, campaign_gui)) = self.tree.find_active() {
                                if let Some(result) = campaign_gui.save() {
                                    match result {
                                        Err(err) => {
                                            error_log_and_notify(
                                                &mut self.toasts,
                                                format!("Error when saving file: {err}"),
                                            );
                                        }
                                        Ok(_) => {
                                            frame.close();
                                        }
                                    }
                                }
                            }
                        }

                        if ui.button("Quit").clicked() {
                            self.allowed_to_quit = true;
                            frame.close();
                        }
                    });
                });
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        info!("Saving! AppState: {:?}", self.app_state);
        eframe::set_value(storage, eframe::APP_KEY, &self.app_state);
        if eframe::get_value::<ClassCache>(storage, "class_cache").is_none() {
            eframe::set_value(storage, "class_cache", &*self.class_cache_rc.borrow());
        }
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn on_close_event(&mut self) -> bool {
        if let Some((_, campaign_gui)) = self.tree.find_active() {
            if let Some(path) = campaign_gui.get_path() {
                if let Ok(old_save) = Save::from_file(path) {
                    if old_save != *campaign_gui.get_save() {
                        self.show_save_on_quit_dialog = true;
                        return self.allowed_to_quit;
                    }
                } else {
                    self.show_save_on_quit_dialog = true;
                    return self.allowed_to_quit;
                }
            }
        }

        true
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        info!("App Shutting down!");
    }
}
