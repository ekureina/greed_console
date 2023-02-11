use super::state::AppState;
use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};
use crate::model::classes::{Class, ClassCache};
use crate::model::game_state::GameState;

use eframe::egui;
use eframe::glow::Context;
use eframe::Storage;
use log::info;

use std::time::Duration;

#[derive(Default)]
pub struct GuiGreedApp {
    game_state: GameState,
    app_state: AppState,
    new_campaign_text: String,
    primary_actions: Vec<PrimaryAction>,
    primary_add_text_buffer: String,
    primary_add_description_text_buffer: String,
    secondary_actions: Vec<SecondaryAction>,
    secondary_add_text_buffer: String,
    secondary_add_description_text_buffer: String,
    special_add_text_buffer: String,
    special_add_description_text_buffer: String,
    class_cache: ClassCache,
    character_race: Option<Class>,
    character_classes: Vec<Class>,
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

        let character_race = character
            .get_race()
            .map(|race_name| {
                class_cache
                    .get_races()
                    .iter()
                    .find(|race| race.get_name() == race_name.clone())
                    .map(|race| race.clone())
            })
            .flatten();

        GuiGreedApp {
            game_state,
            app_state,
            new_campaign_text: String::default(),
            primary_actions,
            secondary_actions,
            class_cache,
            character_race,
            ..Default::default()
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
                                    self.new_campaign_text.clear();
                                }
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Switch", |ui| {
                            for campaign in self.app_state.get_campaign_names() {
                                if ui.button(campaign.clone()).clicked() {
                                    self.switch_campaign(campaign);
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.menu_button("Remove", |ui| {
                            for campaign in self.app_state.get_campaign_names() {
                                if ui.button(campaign.clone()).clicked() {
                                    self.app_state.remove_campaign(campaign);
                                    ui.close_menu();
                                }
                            }
                        });
                        ui.menu_button("Origin", |ui| {
                            let old_race = self.character_race.clone();
                            for race in &self.class_cache.get_races() {
                                ui.radio_value(&mut self.character_race, Some(race.clone()), race.get_name());
                            }
                            if self.character_race != old_race {
                                self.change_race(old_race, self.character_race.clone());
                            }
                        });
                    });
                    ui.menu_button("Actions", |ui| self.actions_menu(ui));
                    ui.menu_button("Classes", |ui| self.classes_menu(ui));
                    if ui.button("Next Battle").clicked() {
                        self.game_state.next_battle();
                    }

                    if ui.button("Next Turn").clicked() {
                        self.game_state.next_turn();
                    }

