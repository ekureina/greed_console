#[derive(Debug)]
pub struct GameState {
    turn_num: u8,
    primary_actions: u8,
    secondary_actions: u8,
    special_usable: bool,
    inspiration_usable: bool,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            turn_num: 1,
            primary_actions: 1,
            secondary_actions: 1,
            special_usable: true,
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
    pub fn use_special(&mut self) {
        self.special_usable = false;
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
    pub fn get_special_usable(&self) -> bool {
        self.special_usable
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
    use crate::model::game_state::GameState;

    #[test]
    fn test_default() {
        let state = GameState::default();

        assert_eq!(state.get_turn_num(), 1);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(state.get_special_usable());
        assert!(state.get_inspiration_usable());
    }

    #[test]
    fn test_exclusion() {
        let mut state = GameState::default();

        assert!(state.get_primary_usable());
        state.use_primary();
        assert!(!state.get_primary_usable());

        assert!(state.get_secondary_usable());
        state.use_secondary();
        assert!(!state.get_secondary_usable());

        assert!(state.get_special_usable());
        state.use_special();
        assert!(!state.get_special_usable());

        assert!(state.get_inspiration_usable());
        state.use_inspiration();
        assert!(!state.get_inspiration_usable());
    }

    #[test]
    fn test_next_turn() {
        let mut state = GameState::default();

        state.use_primary();
        state.use_secondary();
        state.use_special();
        state.use_inspiration();

        assert!(!state.get_primary_usable());
        assert!(!state.get_secondary_usable());
        assert!(!state.get_special_usable());
        assert!(!state.get_inspiration_usable());
        assert_eq!(state.get_turn_num(), 1);

        state.next_turn();

        assert_eq!(state.get_turn_num(), 2);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(state.get_special_usable());
        assert!(!state.get_inspiration_usable());
    }

    #[test]
    fn test_next_battle() {
        let mut state = GameState::default();

        state.use_primary();
        state.use_secondary();
        state.use_special();
        state.use_inspiration();

        assert!(!state.get_primary_usable());
        assert!(!state.get_secondary_usable());
        assert!(!state.get_special_usable());
        assert!(!state.get_inspiration_usable());
        assert_eq!(state.get_turn_num(), 1);

        state.next_turn();

        assert_eq!(state.get_turn_num(), 2);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(state.get_special_usable());
        assert!(!state.get_inspiration_usable());

        state.next_battle();

        assert_eq!(state.get_turn_num(), 1);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(state.get_special_usable());
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
}
