use super::state::AppState;
use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};
use crate::model::classes::{Class, ClassCache};
use crate::model::game_state::GameState;

use eframe::egui;
use eframe::glow::Context;
use eframe::Storage;
use log::info;
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
    new_campaign_text: String,
    primary_actions: Vec<PrimaryAction>,
    secondary_actions: Vec<SecondaryAction>,
    class_cache: ClassCache,
    character_origin: Option<Class>,
    character_classes: Vec<Class>,
    rule_refresh_runtime: Runtime,
    rule_refresh_handle: RefCell<Option<JoinHandle<ClassCache>>>,
}

impl GuiGreedApp {
    pub fn new(cc: &eframe::CreationContext, class_cache: ClassCache) -> GuiGreedApp {
        info!("Starting up app!");
        let app_state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppState::new()
        };
        let character = app_state
            .get_current_campaign()
            .map(Clone::clone)
            .unwrap_or_default();

        let (primary_actions, secondary_actions, special_actions) =
            character.get_all_actions(&class_cache);

        let mut game_state = GameState::default();
        for action in special_actions {
            game_state.push_special(action);
        }

        let character_origin = character.get_origin().and_then(|origin_name| {
            class_cache
                .get_origins()
                .iter()
                .find(|origin| origin.get_name() == origin_name.clone())
                .map(std::clone::Clone::clone)
        });

        let character_classes = character
            .get_classes()
            .iter()
            .filter_map(|class_name| {
                class_cache
                    .get_classes()
                    .iter()
                    .find(|class| class.get_name() == class_name.clone())
                    .map(std::clone::Clone::clone)
            })
            .collect();

        let rule_refresh_runtime = tokio::runtime::Builder::new_multi_thread().thread_name("rules-refresh-worker").enable_all().build().unwrap();

