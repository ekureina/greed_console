use super::state::AppState;
use super::widgets::panels::StatsPanel;
use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};
use crate::model::classes::{Class, ClassCache, ClassPassive, ClassUtility};
use crate::model::game_state::GameState;
use crate::model::save::Save;

use eframe::egui;
use eframe::glow::Context;
use eframe::Storage;
use egui_file::FileDialog;
use log::{error, info};
use tokio::join;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use std::cell::RefCell;
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
    game_state: GameState,
    app_state: AppState,
    new_campaign_name_entry: String,
    current_save: Option<Save>,
    open_file_dialog: Option<FileDialog>,
    save_file_dialog: Option<FileDialog>,
    utilities: Vec<ClassUtility>,
    show_utilities: bool,
    passives: Vec<ClassPassive>,
    primary_actions: Vec<PrimaryAction>,
    secondary_actions: Vec<SecondaryAction>,
    class_cache: ClassCache,
    character_origin: Option<Class>,
    character_classes: Vec<Class>,
    rule_refresh_runtime: Runtime,
    rule_refresh_handle: RefCell<Option<JoinHandle<ClassCache>>>,
    show_save_on_quit_dialog: bool,
    allowed_to_quit: bool,
}

impl GuiGreedApp {
    pub fn new(cc: &eframe::CreationContext, class_cache: ClassCache) -> GuiGreedApp {
        info!("Starting up app!");
        let mut app_state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppState::new()
        };

        let mut game_state = GameState::default();

        let rule_refresh_runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name("rules-refresh-worker")
            .enable_all()
            .build()
            .unwrap();

