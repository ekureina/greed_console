use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};
use crate::model::{game_state::GameState, sheets::Character};

use eframe::egui;
use eframe::glow::Context;
use eframe::Storage;
use log::info;

use std::time::Duration;

#[derive(Default)]
pub struct GuiGreedApp {
    game_state: GameState,
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        info!("Starting up app!");
        let storage = cc.storage.unwrap();
        let character: Character = if let Some(str) = storage.get_string("test.character_sheet") {
            info!("Saved Character: {}", str);
            ron::from_str(&str).unwrap()
        } else {
            info!("Starting with fresh character!");
            Character::default()
        };
        let mut game_state = GameState::default();
        for action in &character.get_special_actions() {
            game_state.new_special(action.get_name(), action.get_description());
        }
        GuiGreedApp {
            game_state,
            primary_actions: character.get_primary_actions(),
            secondary_actions: character.get_secondary_actions(),
            ..Default::default()
        }
    }

    fn globals_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("globals")
            .resizable(false)
            .show(ctx, |ui| {
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
            });
    }

    fn extras_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("extras")
            .resizable(false)
            .show(ctx, |ui| {
                ui.group(|ui| {
                    ui.label("Refresh Actions");
                    ui.group(|ui| {
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

                        if ui
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
                })
            });
    }

    fn main_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
                        ui.label(action.get_name())
                            .on_hover_text(action.get_description());
                    }
                });
            });

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

                for action in &self.secondary_actions {
                    ui.label(action.get_name())
                        .on_hover_text(action.get_description());
                }
            });

            // List of all non-Action surge specials
            if !self.game_state.get_special_actions().is_empty()
                && self.game_state.get_special_actions().len() > 1
                || !self.game_state.get_special_action_exists(&"Action Surge")
            {
                ui.group(|ui| {
                    ui.label("Specials:");

                    #[allow(clippy::explicit_iter_loop)]
                    for action in self.game_state.get_special_actions().clone().iter() {
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

            if ui
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
            self.primary_actions.push(PrimaryAction::new(
                self.primary_add_text_buffer.clone(),
                self.primary_add_description_text_buffer.clone(),
            ));
            self.primary_add_text_buffer.clear();
            self.primary_add_description_text_buffer.clear();
        }
    }

    fn add_new_secondary(&mut self) {
        if !self.secondary_add_text_buffer.is_empty() {
            self.secondary_actions.push(SecondaryAction::new(
                self.secondary_add_text_buffer.clone(),
                self.secondary_add_description_text_buffer.clone(),
            ));
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

        self.globals_panel(ctx);

        self.extras_panel(ctx);

        self.main_panel(ctx);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        // Save character sheet if there is any data to store
        if !self.primary_actions.is_empty()
            || !self.secondary_actions.is_empty()
            || !self.game_state.get_special_actions().is_empty()
        {
            let mut character_sheet = Character::new();
            character_sheet.add_primary_actions(self.primary_actions.clone());
            character_sheet.add_secondary_actions(self.secondary_actions.clone());
            character_sheet.add_special_actions(self.game_state.get_special_actions().clone());
            let previous_sheet = eframe::get_value::<Character>(storage, "test.character_sheet");
            if previous_sheet.is_none() || previous_sheet.unwrap() != character_sheet {
                let ron_sheet = ron::to_string(&character_sheet).unwrap();
                info!("Saving! Character sheet: {}", ron_sheet);
                storage.set_string("test.character_sheet", ron_sheet);
                storage.flush();
            } else {
                info!("No update to character sheet, not saving!");
            }
        } else {
            info!("No Character sheet, not saving!");
        }
    }

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn on_exit(&mut self, _gl: Option<&Context>) {
        info!("App Shutting down!");
    }
}
