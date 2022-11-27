use serde::{Deserialize, Serialize};

use super::actions::{PrimaryAction, SecondaryAction, SpecialAction};

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq, Clone)]
pub struct Character {
    primary_actions: Vec<PrimaryAction>,
    secondary_actions: Vec<SecondaryAction>,
    special_actions: Vec<SpecialAction>,
}

impl Character {
    pub fn new() -> Self {
        Self::default()
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let character = Character::new();

        assert!(character.special_actions.is_empty());
    }

    #[test]
    fn test_add_primary_actions() {
        let mut character = Character::new();

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
        let mut character = Character::new();

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
        let mut character = Character::new();

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
        let mut character = Character::new();

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
        let mut character = Character::new();

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
        let mut character = Character::new();

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
