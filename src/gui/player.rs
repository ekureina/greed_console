use crate::model::game_state::GameState;

use eframe::egui;

#[derive(Default)]
pub struct PlayerGuiGreedApp {
    game_state: GameState
}

impl PlayerGuiGreedApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for PlayerGuiGreedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Turn: ".to_owned() + &self.game_state.get_turn_num().to_string());
            if ui.button("Next Turn").clicked() {
                self.game_state.next_turn();
            }

            if ui.button("Next Battle").clicked() {
                self.game_state.next_battle();
            }

            if ui.add_enabled(self.game_state.get_primary_usable(), egui::Button::new("Use Primary")).clicked() {
                self.game_state.use_primary();
            }

            if ui.add_enabled(self.game_state.get_secondary_usable(), egui::Button::new("Use Secondary")).clicked() {
                self.game_state.use_secondary();
            }

            if ui.add_enabled(self.game_state.get_special_usable(), egui::Button::new("Use Special")).clicked() {
                self.game_state.use_special();
            }

            if ui.add_enabled(self.game_state.get_inspiration_usable(), egui::Button::new("Use Inspiration")).clicked() {
                self.game_state.use_inspiration();
            }
        });
    }
}

