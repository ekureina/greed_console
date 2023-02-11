use serde::{Deserialize, Serialize};

use super::{
    actions::{PrimaryAction, SecondaryAction, SpecialAction},
    classes::{Class, ClassCache},
};

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq, Clone)]
pub struct Character {
    primary_actions: Vec<PrimaryAction>,
    secondary_actions: Vec<SecondaryAction>,
    special_actions: Vec<SpecialAction>,
    race: Option<String>,
    classes: Vec<String>,
}

impl Character {
    pub fn add_primary_action(&mut self, action: PrimaryAction) {
        self.primary_actions.push(action);
    }

    pub fn add_secondary_action(&mut self, action: SecondaryAction) {
        self.secondary_actions.push(action);
    }

    pub fn add_special_action(&mut self, action: SpecialAction) {
        self.special_actions.push(action);
    }

    pub fn get_primary_actions(&self) -> Vec<PrimaryAction> {
        self.primary_actions.clone()
    }

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
            .chain(self.get_primary_actions())
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
            .chain(self.get_secondary_actions())
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
            .chain(self.get_special_actions())
            .collect();

        (primary_actions, secondary_actions, special_actions)
    }

    pub fn get_secondary_actions(&self) -> Vec<SecondaryAction> {
        self.secondary_actions.clone()
    }

    pub fn get_special_actions(&self) -> Vec<SpecialAction> {
        self.special_actions.clone()
    }

    pub fn remove_primary_action(&mut self, index: usize) {
        self.primary_actions.remove(index);
    }

    pub fn remove_secondary_action(&mut self, index: usize) {
        self.secondary_actions.remove(index);
    }

    pub fn remove_special_action(&mut self, index: usize) {
        self.special_actions.remove(index);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_primary_actions() {
        let mut character = Character::default();

        let actions = vec![
            PrimaryAction::new("Test", "Lorem ipsum"),
            PrimaryAction::new("Test2", "Lorem ipsum"),
        ];

        for action in &actions {
            character.add_primary_action(action.clone());
        }

        assert_eq!(character.primary_actions, actions);
    }

    #[test]
    fn test_add_secondary_actions() {
        let mut character = Character::default();

        let actions = vec![
            SecondaryAction::new("Test", "Lorem ipsum"),
            SecondaryAction::new("Test2", "Lorem ipsum"),
        ];

        for action in &actions {
            character.add_secondary_action(action.clone());
        }

        assert_eq!(character.secondary_actions, actions);
    }

    #[test]
    fn test_add_special_actions() {
        let mut character = Character::default();

        let actions = vec![
            SpecialAction::new("Test", "Lorem ipsum"),
            SpecialAction::new("Test2", "Lorem ipsum"),
        ];

        for action in &actions {
            character.add_special_action(action.clone());
        }

        assert_eq!(character.special_actions, actions);
    }

    #[test]
    fn test_get_primary_actions() {
        let mut character = Character::default();

        let actions = vec![
            PrimaryAction::new("Test", "Lorem ipsum"),
            PrimaryAction::new("Test2", "Lorem ipsum"),
        ];

        for action in &actions {
            character.add_primary_action(action.clone());
        }

        assert_eq!(character.get_primary_actions(), actions);
    }

    #[test]
    fn test_get_secondary_actions() {
        let mut character = Character::default();

        let actions = vec![
            SecondaryAction::new("Test", "Lorem ipsum"),
            SecondaryAction::new("Test2", "Lorem ipsum"),
        ];

        for action in &actions {
            character.add_secondary_action(action.clone());
        }

        assert_eq!(character.get_secondary_actions(), actions);
    }

    #[test]
    fn test_get_special_actions() {
        let mut character = Character::default();

        let actions = vec![
            SpecialAction::new("Test", "Lorem ipsum"),
            SpecialAction::new("Test2", "Lorem ipsum"),
        ];

        for action in &actions {
            character.add_special_action(action.clone());
        }

        assert_eq!(character.get_special_actions(), actions);
    }
}
