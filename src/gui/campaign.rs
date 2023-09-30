use std::{cell::RefCell, ffi::OsString, path::Path, rc::Rc};

use crate::model::{
    actions::{PrimaryAction, SecondaryAction},
    classes::{Class, ClassCache, ClassPassive, ClassUtility},
    game_state::GameState,
    save::{Save, SaveToFileError, SaveWithPath},
};

use super::widgets::panels::StatsPanel;

#[derive(Debug, Clone, PartialEq)]
pub struct CampaignGui {
    game_state: GameState,
    current_save: SaveWithPath,
    utilities: Vec<ClassUtility>,
    passives: Vec<ClassPassive>,
    primary_actions: Vec<PrimaryAction>,
    secondary_actions: Vec<SecondaryAction>,
    character_classes: Vec<Class>,
    character_origin: Option<Class>,
    class_cache: Rc<RefCell<ClassCache>>,
    description_hovering: bool,
}

impl CampaignGui {
    pub fn new_refreshable(
        current_save: SaveWithPath,
        class_cache: Rc<RefCell<ClassCache>>,
    ) -> CampaignGui {
        CampaignGui {
            game_state: GameState::default(),
            current_save,
            utilities: vec![],
            passives: vec![],
            primary_actions: vec![],
            secondary_actions: vec![],
            character_classes: vec![],
            character_origin: None,
            class_cache,
            description_hovering: true,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            self.campaign_menu(ui);

            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if !self.utilities.is_empty() {
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
        self.current_save
            .get_save_mut()
            .set_battle_power(self.game_state.get_battle_power());
        self.current_save
            .get_save_mut()
            .set_battle_defense(self.game_state.get_battle_defense());
    }

    pub fn get_level(&self) -> usize {
        let class_count = self.character_classes.len();
        if self
            .character_origin
            .as_ref()
            .is_some_and(|origin| origin.get_name() == "Human")
        {
            if class_count < 1 {
                0
            } else {
                class_count - 1
            }
        } else {
            class_count
        }
    }

    fn campaign_menu(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Origin", |ui| {
                    let old_origin = self.character_origin.clone();
                    for origin in self.class_cache.borrow().get_origins() {
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
                ui.menu_button("Actions", |ui| {
                    if ui.button("Refresh Primary Action").clicked() {
                        self.game_state.extra_primary();
                    }
                    if ui.button("Refresh Secondary Action").clicked() {
                        self.game_state.extra_secondary();
                    }
                    if ui.button("Refresh Special Action").clicked() {
                        self.game_state.extra_special();
                    }
                    if ui.button("Refresh Inspiration").clicked() {
                        self.game_state.refresh_inspiration();
                    }
                });

                ui.menu_button("Classes", |ui| self.classes_menu(ui));
                self.next_part_buttons(ui);

                ui.menu_button("Stats", |ui| {
                    ui.add(StatsPanel::new(
                        self.current_save.get_save(),
                        &mut self.game_state,
                    ))
                });

                ui.checkbox(&mut self.description_hovering, "Hover Description");
            });
        });
    }

    fn utility_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 5.0);
        ui.group(|ui| {
            ui.label("Utilities:");
            for utility in &self.utilities {
                if self.description_hovering {
                    ui.label(egui::RichText::new(utility.get_name()).strong())
                        .on_hover_text(utility.get_description());
                } else {
                    ui.label(egui::RichText::new(utility.get_name()).strong());
                    ui.label(utility.get_description());
                }
            }
        });
    }

    fn passive_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 4.0);
        ui.group(|ui| {
            ui.label("Passives:");
            for passive in &self.passives {
                if self.description_hovering {
                    ui.label(egui::RichText::new(passive.get_name()).strong())
                        .on_hover_text(passive.get_description());
                } else {
                    ui.label(egui::RichText::new(passive.get_name()).strong());
                    ui.label(passive.get_description());
                }
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
                let button_response = ui.add_enabled(
                    self.game_state.get_primary_usable(),
                    egui::Button::new(action.get_name()),
                );
                let button_response = if self.description_hovering {
                    button_response
                        .on_hover_text(action.get_description())
                        .on_disabled_hover_text(action.get_description())
                } else {
                    ui.label(action.get_description());
                    button_response
                };

                if button_response.clicked() && action.get_name() != "Execute" {
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
                let button_response = ui.add_enabled(
                    self.game_state.get_secondary_usable(),
                    egui::Button::new(action.get_name()),
                );
                let button_response = if self.description_hovering {
                    button_response
                        .on_hover_text(action.get_description())
                        .on_disabled_hover_text(action.get_description())
                } else {
                    ui.label(action.get_description());
                    button_response
                };
                if button_response.clicked() {
                    self.game_state.use_secondary();
                }
            }
        });
    }

    fn special_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Specials:");

            for action in &self.game_state.get_special_actions().clone() {
                let button_response = ui.add_enabled(
                    action.is_usable() && self.game_state.get_any_special_usable(),
                    egui::Button::new(action.get_name()),
                );
                let button_response = if self.description_hovering {
                    button_response
                        .on_hover_text(action.get_description())
                        .on_disabled_hover_text(action.get_description())
                } else {
                    ui.label(action.get_description());
                    button_response
                };
                if button_response.clicked() {
                    if action
                        .get_description()
                        .to_lowercase()
                        .contains("repeatable")
                    {
                        self.game_state.use_repeatable_special();
                    } else {
                        self.game_state.use_special(action.get_name().as_str());
                        self.current_save
                            .get_save_mut()
                            .use_special(action.get_name().as_str());
                    }
                    if action.is_named("Action Surge") {
                        self.game_state.extra_primary();
                        self.game_state.extra_primary();
                    }
                }
            }
        });
    }

    pub fn refresh_campaign(&mut self) {
        let current_campaign = self.current_save.get_save().get_character();
        let class_cache = self.class_cache.borrow();
        let (utility, passive, primary, secondary, mut special) =
            current_campaign.get_all_actions(&class_cache);
        self.primary_actions = primary;
        self.secondary_actions = secondary;
        self.utilities = utility;
        self.passives = passive;
        self.game_state = GameState::default();
        let used_specials = self.current_save.get_save().get_used_specials();
        for action in &mut special {
            if used_specials.contains(&action.get_name()) {
                action.use_action();
            }
            self.game_state.push_special(action.clone());
        }
        let new_origin = current_campaign
            .get_origin()
            .and_then(|origin_name| class_cache.get_origin(origin_name.as_str()))
            .cloned();
        self.character_origin = new_origin;
        self.character_classes =
            class_cache.map_to_concrete_classes(current_campaign.get_classes());
        self.game_state
            .set_round(self.current_save.get_save().get_round());
        self.game_state
            .change_power_for_battle(self.current_save.get_save().get_battle_power());
        self.game_state
            .change_defense_for_battle(self.current_save.get_save().get_battle_defense());
    }

    pub fn change_origin(&mut self, new_origin: Option<Class>) {
        let campaign = self.current_save.get_save_mut().get_character_mut();
        campaign.replace_origin(new_origin.map(|class| class.get_name()));
        self.refresh_campaign();
    }

    fn next_part_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("Next Battle").clicked() {
            self.next_battle();
        }

        if ui.button("Next Turn").clicked() {
            self.next_turn();
        }
    }

    pub fn next_battle(&mut self) {
        self.game_state.next_battle();
        self.current_save.get_save_mut().refresh_specials();
        self.current_save.get_save_mut().inc_battle();
    }

    pub fn next_turn(&mut self) {
        self.game_state.next_turn();
        self.current_save
            .get_save_mut()
            .set_round(self.game_state.get_round_num());
    }

    fn classes_menu(&mut self, ui: &mut egui::Ui) {
        if self.character_classes.len() != self.class_cache.borrow().get_class_cache_count() {
            ui.menu_button("Add", |ui| {
                let mut classes_to_add = vec![];
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for class in self.class_cache.borrow().get_classes() {
                        if !self.character_classes.contains(class)
                            && class.get_class_available(&self.character_classes)
                        {
                            let class_label = class.get_name()
                                + &(match class.get_level() {
                                    Some(level) => format!(" (Level {level})"),
                                    None => String::new(),
                                });
                            if ui.button(class_label).clicked() {
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
        if !self.character_classes.is_empty() {
            ui.menu_button("Remove", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for class in self.character_classes.clone() {
                        let class_label = class.get_name()
                            + &(match class.get_level() {
                                Some(level) => format!(" (Level {level})"),
                                None => String::new(),
                            });
                        if ui.button(class_label).clicked() {
                            self.remove_class(&class);
                        }
                    }
                });
            });
        }
    }

    pub fn add_new_class(&mut self, class: Class) {
        self.utilities.extend_from_slice(class.get_utilities());
        self.passives.extend_from_slice(class.get_passives());
        self.primary_actions.push(class.get_primary_action());
        self.secondary_actions.push(class.get_secondary_action());
        self.game_state.push_special(class.get_special_action());

        let campaign = self.current_save.get_save_mut().get_character_mut();
        campaign.add_class(class.get_name());
        self.character_classes.push(class);
    }

    fn remove_class(&mut self, class: &Class) {
        self.utilities.retain(|utility| {
            !class
                .get_utilities()
                .iter()
                .map(ClassUtility::get_name)
                .collect::<Vec<_>>()
                .contains(&utility.get_name())
        });
        self.passives.retain(|passive| {
            !class
                .get_passives()
                .iter()
                .map(ClassPassive::get_name)
                .collect::<Vec<_>>()
                .contains(&passive.get_name())
        });
        self.primary_actions
            .retain(|primary| class.get_primary_action().get_name() != primary.get_name());
        self.secondary_actions
            .retain(|secondary| class.get_secondary_action().get_name() != secondary.get_name());
        if let Some(special_index) = self
            .game_state
            .get_special_actions()
            .iter()
            .position(|action| action.clone().get_name() == class.get_special_action().get_name())
        {
            self.game_state.remove_special_action(special_index);
        }
        self.character_classes
            .retain(|stored_class| stored_class.get_name() == class.get_name());
        self.current_save
            .get_save_mut()
            .get_character_mut()
            .remove_class(class.get_name());

        let mut subclasses = vec![];
        for remaining_class in &self.character_classes {
            if remaining_class
                .get_class_requirements()
                .contains(&class.get_name())
            {
                subclasses.push(remaining_class.clone());
            }
        }

        for subclass in &subclasses {
            self.remove_class(subclass);
        }
    }

    pub fn clear_campaign(&mut self) {
        self.current_save.get_save_mut().get_character_mut().clear();
        self.current_save.get_save_mut().refresh_specials();
        self.refresh_campaign();
    }

    pub fn get_save(&self) -> &Save {
        self.current_save.get_save()
    }

    pub fn save(&self) -> Option<Result<(), SaveToFileError>> {
        self.current_save.save()
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), SaveToFileError> {
        self.current_save.save_to(path)
    }

    pub fn set_path<P: Into<OsString>>(&mut self, path: P) -> Option<OsString> {
        self.current_save.set_path(path)
    }

    pub fn get_path(&self) -> &Option<OsString> {
        self.current_save.get_path()
    }

    pub fn save_is_dirty(&self) -> bool {
        self.current_save.is_dirty()
    }
}
