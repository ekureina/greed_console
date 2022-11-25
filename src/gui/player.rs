use crate::model::{actions::SpecialAction, game_state::GameState};

use eframe::egui;

#[derive(Default)]
pub struct GuiGreedApp {
    game_state: GameState,
    special_add_text_buffer: String,
}

impl GuiGreedApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
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
                    ui.label("Add Special:");
                    if ui
                        .text_edit_singleline(&mut self.special_add_text_buffer)
                        .lost_focus()
                    {
                        self.add_new_special();
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
            });

            if !self.game_state.get_special_actions().is_empty() {
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

    fn add_new_special(&mut self) {
        if !self.special_add_text_buffer.is_empty() {
            self.game_state
                .new_special(self.special_add_text_buffer.clone());
            self.special_add_text_buffer.clear();
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
}
