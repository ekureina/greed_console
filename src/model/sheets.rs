use serde::{Deserialize, Serialize};

use super::actions::{PrimaryAction, SecondaryAction, SpecialAction};

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
