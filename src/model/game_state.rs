use log::info;

use super::actions::SpecialAction;

#[derive(Debug)]
pub struct GameState {
    turn_num: u8,
    primary_actions: u8,
    secondary_actions: u8,
    special_usable: bool,
    special_actions: Vec<SpecialAction>,
    inspiration_usable: bool,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            turn_num: 1,
            primary_actions: 1,
            secondary_actions: 1,
            special_usable: true,
            special_actions: vec![],
            inspiration_usable: true,
        }
    }
}

impl GameState {
    /**
     * Resets the GameState to setup for the next turn
     */
    pub fn next_turn(&mut self) {
        self.turn_num += 1;
        self.primary_actions = 1;
        self.secondary_actions = 1;
        self.special_usable = true;
    }

    /**
     * Resets the GameState for the next battle
     */
    pub fn next_battle(&mut self) {
        self.next_turn();
        self.inspiration_usable = true;
        self.turn_num = 1;
        for action in &mut self.special_actions.iter_mut() {
            action.refresh();
        }
    }

    /**
     * Set GameState such that a primary action was used
     */
    pub fn use_primary(&mut self) {
        self.primary_actions -= 1;
    }

    /**
     * Set GameState such that a secondary action was used
     */
    pub fn use_secondary(&mut self) {
        self.secondary_actions -= 1;
    }

    /**
     * Set GameState such that a special action was used
     */
    pub fn use_special<S: Into<String> + Clone>(&mut self, name: S) {
        for action in &mut self.special_actions.iter_mut() {
            if action.is_named(name.clone()) {
                action.use_action();
                info!("Used action {}", name.into());
                break;
            }
        }
        self.special_usable = false;
    }

    /**
     * Refresh a specific Special Move
     */
    pub fn refresh_special<S: Into<String> + Clone>(&mut self, name: S) {
        for action in &mut self.special_actions.iter_mut() {
            if action.is_named(name.clone()) {
                action.refresh();
                info!("Refreshed Action {}", name.into());
                break;
            }
        }
    }

    /**
     * Get the description of the specified special move
     */
    pub fn get_special_description<S: Into<String> + Clone>(&mut self, name: &S) -> Option<String> {
        for action in &mut self.special_actions.iter() {
            if action.is_named(name.clone()) {
                return Some(action.get_description());
            }
        }
        None
    }

    /**
     * Exhaust all specials in this game
     */
    pub fn exhaust_specials(&mut self) {
        for action in &mut self.special_actions.iter_mut() {
            action.use_action();
        }
    }

    /**
     * Set GameState such that inspiration was used
     */
    pub fn use_inspiration(&mut self) {
        self.inspiration_usable = false;
    }

    /**
     * Allow extra primary action to be used this turn
     */
    pub fn extra_primary(&mut self) {
        self.primary_actions += 1;
    }

    /**
     * Allow extra secondary action to be used this turn
     */
    pub fn extra_secondary(&mut self) {
        self.secondary_actions += 1;
    }

    /**
     * Register a new special action
     */
    pub fn new_special<N: Into<String>, D: Into<String>>(&mut self, name: N, description: D) {
        self.special_actions
            .push(SpecialAction::new(name, description));
    }

    /**
     * Get the current battle's turn number
     */
    pub fn get_turn_num(&self) -> u8 {
        self.turn_num
    }

    /**
     * Query if it is possible to use a primary action
     */
    pub fn get_primary_usable(&self) -> bool {
        self.primary_actions > 0
    }

    /**
     * Query the number of primary actions available
     */
    pub fn get_primary_actions(&self) -> u8 {
        self.primary_actions
    }

    /**
     * Query the number of primary actions available
     */
    pub fn get_secondary_actions(&self) -> u8 {
        self.secondary_actions
    }

    /**
     * Query if it is possible to use a secondary action
     */
    pub fn get_secondary_usable(&self) -> bool {
        self.secondary_actions > 0
    }

    /**
     * Query if it is possible to use a special action
     */
    pub fn get_any_special_usable(&self) -> bool {
        self.special_usable
            && !self.special_actions.is_empty()
            && self.special_actions.iter().any(SpecialAction::is_usable)
    }

    /**
     * Query if the given special action is a contained special action
     */
    pub fn get_special_action_exists<S: Into<String> + Clone>(&self, name: &S) -> bool {
        self.special_actions
            .iter()
            .any(|action| action.is_named(name.clone()))
    }

    /**
     * Query if the given special action is usable
     */
    pub fn get_special_action_usable<S: Into<String> + Clone>(&self, name: &S) -> bool {
        self.special_actions
            .iter()
            .any(|action| action.is_named(name.clone()) && action.is_usable())
    }

    /**
     * Get the Vec of Special Actions as a reference
     */
    pub fn get_special_actions(&self) -> &Vec<SpecialAction> {
        &self.special_actions
    }

    /**
     * Query if it is possible to use inspiration
     */
    pub fn get_inspiration_usable(&self) -> bool {
        self.inspiration_usable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let state = GameState::default();

        assert_eq!(state.get_turn_num(), 1);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(state.get_inspiration_usable());
    }

