use serde::{Deserialize, Serialize};

use super::{
    actions::{PrimaryAction, SecondaryAction, SpecialAction},
    classes::{Class, ClassCache},
};

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq, Clone)]
pub struct Character {
    origin: Option<String>,
    classes: Vec<String>,
}

impl Character {
    pub fn get_all_actions(
        &self,
        class_cache: &ClassCache,
    ) -> (Vec<PrimaryAction>, Vec<SecondaryAction>, Vec<SpecialAction>) {
        let character_origin = self.get_origin().and_then(|origin_name| {
            class_cache
                .get_origins()
                .iter()
                .find(|origin| origin.get_name() == origin_name.clone())
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

        let primary_actions = character_origin
            .clone()
            .map_or_else(Vec::new, |origin| {
                if origin.get_name() == "Human" {
                    vec![]
                } else {
                    vec![origin.get_primary_action()]
                }
            })
            .into_iter()
            .chain(character_classes.iter().map(Class::get_primary_action))
            .collect();

        let secondary_actions = character_origin
            .clone()
            .map_or_else(Vec::new, |origin| {
                if origin.get_name() == "Human" {
                    vec![]
                } else {
                    vec![origin.get_secondary_action()]
                }
            })
            .into_iter()
            .chain(character_classes.iter().map(Class::get_secondary_action))
            .collect();

        let special_actions = character_origin
            .map_or_else(Vec::new, |origin| {
                if origin.get_name() == "Human" {
                    vec![]
                } else {
                    vec![origin.get_special_action()]
                }
            })
            .into_iter()
            .chain(character_classes.iter().map(Class::get_special_action))
            .collect();

        (primary_actions, secondary_actions, special_actions)
    }

    pub fn get_origin(&self) -> Option<String> {
        self.origin.clone()
    }

    pub fn replace_origin(&mut self, new_origin: Option<String>) {
        self.origin = new_origin;
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