        GuiGreedApp {
            game_state,
            app_state,
            new_campaign_text: String::default(),
            primary_actions,
            secondary_actions,
            class_cache,
            character_origin,
            character_classes,
            rule_refresh_runtime,
            rule_refresh_handle: RefCell::new(None),
        }
    }

    fn stats_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(format!(
            "Campaign: {}",
            self.app_state
                .get_current_campaign_name()
                .unwrap_or_else(|| String::from("None"))
        ));
        ui.label(format!("Round Number: {}", self.game_state.get_round_num()));
        ui.label(format!("Turn: {}", self.game_state.get_turn_side()));
        ui.menu_button(format!("Power: {}", self.game_state.get_power()), |ui| {
            if ui.button("Increment Power for Turn").clicked() {
                self.game_state.change_power_for_turn(1);
            }
            if ui.button("Decrement Power for Turn").clicked() {
                self.game_state.change_power_for_turn(-1);
            }
            if ui.button("Increment Power for Round").clicked() {
                self.game_state.change_power_for_round(1);
            }
            if ui.button("Decrement Power for Round").clicked() {
                self.game_state.change_power_for_round(-1);
            }
            if ui.button("Increment Power for Battle").clicked() {
                self.game_state.change_power_for_battle(1);
            }
            if ui.button("Decrement Power for Battle").clicked() {
                self.game_state.change_power_for_battle(-1);
            }
        });
        ui.menu_button(
            format!("Defense: {}", self.game_state.get_defense()),
            |ui| {
                if ui.button("Increment Defense for Turn").clicked() {
                    self.game_state.change_defense_for_turn(1);
                }
                if ui.button("Decrement Defense for Turn").clicked() {
                    self.game_state.change_defense_for_turn(-1);
                }
                if ui.button("Increment Defense for Round").clicked() {
                    self.game_state.change_defense_for_round(1);
                }
                if ui.button("Decrement Defense for Round").clicked() {
                    self.game_state.change_defense_for_round(-1);
                }
                if ui.button("Increment Defense for Battle").clicked() {
                    self.game_state.change_defense_for_battle(1);
                }
                if ui.button("Decrement Defense for Battle").clicked() {
                    self.game_state.change_defense_for_battle(-1);
                }
            },
        );
    }

    fn menu_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu")
            .resizable(false)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    let about_response = ui.button("About");
                    let about_popup_id = ui.make_persistent_id("about_popup_id");
                    if about_response.clicked() {
                        ui.memory().toggle_popup(about_popup_id);
                    }
                    egui::popup_below_widget(ui, about_popup_id, &about_response, |ui| {
                        ui.set_min_width(450.0);
                        ui.label(COPYRIGHT_NOTICE);
                    });

                    ui.menu_button("Campaign", |ui| {
                        ui.set_min_width(200.0);
                        ui.menu_button("New", |ui| {
                            if (ui
                                .text_edit_singleline(&mut self.new_campaign_text)
                                .lost_focus()
                                || ui.button("Create").clicked())
                                && !self.new_campaign_text.is_empty()
                            {
                                info!("New Campaign: {}", self.new_campaign_text);
                                if !self
                                    .app_state
                                    .get_campaign_exists(self.new_campaign_text.clone())
                                {
                                    self.app_state
                                        .create_campaign(self.new_campaign_text.clone());
                                    self.switch_campaign(self.new_campaign_text.clone());
                                }
                                self.new_campaign_text.clear();
                                ui.close_menu();
                            }
                        });
                        if self.app_state.get_campaign_names().iter().any(|name| *name.clone() != self.app_state.get_current_campaign_name().unwrap_or_else(|| "None".to_owned())) {
                            ui.menu_button("Switch", |ui| {
                                for campaign in self.app_state.get_campaign_names() {
                                    if self.app_state.get_current_campaign_name().is_some() && self.app_state.get_current_campaign_name().unwrap() != campaign && ui.button(campaign.clone()).clicked() {
                                        self.switch_campaign(campaign);
                                        ui.close_menu();
                                    }
                                }
                            });
                        }
                        if !self.app_state.get_campaign_names().is_empty() {
                            ui.menu_button("Remove", |ui| {
                                for campaign in self.app_state.get_campaign_names() {
                                    if ui.button(campaign.clone()).clicked() {
                                        self.app_state.remove_campaign(campaign);
                                        ui.close_menu();
                                    }
                                }
                            });
                        }
                        ui.menu_button("Origin", |ui| {
                            let old_origin = self.character_origin.clone();
                            for origin in &self.class_cache.get_origins() {
                                ui.radio_value(&mut self.character_origin, Some(origin.clone()), origin.get_name());
                            }
                            if self.character_origin != old_origin {
                                self.change_origin(old_origin, self.character_origin.clone());
                            }
                        });
                    });
                    
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
                    }

                    ui.menu_button("Classes", |ui| self.classes_menu(ui));
                    self.next_part_buttons(ui);

                    ui.menu_button("Stats", |ui| self.stats_panel(ui));

                    self.refresh_rules(ui, frame);

                    ui.hyperlink_to("Greed Rulset", "https://docs.google.com/document/d/1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4/edit?usp=sharing");
                });
            });
    }

    fn next_part_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("Next Battle").clicked() {
            self.game_state.next_battle();
        }

        if ui.button("Next Turn").clicked() {
            self.game_state.next_turn();
        }
    }

    fn refresh_rules(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if ui.button("Refresh Rules").clicked() {
            info!("Started refreshing rules!");
            self.rule_refresh_handle = RefCell::new(Some(self.rule_refresh_runtime.spawn(crate::google::get_origins_and_classes())));
        }
        if self.rule_refresh_handle.borrow().is_some() {
            info!("Refreshing rules!");
            if self.rule_refresh_handle.borrow().as_ref().unwrap().is_finished() {
                info!("Rules refreshed...");
                let refresh_handle = self.rule_refresh_handle.replace(None);
                self.class_cache = self.rule_refresh_runtime.block_on(async { join!(refresh_handle.unwrap()) }).0.unwrap();
                eframe::set_value(frame.storage_mut().unwrap(), "class_cache", &self.class_cache);
                if let Some(campaign_name) = self.app_state.get_current_campaign_name() {
                    self.switch_campaign(campaign_name);
                }
                info!("Campaign updated to new rules.");
            }
        }
    }

    fn classes_menu(&mut self, ui: &mut egui::Ui) {
        if self.character_classes.len() != self.class_cache.get_classes().len() {
            ui.menu_button("Add", |ui| {
                let mut classes_to_add = vec![];
                let current_class_names = self
                    .character_classes
                    .iter()
                    .map(Class::get_name)
                    .collect::<Vec<String>>();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for class in &self.class_cache.get_classes() {
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

    fn switch_campaign(&mut self, new_campaign_name: String) {
        self.app_state.set_current_campaign(new_campaign_name);
        let current_campaign = self.app_state.get_current_campaign().unwrap();
        let (primary, secondary, special) = current_campaign.get_all_actions(&self.class_cache);
        self.primary_actions = primary;
        self.secondary_actions = secondary;
        self.game_state = GameState::default();
        let old_origin = self.character_origin.clone();
        let new_origin = current_campaign.get_origin().and_then(|origin_name| {
            self.class_cache
                .get_origins()
                .iter()
                .find(|origin| origin.get_name() == origin_name)
                .map(std::clone::Clone::clone)
        });
        self.change_origin(old_origin, new_origin);
        for action in special {
            self.game_state.push_special(action);
        }
    }

    fn main_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal(|ui| {
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
                        self.game_state.use_special(action.get_name());
                        if action.is_named("Action Surge") {
                            self.game_state.extra_primary();
                            self.game_state.extra_primary();
                        }
                    }
                    if !action.is_usable() && ui.button("Refresh").clicked() {
                        self.refresh_special(action.get_name());
                    }
                });
            }
        });
    }

    fn add_new_class(&mut self, class: Class) {
        self.primary_actions.push(class.get_primary_action());
        self.secondary_actions.push(class.get_secondary_action());
        self.game_state.push_special(class.get_special_action());
        if let Some(campaign) = self.app_state.get_current_campaign_mut() {
            campaign.add_class(class.get_name());
        }
        self.character_classes.push(class);
    }

    fn change_origin(&mut self, old_origin: Option<Class>, new_origin: Option<Class>) {
        if let Some(origin) = old_origin {
            if let Some(primary_index) = self
                .primary_actions
                .iter()
                .position(|action| action.clone() == origin.get_primary_action())
            {
                self.primary_actions.remove(primary_index);
            }
            if let Some(secondary_index) = self
                .secondary_actions
                .iter()
                .position(|action| action.clone() == origin.get_secondary_action())
            {
                self.secondary_actions.remove(secondary_index);
            }
            if let Some(special_index) = self
                .game_state
                .get_special_actions()
                .iter()
                .position(|action| action.clone() == origin.get_special_action())
            {
                self.game_state.remove_special_action(special_index);
            }
        }
        if let Some(new_origin) = new_origin.clone() {
            if new_origin.get_name() != "Human" {
                self.primary_actions
                    .insert(0, new_origin.get_primary_action());
                self.secondary_actions
                    .insert(0, new_origin.get_secondary_action());
                self.game_state
                    .insert_special(0, new_origin.get_special_action());
            }
        }
        if let Some(campaign) = self.app_state.get_current_campaign_mut() {
            campaign.replace_origin(new_origin.map(|class| class.get_name()));
        }
    }

    fn remove_class(&mut self, class: &Class) {
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
        if let Some(campaign) = self.app_state.get_current_campaign_mut() {
            campaign.remove_class(class.get_name());
        }
    }

    fn refresh_special(&mut self, special_name: String) {
        self.game_state.refresh_special(special_name);
    }
}

impl eframe::App for GuiGreedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        frame.set_window_title("Greed Console");

        self.menu_panel(ctx, frame);

        self.main_panel(ctx);
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

    fn on_exit(&mut self, _gl: Option<&Context>) {
        info!("App Shutting down!");
    }
}