                    ui.menu_button("Stats", |ui| self.stats_panel(ui));
                    if ui.button("Refresh Rules").clicked() {
                        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                        let (races, classes) = rt.block_on(crate::google::get_races_and_classes());
                        self.class_cache = ClassCache::new(races, classes);
                        eframe::set_value(frame.storage_mut().unwrap(), "class_cache", &self.class_cache);
                        if let Some(campaign_name) = self.app_state.get_current_campaign_name() {
                            self.switch_campaign(campaign_name);
                        }
                    }
                    ui.hyperlink_to("Greed Rulset", "https://docs.google.com/document/d/1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4/edit?usp=sharing");
                });
            });
    }

    fn actions_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Primary", |ui| {
            ui.menu_button("Add", |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.primary_add_text_buffer);

                ui.label("Description:");
                ui.text_edit_multiline(&mut self.primary_add_description_text_buffer);

                if ui.button("Add").clicked() {
                    self.add_new_primary();
                    ui.close_menu();
                }
            });
            ui.menu_button("Remove", |ui| {
                for (idex, action) in self.primary_actions.clone().iter().enumerate() {
                    if ui.button(action.get_name()).clicked() {
                        info!("Removing Primary Action: {}", action.get_name());
                        self.primary_actions.remove(idex);
                        if let Some(campaign) = self.app_state.get_current_campaign_mut() {
                            campaign.remove_primary_action(idex);
                        }
                    }
                }
            });
        });
        ui.menu_button("Secondary", |ui| {
            ui.menu_button("Add", |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.secondary_add_text_buffer);

                ui.label("Description:");
                ui.text_edit_multiline(&mut self.secondary_add_description_text_buffer);

                if ui.button("Add").clicked() {
                    self.add_new_secondary();
                }
            });
            ui.menu_button("Remove", |ui| {
                for (idex, action) in self.secondary_actions.clone().iter().enumerate() {
                    if ui.button(action.get_name()).clicked() {
                        info!("Removing Secondaary Action: {}", action.get_name());
                        self.secondary_actions.remove(idex);
                        if let Some(campaign) = self.app_state.get_current_campaign_mut() {
                            campaign.remove_secondary_action(idex);
                        }
                    }
                }
            });
            if ui.button("Refresh via Other Player Target").clicked() {
                self.game_state.extra_secondary();
            }
        });
        ui.menu_button("Special", |ui| {
            ui.menu_button("Add", |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.special_add_text_buffer);

                ui.label("Description:");
                ui.text_edit_multiline(&mut self.special_add_description_text_buffer);

                if ui.button("Add").clicked() {
                    self.add_new_special();
                }
            });
            ui.menu_button("Remove", |ui| {
                for (idex, action) in self
                    .app_state
                    .get_current_campaign()
                    .unwrap()
                    .get_special_actions()
                    .iter()
                    .enumerate()
                {
                    if ui.button(action.get_name()).clicked() {
                        info!("Removing Special Action: {}", action.get_name());
                        if let Some(campaign) = self.app_state.get_current_campaign_mut() {
                            self.game_state.remove_special_action(idex);
                            campaign.remove_special_action(idex);
                        }
                    }
                }
            });
            if self
                .game_state
                .get_special_actions()
                .iter()
                .any(|action| !action.is_usable())
            {
                ui.menu_button("Refresh", |ui| {
                    for action in self.game_state.get_special_actions().clone() {
                        if !action.is_usable() && ui.button(action.get_name()).clicked() {
                            self.refresh_special(action.get_name());
                        }
                    }
                });
            }
            if self
                .game_state
                .get_special_actions()
                .iter()
                .any(SpecialAction::is_usable)
            {
                if ui.button("Exhaust").clicked() {
                    self.game_state.exhaust_specials();
                }
            }
        });
    }

    fn classes_menu(&mut self, ui: &mut egui::Ui) {
        if self.character_classes.len() != self.class_cache.get_classes().len() {
            ui.menu_button("Add", |ui| {
                let mut classes_to_add = vec![];
                let current_class_names = self
                    .character_classes
                    .iter()
                    .map(|class| class.get_name().clone())
                    .collect::<Vec<String>>();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for class in &self.class_cache.get_classes() {
                        if !self.character_classes.contains(class)
                            && class.get_class_available(current_class_names.clone())
                        {
                            if ui.button(class.get_name()).clicked() {
                                classes_to_add.push(class.clone());
                            }
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
                        self.remove_class(class);
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
        let old_race = self.character_race.clone();
        let new_race = current_campaign
            .get_race()
            .map(|race_name| {
                self.class_cache
                    .get_races()
                    .iter()
                    .find(|race| race.get_name() == race_name)
                    .map(|race| race.clone())
            })
            .flatten();
        self.change_race(old_race, new_race);
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
                    .clicked()
                {
                    if action.get_name() != "Execute" {
                        self.game_state.use_primary();
                    }
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
                if ui
                    .add_enabled(
                        action.is_usable() && self.game_state.get_any_special_usable(),
                        egui::Button::new(action.get_name()),
                    )
                    .on_hover_text(action.get_description())
                    .clicked()
                {
                    self.game_state.use_special(action.get_name());
                    if action.is_named("Action Surge") {
                        self.game_state.extra_primary();
                        self.game_state.extra_primary();
                    }
                }
            }
        });
    }

    fn add_new_primary(&mut self) {
        if !self.primary_add_text_buffer.is_empty() {
            let primary_action = PrimaryAction::new(
                self.primary_add_text_buffer.clone(),
                self.primary_add_description_text_buffer.clone(),
            );
            self.primary_actions.push(primary_action.clone());
            if let Some(campaign) = self.app_state.get_current_campaign_mut() {
                campaign.add_primary_action(primary_action);
            }
            self.primary_add_text_buffer.clear();
            self.primary_add_description_text_buffer.clear();
        }
    }

    fn add_new_secondary(&mut self) {
        if !self.secondary_add_text_buffer.is_empty() {
            let secondary_action = SecondaryAction::new(
                self.secondary_add_text_buffer.clone(),
                self.secondary_add_description_text_buffer.clone(),
            );
            self.secondary_actions.push(secondary_action.clone());
            if let Some(campaign) = self.app_state.get_current_campaign_mut() {
                campaign.add_secondary_action(secondary_action);
            }
            self.secondary_add_text_buffer.clear();
            self.secondary_add_description_text_buffer.clear();
        }
    }

    fn add_new_special(&mut self) {
        if !self.special_add_text_buffer.is_empty() {
            self.game_state.new_special(
                self.special_add_text_buffer.clone(),
                self.special_add_description_text_buffer.clone(),
            );
            if let Some(campaign) = self.app_state.get_current_campaign_mut() {
                campaign.add_special_action(SpecialAction::new(
                    self.special_add_text_buffer.clone(),
                    self.special_add_description_text_buffer.clone(),
                ));
            }
            self.special_add_text_buffer.clear();
            self.special_add_description_text_buffer.clear();
        }
    }

    fn add_new_class(&mut self, class: Class) {
        self.primary_actions.push(class.get_primary_action());
        self.secondary_actions.push(class.get_secondary_action());
        self.game_state.push_special(class.get_special_action());
        if let Some(campaign) = self.app_state.get_current_campaign_mut() {
            campaign.add_class(class.get_name().clone());
        }
        self.character_classes.push(class);
    }

    fn change_race(&mut self, old_race: Option<Class>, new_race: Option<Class>) {
        if let Some(race) = old_race {
            if let Some(primary_index) = self
                .primary_actions
                .iter()
                .position(|action| action.clone() == race.get_primary_action())
            {
                self.primary_actions.remove(primary_index);
            }
            if let Some(secondary_index) = self
                .secondary_actions
                .iter()
                .position(|action| action.clone() == race.get_secondary_action())
            {
                self.secondary_actions.remove(secondary_index);
            }
            if let Some(special_index) = self
                .game_state
                .get_special_actions()
                .iter()
                .position(|action| action.clone() == race.get_special_action())
            {
                self.game_state.remove_special_action(special_index);
            }
        }
        if let Some(new_race) = new_race.clone() {
            if new_race.get_name() != "Human" {
                self.primary_actions
                    .insert(0, new_race.get_primary_action());
                self.secondary_actions
                    .insert(0, new_race.get_secondary_action());
                self.game_state
                    .insert_special(0, new_race.get_special_action());
            }
        }
        if let Some(campaign) = self.app_state.get_current_campaign_mut() {
            campaign.replace_race(new_race.map(|class| class.clone().get_name()));
        }
    }

    fn remove_class(&mut self, class: Class) {
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
            .position(|stored_class| stored_class.clone() == class)
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
        if let None = eframe::get_value::<ClassCache>(storage, "class_cache") {
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
