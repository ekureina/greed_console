use super::state::AppState;
use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};
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
    special_refresh_text_buffer: String,
    special_add_description_text_buffer: String,
}

impl GuiGreedApp {
    pub fn new(cc: &eframe::CreationContext) -> GuiGreedApp {
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

        let mut game_state = GameState::default();
        for action in &character.get_special_actions() {
            game_state.new_special(action.get_name(), action.get_description());
        }
        GuiGreedApp {
            game_state,
            app_state,
            new_campaign_text: String::default(),
            primary_actions: character.get_primary_actions(),
            secondary_actions: character.get_secondary_actions(),
            ..Default::default()
        }
    }

    fn menu_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
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
                    });
                    ui.menu_button("Actions", |ui| {
                        ui.menu_button("Add Primary", |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.primary_add_text_buffer);

                            ui.label("Description:");
                            ui.text_edit_multiline(&mut self.primary_add_description_text_buffer);

                            if ui.button("Add").clicked() {
                                self.add_new_primary();
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Add Secondary", |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.secondary_add_text_buffer);

                            ui.label("Description:");
                            ui.text_edit_multiline(&mut self.secondary_add_description_text_buffer);

                            if ui.button("Add").clicked() {
                                self.add_new_secondary();
                            }
                        });
                        ui.menu_button("Add Special", |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.special_add_text_buffer);

                            ui.label("Description:");
                            ui.text_edit_multiline(&mut self.special_add_description_text_buffer);

                            if ui.button("Add").clicked() {
                                self.add_new_special();
                            }
                        });
                    });
                    if ui.button("Next Battle").clicked() {
                        self.game_state.next_battle();
                    }

                    if ui.button("Next Turn").clicked() {
                        self.game_state.next_turn();
                    }
                });
            });
    }
    fn switch_campaign(&mut self, new_campaign_name: String) {
        self.app_state.set_current_campaign(new_campaign_name);
        let current_campaign = self.app_state.get_current_campaign().unwrap();
        self.primary_actions = current_campaign.get_primary_actions();
        self.secondary_actions = current_campaign.get_secondary_actions();
        self.game_state = GameState::default();
        for action in &current_campaign.get_special_actions() {
            self.game_state
                .new_special(action.get_name(), action.get_description());
        }
    }

    fn extras_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("extras")
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.label("Refresh Actions");
                    if self.game_state.get_special_action_exists(&"Action Surge")
                        || self
                            .primary_actions
                            .iter()
                            .any(|action| action.get_name() == "Execute")
                    {
                        ui.group(|ui| {
                            ui.set_max_width(200.0);
                            ui.label("Primary Refreshing Actions");
                            if self.game_state.get_special_action_exists(&"Action Surge")
                                && ui
                                    .add_enabled(
                                        self.game_state.get_special_action_usable(&"Action Surge")
                                            && self.game_state.get_any_special_usable(),
                                        egui::Button::new("Action Surge (Special Action)"),
                                    )
                                    .on_hover_text(
                                        self.game_state
                                            .get_special_description(&"Action Surge")
                                            .unwrap(),
                                    )
                                    .clicked()
                            {
                                self.game_state.extra_primary();
                                self.game_state.extra_primary();
                                self.game_state.use_special("Action Surge");
                            }

                            if self
                                .primary_actions
                                .iter()
                                .any(|action| action.get_name() == "Execute")
                                && ui
                                    .add_enabled(
                                        self.game_state.get_primary_usable(),
                                        egui::Button::new("Execute (Primary Action)"),
                                    )
                                    .clicked()
                            {
                                self.game_state.extra_primary();
                                self.game_state.use_primary();
                            }
                        });
                    }
                    ui.group(|ui| {
                        ui.label("Secondary Refreshing Actions");
                        if ui.button("Rally Wink Targeted").clicked() {
                            self.game_state.extra_secondary();
                        }
                    });
                });

                ui.group(|ui| {
                    ui.label("Other Extras:");
                    if ui
                        .add_enabled(
                            self.game_state
                                .get_special_actions()
                                .iter()
                                .any(SpecialAction::is_usable),
                            egui::Button::new("Exhaust Special Actions"),
                        )
                        .clicked()
                    {
                        self.game_state.exhaust_specials();
                    }
                    ui.group(|ui| {
                        ui.label("Refresh Special:");
                        if ui
                            .text_edit_singleline(&mut self.special_refresh_text_buffer)
                            .lost_focus()
                        {
                            self.refresh_special();
                        }

                        if ui.button("Refresh").clicked() {
                            self.refresh_special();
                        }
                    });
                })
            });
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
                    if self.game_state.get_special_actions().len() > 1
                        || (self.game_state.get_special_actions().len() == 1
                            && !self.game_state.get_special_action_exists(&"Action Surge"))
                    {
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
                "Primary Actions Remaining: {}",
                self.game_state.get_primary_actions()
            ));
            if ui
                .add_enabled(
                    self.game_state.get_primary_usable(),
                    egui::Button::new("Use Primary"),
                )
                .clicked()
            {
                self.game_state.use_primary();
            }

            ui.group(|ui| {
                ui.label("Primary Actions:");
                for action in &self.primary_actions {
                    if action.get_name() != "Execute" {
                        ui.label(action.get_name())
                            .on_hover_text(action.get_description());
                    }
                }
            });
        });
    }

    fn secondary_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 2.0);
        ui.group(|ui| {
            ui.label(format!(
                "Secondary Actions Remaining: {}",
                self.game_state.get_secondary_actions()
            ));
            if ui
                .add_enabled(
                    self.game_state.get_secondary_usable(),
                    egui::Button::new("Use Secondary"),
                )
                .clicked()
            {
                self.game_state.use_secondary();
            }

            ui.group(|ui| {
                ui.label("Secondary Actions:");
                for action in &self.secondary_actions {
                    ui.label(action.get_name())
                        .on_hover_text(action.get_description());
                }
            });
        });
    }

    fn special_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Specials:");

            for action in &self.game_state.get_special_actions().clone() {
                if !action.is_named("Action Surge")
                    && ui
                        .add_enabled(
                            action.is_usable() && self.game_state.get_any_special_usable(),
                            egui::Button::new(action.get_name()),
                        )
                        .on_hover_text(action.get_description())
                        .clicked()
                {
                    self.game_state.use_special(action.get_name());
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

    fn refresh_special(&mut self) {
        if !self.special_refresh_text_buffer.is_empty() {
            self.game_state
                .refresh_special(self.special_refresh_text_buffer.clone());
            self.special_refresh_text_buffer.clear();
        }
    }
}

impl eframe::App for GuiGreedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        frame.set_window_title(&format!(
            "Greed Console (Campaign: {}; Turn: {})",
            self.app_state
                .get_current_campaign_name()
                .unwrap_or_else(|| String::from("None")),
            self.game_state.get_turn_num().to_string()
        ));

        self.menu_panel(ctx);

        self.extras_panel(ctx);

        self.main_panel(ctx);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        info!("Saving! AppState: {:?}", self.app_state);
        eframe::set_value(storage, eframe::APP_KEY, &self.app_state);
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        info!("App Shutting down!");
    }
}
