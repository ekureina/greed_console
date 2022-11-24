use crate::model::game_state::GameState;

use eframe::egui;

#[derive(Default)]
pub struct PlayerGuiGreedApp {
    game_state: GameState,
}

impl PlayerGuiGreedApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for PlayerGuiGreedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        frame.set_window_title("Greed Console");

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
                    ui.label("Turn: ".to_owned() + &self.game_state.get_turn_num().to_string());
                    if ui.button("Next Turn").clicked() {
                        self.game_state.next_turn();
                    }
                });
            });

        egui::SidePanel::right("extras")
            .resizable(false)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("primary_extras")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        if ui
                            .add_enabled(
                                self.game_state.get_special_usable(),
                                egui::Button::new("Action Surge"),
                            )
                            .clicked()
                        {
                            self.game_state.extra_primary();
                            self.game_state.extra_primary();
                            self.game_state.use_special();
                        }

                        if ui
                            .add_enabled(
                                self.game_state.get_primary_usable(),
                                egui::Button::new("Execute"),
                            )
                            .clicked()
                        {
                            self.game_state.extra_primary();
                            self.game_state.use_primary();
                        }
                    });

                if ui.button("Rally Wink Targeted").clicked() {
                    self.game_state.extra_secondary();
                }
            });

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

            if ui
                .add_enabled(
                    self.game_state.get_special_usable(),
                    egui::Button::new("Use Special"),
                )
                .clicked()
            {
                self.game_state.use_special();
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
}