    #[test]
    fn test_exclusion() {
        let mut state = GameState::default();
        state.new_special("Test", "Lorem ipsum");

        assert!(state.get_primary_usable());
        state.use_primary();
        assert!(!state.get_primary_usable());

        assert!(state.get_secondary_usable());
        state.use_secondary();
        assert!(!state.get_secondary_usable());

        assert!(state.get_any_special_usable());
        state.use_special("Test");
        assert!(!state.get_any_special_usable());

        assert!(state.get_inspiration_usable());
        state.use_inspiration();
        assert!(!state.get_inspiration_usable());
    }

    #[test]
    fn test_next_turn() {
        let mut state = GameState::default();
        state.new_special("Test", "Lorem ipsum");

        state.use_primary();
        state.use_secondary();
        state.use_special("Test");
        state.use_inspiration();

        assert!(!state.get_primary_usable());
        assert!(!state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(!state.get_inspiration_usable());
        assert_eq!(state.get_turn_num(), 1);

        state.next_turn();

        assert_eq!(state.get_turn_num(), 2);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(!state.get_inspiration_usable());

        state.new_special("Test2", "Lorem ipsum");
        assert!(state.get_any_special_usable());
    }

    #[test]
    fn test_next_battle() {
        let mut state = GameState::default();
        state.new_special("Test", "Lorem ipsum");

        state.use_primary();
        state.use_secondary();
        state.use_special("Test");
        state.use_inspiration();

        assert!(!state.get_primary_usable());
        assert!(!state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(!state.get_inspiration_usable());
        assert_eq!(state.get_turn_num(), 1);

        state.next_turn();

        assert_eq!(state.get_turn_num(), 2);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(!state.get_inspiration_usable());

        state.new_special("Test2", "Lorem ipsum");
        assert!(state.get_any_special_usable());

        state.next_battle();

        assert_eq!(state.get_turn_num(), 1);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(state.get_any_special_usable());
        assert!(state.get_inspiration_usable());
    }

    #[test]
    fn test_primary_extras() {
        let mut state = GameState::default();

        state.use_primary();
        assert!(!state.get_primary_usable());

        state.extra_primary();
        assert!(state.get_primary_usable());
    }

    #[test]
    fn test_primary_multiple_extras() {
        let mut state = GameState::default();

        state.extra_primary();
        assert_eq!(state.get_primary_actions(), 2);
    }

    #[test]
    fn test_secondary_multiple_extras() {
        let mut state = GameState::default();

        state.extra_secondary();
        assert_eq!(state.get_secondary_actions(), 2);
    }

    #[test]
    fn test_secondary_extras() {
        let mut state = GameState::default();

        state.use_secondary();
        assert!(!state.get_secondary_usable());

        state.extra_secondary();
        assert!(state.get_secondary_usable());
    }

    #[test]
    fn specials() {
        let mut state = GameState::default();

        assert_eq!(state.get_special_actions().len(), 0);
        assert!(!state.get_any_special_usable());

        state.new_special("Test", "Lorem ipsum");

        assert_eq!(state.get_special_actions().len(), 1);
        assert!(state.get_special_action_usable(&"Test"));
        assert!(state.get_any_special_usable());

        state.use_special("Test");

        assert_eq!(state.get_special_actions().len(), 1);
        assert!(!state.get_special_action_usable(&"Test"));
        assert!(!state.get_any_special_usable());

        state.next_turn();

        assert_eq!(state.get_special_actions().len(), 1);
        assert!(!state.get_special_action_usable(&"Test"));
        assert!(!state.get_any_special_usable());

        state.new_special("Test2", "Lorem ipsum");

        assert_eq!(state.get_special_actions().len(), 2);
        assert!(state.get_special_action_usable(&"Test2"));
        assert!(state.get_any_special_usable());

        state.use_special("Test2");

        assert_eq!(state.get_special_actions().len(), 2);
        assert!(!state.get_special_action_usable(&"Test2"));
        assert!(!state.get_any_special_usable());

        state.next_turn();

        assert_eq!(state.get_special_actions().len(), 2);
        assert!(!state.get_special_action_usable(&"Test2"));
        assert!(!state.get_any_special_usable());
    }

    #[test]
    fn test_exhaust_specials() {
        let mut state = GameState::default();

        for i in 0..500 {
            state.new_special(format!("Test{}", i), "Lorem ipsum");
        }

        assert_eq!(state.special_actions.len(), 500);
        assert!(state.get_any_special_usable());

        state.exhaust_specials();

        assert_eq!(state.special_actions.len(), 500);
        assert!(!state.get_any_special_usable());
    }

    #[test]
    fn test_refreshable_specials() {
        let mut state = GameState::default();
        state.new_special("Test", "Lorem ipsum");

        state.use_special("Test");

        assert!(!state.get_special_action_usable(&"Test"));

        state.next_turn();

        assert!(!state.get_special_action_usable(&"Test"));

        state.refresh_special("Test");

        assert!(state.get_special_action_usable(&"Test"));
    }

    #[test]
    fn test_get_special_description() {
        let mut state = GameState::default();
        state.new_special("Test", "Lorem ipsum");

        assert_eq!(
            state.get_special_description(&"Test"),
            Some("Lorem ipsum".to_owned())
        );
        assert_eq!(state.get_special_description(&"Test2"), None);
    }
}
