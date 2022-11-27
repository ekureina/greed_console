use serde::{Deserialize, Serialize};

use super::actions::SpecialAction;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Character {
    special_actions: Vec<SpecialAction>,
}

impl Character {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_special_action(&mut self, action: &SpecialAction) {
        self.special_actions.push(action.clone());
    }

    pub fn add_special_actions<V: Into<Vec<SpecialAction>>>(&mut self, actions: V) {
        self.special_actions.extend(actions.into());
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
    fn test_add_special_action() {
        let mut character = Character::new();

        let action = SpecialAction::new("Test", "Lorem ipsum");

        character.add_special_action(&action);

        assert!(!character.special_actions.is_empty());
        assert_eq!(character.special_actions[0], action);
    }

    #[test]
    fn test_add_special_actions() {
        let mut character = Character::new();

        let actions = vec![
            SpecialAction::new("Test", "Lorem ipsum"),
            SpecialAction::new("Test2", "Lorem ipsum"),
        ];

        character.add_special_actions(actions.clone());

        assert_eq!(character.special_actions, actions);
    }

    #[test]
    fn test_get_special_actions() {
        let mut character = Character::new();

        let actions = vec![
            SpecialAction::new("Test", "Lorem ipsum"),
            SpecialAction::new("Test2", "Lorem ipsum"),
        ];

        character.add_special_actions(actions.clone());

        assert_eq!(character.get_special_actions(), actions);
    }
}