        if let Some(save) = app_state
            .get_current_campaign_path()
            .map(Save::from_file)
            .and_then(|result| {
                result
                    .map_err(|err| {
                        error!("{}", err);
                        app_state.clear_current_campaign_path();
                    })
                    .ok()
            })
        {
            game_state.set_round(save.get_round());

            let character = save.get_character();

            let (utilities, passives, primary_actions, secondary_actions, mut special_actions) =
                character.get_all_actions(&class_cache);
            let used_specials = save.get_used_specials();
            for action in &mut special_actions {
                if used_specials.contains(&action.get_name()) {
                    action.use_action();
                }
                game_state.push_special(action.clone());
            }
            let character_origin = character
                .get_origin()
                .and_then(|origin_name| class_cache.get_origin(origin_name.as_str()))
                .cloned();

            let character_classes = class_cache.map_to_concrete_classes(character.get_classes());

            GuiGreedApp {
                game_state,
                app_state,
                new_campaign_name_entry: String::new(),
                current_save: Some(save),
                open_file_dialog: None,
                save_file_dialog: None,
                utilities,
                show_utilities: true,
                passives,
                primary_actions,
                secondary_actions,
                class_cache,
                character_origin,
                character_classes,
                rule_refresh_runtime,
                rule_refresh_handle: RefCell::new(None),
                show_save_on_quit_dialog: false,
                allowed_to_quit: false,
            }
        } else {
            GuiGreedApp {
                game_state,
                app_state,
                new_campaign_name_entry: String::new(),
                current_save: None,
                open_file_dialog: None,
                save_file_dialog: None,
                utilities: vec![],
                show_utilities: true,
                passives: vec![],
                primary_actions: vec![],
                secondary_actions: vec![],
                class_cache,
                character_origin: None,
                character_classes: vec![],
                rule_refresh_runtime,
                rule_refresh_handle: RefCell::new(None),
                show_save_on_quit_dialog: false,
                allowed_to_quit: false,
            }
        }
    }

    fn stats_panel(&mut self, ui: &mut egui::Ui) {
        ui.add(StatsPanel::new(
            self.current_save.as_mut().unwrap(),
            &mut self.game_state,
        ));
    }

    fn menu_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu")
            .resizable(false)
            .show(ctx, |ui| {
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

                    if let Some(dialog) = &mut self.open_file_dialog {
                        if dialog.show(ctx).selected() {
                            if let Some(file) = dialog.path() {
                                let picked_file = file.to_str().map_or_else(String::new, String::from);
                                self.current_save = Some(Save::from_file(&file).unwrap());
                                self.app_state.set_current_campaign_path(picked_file);
                                self.refresh_campaign();
                            }
                        }
                    }

                    if let Some(dialog) = &mut self.save_file_dialog {
                        if dialog.show(ctx).selected() {
                            if let Some(file) = dialog.path() {
                                if let Some(save) = self.current_save.clone() {
                                    if let Some(path) = file.to_str() {
                                        self.app_state.set_current_campaign_path(path);
                                    }

                                    save.to_file(file).unwrap();
                                }
                            }
                        }
                    }

                    ui.menu_button("View", |ui| {
                        self.view_menu(ui);
                    });

                    if self.current_save.is_some() {
                        ui.menu_button("Actions", |ui| {
                            if ui.button("Refresh Secondary Action").clicked() {
                                self.game_state.extra_secondary();
                            }

                            if self
                                .game_state
                                .get_special_actions()
                                .iter()
                                .any(SpecialAction::is_usable)
                                && ui.button("Exhaust All Specials").clicked()
                            {
                                self.game_state.exhaust_specials();
                                if let Some(save) = &mut self.current_save {
                                self.game_state.get_special_actions().iter().for_each(|action| save.use_special(action.get_name()));
                                    }
                            }
                        });

                        ui.menu_button("Classes", |ui| self.classes_menu(ui));
                        self.next_part_buttons(ui);

                        ui.menu_button("Stats", |ui| self.stats_panel(ui));
                    }

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
                self.current_save = Some(Save::new(&self.new_campaign_name_entry));
                self.new_campaign_name_entry.clear();
                self.refresh_campaign();
            }
        });
        if ui.button("Open").clicked() {
            let mut dialog = FileDialog::open_file(None);
            dialog.open();
            self.open_file_dialog = Some(dialog);
        }

        if ui.button("Save").clicked() {
            let start_path = self.app_state.get_current_campaign_path().map(Into::into);
            let mut dialog = FileDialog::save_file(start_path);
            dialog.open();
            self.save_file_dialog = Some(dialog);
        }

        if self.current_save.is_some() {
            ui.menu_button("Origin", |ui| {
                let old_origin = self.character_origin.clone();
                for origin in self.class_cache.get_origins() {
                    ui.radio_value(
                        &mut self.character_origin,
                        Some(origin.clone()),
                        origin.get_name(),
                    );
                }
                if self.character_origin != old_origin {
                    self.change_origin(self.character_origin.clone());
                }
            });
        }
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.show_utilities, "Utilities");
    }

    fn next_part_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("Next Battle").clicked() {
            self.game_state.next_battle();
            if let Some(save) = &mut self.current_save {
                save.refresh_specials();
                save.inc_battle();
            }
        }

        if ui.button("Next Turn").clicked() {
            self.game_state.next_turn();
            if let Some(save) = &mut self.current_save {
                save.set_round(self.game_state.get_round_num());
            }
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
                self.class_cache = self
                    .rule_refresh_runtime
                    .block_on(async { join!(refresh_handle.unwrap()) })
                    .0
                    .unwrap();
                eframe::set_value(
                    frame.storage_mut().unwrap(),
                    "class_cache",
                    &self.class_cache,
                );
                info!("Campaign updated to new rules.");
                self.refresh_campaign();
            }
        }
    }

    fn classes_menu(&mut self, ui: &mut egui::Ui) {
        if self.character_classes.len() != self.class_cache.get_class_cache_count() {
            ui.menu_button("Add", |ui| {
                let mut classes_to_add = vec![];
                let current_class_names = self
                    .character_classes
                    .iter()
                    .map(Class::get_name)
                    .collect::<Vec<String>>();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for class in self.class_cache.get_classes() {
                        if !self.character_classes.contains(class)
                            && class.get_class_available(&current_class_names)
                            && ui.button(class.get_name()).clicked()
                        {
                            classes_to_add.push(class.clone());
                        }
                    }
                });
                for class in classes_to_add {
                    self.add_new_class(class);
                }
            });
        }
        if !self.character_classes.is_empty() {
            ui.menu_button("Remove", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for class in self.character_classes.clone() {
                        if ui.button(class.get_name()).clicked() {
                            self.remove_class(&class);
                        }
                    }
                });
            });
        }
    }

    fn refresh_campaign(&mut self) {
        if let Some(save) = self.current_save.clone() {
            let current_campaign = save.get_character();
            let (utility, passive, primary, secondary, mut special) =
                current_campaign.get_all_actions(&self.class_cache);
            self.primary_actions = primary;
            self.secondary_actions = secondary;
            self.utilities = utility;
            self.passives = passive;
            self.game_state = GameState::default();
            let used_specials = save.get_used_specials();
            for action in &mut special {
                if used_specials.contains(&action.get_name()) {
                    action.use_action();
                }
                self.game_state.push_special(action.clone());
            }
            let new_origin = current_campaign
                .get_origin()
                .and_then(|origin_name| self.class_cache.get_origin(origin_name.as_str()))
                .cloned();

            self.character_origin = new_origin;
            self.game_state.set_round(save.get_round());
        }
    }

    fn main_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if !self.utilities.is_empty() && self.show_utilities {
                        ui.vertical(|ui| {
                            self.utility_panel(ui);
                        });
                    }
                    if !self.passives.is_empty() {
                        ui.vertical(|ui| {
                            self.passive_panel(ui);
                        });
                    }
                    // List of all non-Execute primary actions
                    if self.primary_actions.len() > 1
                        || (self.primary_actions.len() == 1
                            && self.primary_actions[0].get_name() != "Execute")
                    {
                        ui.vertical(|ui| {
                            self.primary_panel(ui);
                        });
                    }

                    if !self.secondary_actions.is_empty() {
                        ui.vertical(|ui| {
                            self.secondary_panel(ui);
                        });
                    }

                    // List of all non-Action surge specials
                    if !(self.game_state.get_special_actions().is_empty()) {
                        ui.vertical(|ui| {
                            self.special_panel(ui);
                        });
                    }
                });

                if !(self.primary_actions.is_empty()
                    && self.secondary_actions.is_empty()
                    && self.game_state.get_special_actions().is_empty())
                    && ui
                        .add_enabled(
                            self.game_state.get_inspiration_usable(),
                            egui::Button::new("Use Inspiration"),
                        )
                        .clicked()
                {
                    self.game_state.use_inspiration();
                }
            });
        });
    }

    fn utility_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 5.0);
        ui.group(|ui| {
            ui.label("Utilities:");
            for utility in &self.utilities {
                ui.label(utility.get_name())
                    .on_hover_text(utility.get_description());
            }
        });
    }

    fn passive_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 4.0);
        ui.group(|ui| {
            ui.label("Passives:");
            for passive in &self.passives {
                ui.label(passive.get_name())
                    .on_hover_text(passive.get_description());
            }
        });
    }

    fn primary_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 3.0);
        ui.group(|ui| {
            ui.label(format!(
                "Primary Actions ({} remaining):",
                self.game_state.get_primary_actions()
            ));
            for action in &self.primary_actions {
                if ui
                    .add_enabled(
                        self.game_state.get_primary_usable(),
                        egui::Button::new(action.get_name()),
                    )
                    .on_hover_text(action.get_description())
                    .on_disabled_hover_text(action.get_description())
                    .clicked()
                    && action.get_name() != "Execute"
                {
                    self.game_state.use_primary();
                }
            }
        });
    }

    fn secondary_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 2.0);
        ui.group(|ui| {
            ui.label(format!(
                "Secondary Actions ({} remaining):",
                self.game_state.get_secondary_actions()
            ));
            for action in &self.secondary_actions {
                if ui
                    .add_enabled(
                        self.game_state.get_secondary_usable(),
                        egui::Button::new(action.get_name()),
                    )
                    .on_hover_text(action.get_description())
                    .on_disabled_hover_text(action.get_description())
                    .clicked()
                {
                    self.game_state.use_secondary();
                }
            }
        });
    }

    fn special_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Specials:");

            for action in &self.game_state.get_special_actions().clone() {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            action.is_usable() && self.game_state.get_any_special_usable(),
                            egui::Button::new(action.get_name()),
                        )
                        .on_hover_text(action.get_description())
                        .on_disabled_hover_text(action.get_description())
                        .clicked()
                    {
                        self.game_state.use_special(action.get_name().as_str());
                        if let Some(save) = &mut self.current_save {
                            save.use_special(action.get_name());
                        }
                        if action.is_named("Action Surge") {
                            self.game_state.extra_primary();
                            self.game_state.extra_primary();
                        }
                    }
                    if !action.is_usable() && ui.button("Refresh").clicked() {
                        self.refresh_special(&action.get_name());
                        if let Some(save) = &mut self.current_save {
                            save.refresh_special(action.get_name());
                        }
                    }
                });
            }
        });
    }

    fn add_new_class(&mut self, class: Class) {
        self.utilities.push(class.get_utility());
        self.passives.push(class.get_passive());
        self.primary_actions.push(class.get_primary_action());
        self.secondary_actions.push(class.get_secondary_action());
        self.game_state.push_special(class.get_special_action());

        if let Some(campaign) = self.current_save.as_mut().map(Save::get_character_mut) {
            campaign.add_class(class.get_name());
        }
        self.character_classes.push(class);
    }

    fn change_origin(&mut self, new_origin: Option<Class>) {
        if let Some(campaign) = self.current_save.as_mut().map(Save::get_character_mut) {
            campaign.replace_origin(new_origin.map(|class| class.get_name()));
            self.refresh_campaign();
        }
    }

    fn remove_class(&mut self, class: &Class) {
        if let Some(utility_index) = self
            .utilities
            .iter()
            .position(|action| action.clone() == class.get_utility())
        {
            self.utilities.remove(utility_index);
        }
        if let Some(passive_index) = self
            .passives
            .iter()
            .position(|action| action.clone() == class.get_passive())
        {
            self.passives.remove(passive_index);
        }
        if let Some(primary_index) = self
            .primary_actions
            .iter()
            .position(|action| action.clone() == class.get_primary_action())
        {
            self.primary_actions.remove(primary_index);
        }
        if let Some(secondary_index) = self
            .secondary_actions
            .iter()
            .position(|action| action.clone() == class.get_secondary_action())
        {
            self.secondary_actions.remove(secondary_index);
        }
        if let Some(special_index) = self
            .game_state
            .get_special_actions()
            .iter()
            .position(|action| action.clone() == class.get_special_action())
        {
            self.game_state.remove_special_action(special_index);
        }
        if let Some(class_index) = self
            .character_classes
            .iter()
            .position(|stored_class| stored_class == class)
        {
            self.character_classes.remove(class_index);
        }
        if let Some(campaign) = self.current_save.as_mut().map(Save::get_character_mut) {
            campaign.remove_class(class.get_name());
        }
    }

    fn refresh_special(&mut self, special_name: &str) {
        self.game_state.refresh_special(special_name);
    }
}

impl eframe::App for GuiGreedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        frame.set_window_title("Greed Console");

        self.menu_panel(ctx, frame);

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
                            if let Some(path) = self.app_state.get_current_campaign_path() {
                                if let Some(save) = &self.current_save {
                                    save.to_file(path).unwrap();
                                    frame.close();
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
            eframe::set_value(storage, "class_cache", &self.class_cache);
        }
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn on_close_event(&mut self) -> bool {
        if let Some(path) = self.app_state.get_current_campaign_path() {
            if let Some(save) = &self.current_save {
                if let Ok(old_save) = Save::from_file(path) {
                    if old_save != save.clone() {
                        self.show_save_on_quit_dialog = true;
                        return self.allowed_to_quit;
                    }
                }
            }
        }

        true
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        info!("App Shutting down!");
    }
}
