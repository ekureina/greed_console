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
    current_campaign_text: String,
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
            current_campaign_text: String::default(),
            primary_actions: character.get_primary_actions(),
            secondary_actions: character.get_secondary_actions(),
            ..Default::default()
        }
    }

    fn campaign_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("campaign")
            .resizable(false)
            .show(ctx, |ui| {
                let click_response = ui.add(
                    egui::Label::new(format!(
                        "Current Campaign: {}",
                        self.app_state
                            .get_current_campaign_name()
                            .unwrap_or_default()
                    ))
                    .sense(egui::Sense::click()),
                );
                let popup_id = ui.make_persistent_id("campaign_poupup");
                if click_response.clicked() {
                    info!("Opened popup!");
                    ui.memory().open_popup(popup_id);
                }
                egui::popup::popup_below_widget(ui, popup_id, &click_response, |ui| {
                    ui.group(|ui| {
                        ui.set_min_width(200.0);
                        ui.label("Switch Campaign:");
                        ui.text_edit_singleline(&mut self.current_campaign_text);
                        if ui.button("Switch").clicked() {
                            info!("Button Clicked!");
                            if !self.current_campaign_text.is_empty() {
                                if !self
                                    .app_state
                                    .get_campaign_exists(self.current_campaign_text.clone())
                                {
                                    self.app_state
                                        .create_campaign(self.current_campaign_text.clone());
                                }
                                self.app_state
                                    .set_current_campaign(self.current_campaign_text.clone());
                                self.current_campaign_text.clear();
                                let current_campaign =
                                    self.app_state.get_current_campaign().unwrap();
                                self.primary_actions = current_campaign.get_primary_actions();
                                self.secondary_actions = current_campaign.get_secondary_actions();
                                self.game_state = GameState::default();
                                for action in &current_campaign.get_special_actions() {
                                    self.game_state
                                        .new_special(action.get_name(), action.get_description());
                                }
                            }
                        }
                    })
                });
            });
    }

    fn globals_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("globals")
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_width(125.0);
                ui.set_height(725.0);
                egui::TopBottomPanel::top("battle")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        if ui.button("Next Battle").clicked() {
                            self.game_state.next_battle();
                        }
                    });

                ui.group(|ui| {
                    ui.label(format!("Turn: {}", self.game_state.get_turn_num()));
                    if ui.button("Next Turn").clicked() {
                        self.game_state.next_turn();
                    }
                });

                ui.group(|ui| {
                    ui.label("Add Actions:");
                    ui.group(|ui| {
                        ui.label("Add Primary:");
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.primary_add_text_buffer);

                        ui.label("Description:");
                        ui.text_edit_multiline(&mut self.primary_add_description_text_buffer);

                        if ui.button("Add").clicked() {
                            self.add_new_primary();
                        }
                    });
                    ui.group(|ui| {
                        ui.label("Add Secondary:");
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.secondary_add_text_buffer);

                        ui.label("Description:");
                        ui.text_edit_multiline(&mut self.secondary_add_description_text_buffer);

                        if ui.button("Add").clicked() {
                            self.add_new_secondary();
                        }
                    });
                    ui.group(|ui| {
                        ui.label("Add Special:");

                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.special_add_text_buffer);

                        ui.label("Description:");
                        ui.text_edit_multiline(&mut self.special_add_description_text_buffer);

                        if ui.button("Add").clicked() {
                            self.add_new_special();
                        }
                    });
                });
            });
    }

    fn extras_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("extras")
            .resizable(false)
            .show(ctx, |ui| {
                ui.group(|ui| {
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
            ui.set_width(200.0);
            // List of all non-Execute primary actions
            if self.primary_actions.len() > 1
                || (self.primary_actions.len() == 1
                    && self.primary_actions[0].get_name() != "Execute")
            {
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

            if !self.secondary_actions.is_empty() {
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

            // List of all non-Action surge specials
            if self.game_state.get_special_actions().len() > 1
                || (self.game_state.get_special_actions().len() == 1
                    && !self.game_state.get_special_action_exists(&"Action Surge"))
            {
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
        frame.set_window_title("Greed Console");

        self.campaign_panel(ctx);

        self.globals_panel(ctx);

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
