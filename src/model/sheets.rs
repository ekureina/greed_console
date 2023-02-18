use serde::{Deserialize, Serialize};

use super::{
    actions::{PrimaryAction, SecondaryAction, SpecialAction},
    classes::{Class, ClassCache},
};

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq, Clone)]
pub struct Character {
    race: Option<String>,
    classes: Vec<String>,
}

impl Character {
    pub fn get_all_actions(
        &self,
        class_cache: &ClassCache,
    ) -> (Vec<PrimaryAction>, Vec<SecondaryAction>, Vec<SpecialAction>) {
        let character_race = self.get_race().and_then(|race_name| {
            class_cache
                .get_races()
                .iter()
                .find(|race| race.get_name() == race_name.clone())
                .cloned()
        });
        let character_classes = self
            .get_classes()
            .iter()
            .filter_map(|class_name| {
                class_cache
                    .get_classes()
                    .iter()
                    .find(|class| class.get_name() == class_name.clone())
                    .cloned()
            })
            .collect::<Vec<Class>>();

        let primary_actions = character_race
            .clone()
            .map_or_else(Vec::new, |race| {
                if race.get_name() == "Human" {
                    vec![]
                } else {
                    vec![race.get_primary_action()]
                }
            })
            .into_iter()
            .chain(character_classes.iter().map(Class::get_primary_action))
            .collect();

        let secondary_actions = character_race
            .clone()
            .map_or_else(Vec::new, |race| {
                if race.get_name() == "Human" {
                    vec![]
                } else {
                    vec![race.get_secondary_action()]
                }
            })
            .into_iter()
            .chain(character_classes.iter().map(Class::get_secondary_action))
            .collect();

        let special_actions = character_race
            .map_or_else(Vec::new, |race| {
                if race.get_name() == "Human" {
                    vec![]
                } else {
                    vec![race.get_special_action()]
                }
            })
            .into_iter()
            .chain(character_classes.iter().map(Class::get_special_action))
            .collect();

        (primary_actions, secondary_actions, special_actions)
    }

    pub fn get_race(&self) -> Option<String> {
        self.race.clone()
    }

    pub fn replace_race(&mut self, new_race: Option<String>) {
        self.race = new_race;
    }

    pub fn get_classes(&self) -> Vec<String> {
        self.classes.clone()
    }

    pub fn add_class<N: Into<String>>(&mut self, class_name: N) {
        self.classes.push(class_name.into());
    }

    pub fn remove_class<N: Into<String>>(&mut self, class_name: N) {
        let class_name_string = class_name.into();

        if let Some(class_position) = self
            .classes
            .iter()
            .position(|class| class_name_string == class.clone())
        {
            self.classes.remove(class_position);
        }
    }
}
