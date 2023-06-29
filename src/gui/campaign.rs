use std::{cell::RefCell, ffi::OsString, path::Path, rc::Rc};

use crate::model::{
    actions::{PrimaryAction, SecondaryAction, SpecialAction},
    classes::{Class, ClassCache, ClassPassive, ClassUtility},
    game_state::GameState,
    save::{Save, SaveToFileError, SaveWithPath},
};

use super::widgets::panels::StatsPanel;

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
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both().show(ui, |ui| {
            ui.vertical(|ui| {
                self.campaign_menu(ui);

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
        self.current_save
            .get_save_mut()
            .set_battle_power(self.game_state.get_battle_power());
        self.current_save
            .get_save_mut()
            .set_battle_defense(self.game_state.get_battle_defense());
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
                        self.game_state
                            .get_special_actions()
                            .iter()
                            .for_each(|action| {
                                self.current_save
                                    .get_save_mut()
                                    .use_special(action.get_name());
                            });
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
            });
        });
    }

    fn utility_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 5.0);
        ui.group(|ui| {
            ui.label("Utilities:");
            for utility in &self.utilities {
                ui.label(utility.get_name())
                    .on_hover_text(utility.get_description());
            }
        });
    }

    fn passive_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width() / 4.0);
        ui.group(|ui| {
            ui.label("Passives:");
            for passive in &self.passives {
                ui.label(passive.get_name())
                    .on_hover_text(passive.get_description());
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
                        self.game_state.use_special(action.get_name().as_str());
                        self.current_save
                            .get_save_mut()
                            .use_special(action.get_name().as_str());
                        if action.is_named("Action Surge") {
                            self.game_state.extra_primary();
                            self.game_state.extra_primary();
                        }
                    }
                    if !action.is_usable() && ui.button("Refresh").clicked() {
                        self.game_state.refresh_special(action.get_name().as_str());
                        self.current_save
                            .get_save_mut()
                            .refresh_special(action.get_name());
                    }
                });
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

    fn change_origin(&mut self, new_origin: Option<Class>) {
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
                let current_class_names = self
                    .character_classes
                    .iter()
                    .map(Class::get_name)
                    .collect::<Vec<String>>();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for class in self.class_cache.borrow().get_classes() {
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
        if !self.character_classes.is_empty() {
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
    }

    fn add_new_class(&mut self, class: Class) {
        self.utilities.push(class.get_utility());
        self.passives.push(class.get_passive());
        self.primary_actions.push(class.get_primary_action());
        self.secondary_actions.push(class.get_secondary_action());
        self.game_state.push_special(class.get_special_action());

        let campaign = self.current_save.get_save_mut().get_character_mut();
        campaign.add_class(class.get_name());
        self.character_classes.push(class);
    }

    fn remove_class(&mut self, class: &Class) {
        if let Some(utility_index) = self
            .utilities
            .iter()
            .position(|action| action.clone() == class.get_utility())
        {
            self.utilities.remove(utility_index);
        }
        if let Some(passive_index) = self
            .passives
            .iter()
            .position(|action| action.clone() == class.get_passive())
        {
            self.passives.remove(passive_index);
        }
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
        self.current_save
            .get_save_mut()
            .get_character_mut()
            .remove_class(class.get_name());
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
