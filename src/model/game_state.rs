#[derive(Debug)]
pub struct GameState {
    turn_num: u8,
    primary_usable: bool,
    secondary_usable: bool,
    special_usable: bool,
    inspiration_usable: bool
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            turn_num: 1,
            primary_usable: true,
            secondary_usable: true,
            special_usable: true,
            inspiration_usable: true
        }
    }
}

impl GameState {
    /**
     * Resets the GameState to setup for the next turn
     */
    fn next_turn(&mut self) {
        self.turn_num += 1;
        self.primary_usable = true;
        self.secondary_usable = true;
        self.special_usable = true;
    }

    /**
     * Resets the GameState for the next battle
     */
    fn next_battle(&mut self) {
        self.next_turn();
        self.inspiration_usable = true;
        self.turn_num = 1;
    }

    /**
     * Set GameState such that a primary action was used
     */
    fn use_primary(&mut self) {
        self.primary_usable = false;
    }

    /**
     * Set GameState such that a secondary action was used
     */
    fn use_secondary(&mut self) {
        self.secondary_usable = false;
    }

    /**
     * Set GameState such that a special action was used
     */
    fn use_special(&mut self) {
        self.special_usable = false;
    }

    /**
     * Set GameState such that inspiration was used
     */
    fn use_inspiration(&mut self) {
        self.inspiration_usable = false;
    }

    /**
     * Get the current battle's turn number
     */
    fn get_turn_num(&self) -> u8 {
        self.turn_num
    }

    /**
     * Query if it is possible to use a primary action
     */
    fn get_primary_usable(&self) -> bool {
        self.primary_usable
    }

    /**
     * Query if it is possible to use a secondary action
     */
    fn get_secondary_usable(&self) -> bool {
        self.secondary_usable
    }

    /**
     * Query if it is possible to use a special action
     */
    fn get_special_usable(&self) -> bool {
        self.special_usable
    }

    /**
     * Query if it is possible to use inspiration
     */
    fn get_inspiration_usable(&self) -> bool {
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
        assert_eq!(state.get_primary_usable(), true);
        assert_eq!(state.get_secondary_usable(), true);
        assert_eq!(state.get_special_usable(), true);
        assert_eq!(state.get_inspiration_usable(), true);
    }

    #[test]
    fn test_exclusion() {
        let mut state = GameState::default();

        assert_eq!(state.get_primary_usable(), true);
        state.use_primary();
        assert_eq!(state.get_primary_usable(), false);

        assert_eq!(state.get_secondary_usable(), true);
        state.use_secondary();
        assert_eq!(state.get_secondary_usable(), false);

        assert_eq!(state.get_special_usable(), true);
        state.use_special();
        assert_eq!(state.get_special_usable(), false);

        assert_eq!(state.get_inspiration_usable(), true);
        state.use_inspiration();
        assert_eq!(state.get_inspiration_usable(), false);
    }

    #[test]
    fn test_next_turn() {
        let mut state = GameState::default();

        state.use_primary();
        state.use_secondary();
        state.use_special();
        state.use_inspiration();

        assert_eq!(state.get_primary_usable(), false);
        assert_eq!(state.get_secondary_usable(), false);
        assert_eq!(state.get_special_usable(), false);
        assert_eq!(state.get_inspiration_usable(), false);
        assert_eq!(state.get_turn_num(), 1);

        state.next_turn();

        assert_eq!(state.get_turn_num(), 2);
        assert_eq!(state.get_primary_usable(), true);
        assert_eq!(state.get_secondary_usable(), true);
        assert_eq!(state.get_special_usable(), true);
        assert_eq!(state.get_inspiration_usable(), false);
    }

    #[test]
    fn test_next_battle() {
        let mut state = GameState::default();

        state.use_primary();
        state.use_secondary();
        state.use_special();
        state.use_inspiration();

        assert_eq!(state.get_primary_usable(), false);
        assert_eq!(state.get_secondary_usable(), false);
        assert_eq!(state.get_special_usable(), false);
        assert_eq!(state.get_inspiration_usable(), false);
        assert_eq!(state.get_turn_num(), 1);

        state.next_turn();

        assert_eq!(state.get_turn_num(), 2);
        assert_eq!(state.get_primary_usable(), true);
        assert_eq!(state.get_secondary_usable(), true);
        assert_eq!(state.get_special_usable(), true);
        assert_eq!(state.get_inspiration_usable(), false);

        state.next_battle();

        assert_eq!(state.get_turn_num(), 1);
        assert_eq!(state.get_primary_usable(), true);
        assert_eq!(state.get_secondary_usable(), true);
        assert_eq!(state.get_special_usable(), true);
        assert_eq!(state.get_inspiration_usable(), true);
    }
}

