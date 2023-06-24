use egui::Widget;

use crate::model::{game_state::GameState, save::Save};

#[derive(Debug)]
pub struct StatsPanel<'a> {
    save: &'a Save,
    game_state: &'a mut GameState,
}

impl<'a> StatsPanel<'a> {
    pub fn new(save: &'a Save, game_state: &'a mut GameState) -> StatsPanel<'a> {
        StatsPanel { save, game_state }
    }
}

impl<'a> Widget for StatsPanel<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.label(format!("Campaign: {}", self.save.get_campaign_name()));
            ui.label(format!("Batttle Number: {}", self.save.get_battle()));
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
        })
        .response
    }
}
