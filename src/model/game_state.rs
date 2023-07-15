use super::actions::SpecialAction;

use log::info;

use std::fmt::{self, Formatter, Result};

/*
 * A console and digital character sheet for campaigns under the greed ruleset.
 * Copyright (C) 2023 Claire Moore
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#[derive(Debug)]
pub struct GameState {
    round_num: u8,
    turn_side: TurnSide,
    primary_actions: u8,
    secondary_actions: u8,
    special_usable: bool,
    special_actions: Vec<SpecialAction>,
    inspiration_usable: bool,
    power: Stat,
    defense: Stat,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            round_num: 1,
            turn_side: TurnSide::PlayerSide,
            primary_actions: 1,
            secondary_actions: 1,
            special_usable: true,
            special_actions: vec![],
            inspiration_usable: true,
            power: Stat::with_base(0),
            defense: Stat::with_base(0),
        }
    }
}

impl GameState {
    /**
     * Sets up for the next side to attack in the next turn
     */
    pub fn next_turn(&mut self) {
        match self.turn_side {
            TurnSide::OpposingSide => {
                self.next_round();
            }
            TurnSide::PlayerSide => {
                self.turn_side = TurnSide::OpposingSide;
                self.power.next_turn();
                self.defense.next_turn();
            }
        }
    }

    fn next_round(&mut self) {
        self.round_num += 1;
        self.primary_actions = 1;
        self.secondary_actions = 1;
        self.special_usable = true;
        self.turn_side = TurnSide::PlayerSide;
        self.power.next_round();
        self.defense.next_round();
    }

    pub fn set_round<R: Into<u8>>(&mut self, round: R) {
        self.next_battle();
        self.round_num = round.into();
        self.turn_side = TurnSide::PlayerSide;
    }

    /**
     * Resets the GameState for the next battle
     */
    pub fn next_battle(&mut self) {
        self.next_round();
        self.inspiration_usable = true;
        self.round_num = 1;
        for action in &mut self.special_actions.iter_mut() {
            action.refresh();
        }
        self.power.next_battle();
        self.defense.next_battle();
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
    pub fn use_special<'a, S: Into<&'a str>>(&mut self, name: S) {
        let name_ref = name.into();
        for action in &mut self.special_actions.iter_mut() {
            if action.is_named(name_ref) {
                action.use_action();
                info!("Used action {}", name_ref);
                break;
            }
        }
        self.special_usable = false;
    }

    pub fn use_repeatable_special(&mut self) {
        self.special_usable = false;
    }

    /**
     * Set GameState such that inspiration was used
     */
    pub fn use_inspiration(&mut self) {
        self.inspiration_usable = false;
    }

    pub fn refresh_inspiration(&mut self) {
        self.inspiration_usable = true;
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

    pub fn extra_special(&mut self) {
        self.special_usable = true;
    }

    pub fn push_special(&mut self, special_action: SpecialAction) {
        self.special_actions.push(special_action);
    }

    /**
     * Get the current battle's turn number
     */
    pub fn get_round_num(&self) -> u8 {
        self.round_num
    }

    /**
     * Gets the current side for the turn
     */
    pub fn get_turn_side(&self) -> TurnSide {
        self.turn_side
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
     * Get the Vec of Special Actions as a reference
     */
    pub fn get_special_actions(&self) -> &Vec<SpecialAction> {
        &self.special_actions
    }

    pub fn remove_special_action(&mut self, index: usize) {
        self.special_actions.remove(index);
    }

    /**
     * Query if it is possible to use inspiration
     */
    pub fn get_inspiration_usable(&self) -> bool {
        self.inspiration_usable
    }

    pub fn get_power(&self) -> i8 {
        self.power.current_value()
    }

    pub fn get_battle_power(&self) -> i8 {
        self.power.battle_mod
    }

    pub fn change_power_for_turn(&mut self, delta: i8) {
        self.power.modify_turn(delta);
    }

    pub fn change_power_for_round(&mut self, delta: i8) {
        self.power.modify_round(delta);
    }

    pub fn change_power_for_battle(&mut self, delta: i8) {
        self.power.modify_battle(delta);
    }

    pub fn get_defense(&self) -> i8 {
        self.defense.current_value()
    }

    pub fn get_battle_defense(&self) -> i8 {
        self.defense.battle_mod
    }

    pub fn change_defense_for_turn(&mut self, delta: i8) {
        self.defense.modify_turn(delta);
    }

    pub fn change_defense_for_round(&mut self, delta: i8) {
        self.defense.modify_round(delta);
    }

    pub fn change_defense_for_battle(&mut self, delta: i8) {
        self.defense.modify_battle(delta);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TurnSide {
    PlayerSide,
    OpposingSide,
}

impl fmt::Display for TurnSide {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TurnSide::PlayerSide => write!(f, "Player"),
            TurnSide::OpposingSide => write!(f, "Opposing"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Stat {
    base: i8,
    turn_mod: i8,
    round_mod: i8,
    battle_mod: i8,
}

impl Stat {
    pub fn with_base(base: i8) -> Stat {
        Stat {
            base,
            turn_mod: 0,
            round_mod: 0,
            battle_mod: 0,
        }
    }

    pub fn current_value(self) -> i8 {
        self.base + self.turn_mod + self.round_mod + self.battle_mod
    }

    pub fn modify_turn(&mut self, delta: i8) {
        self.turn_mod += delta;
    }

    pub fn modify_round(&mut self, delta: i8) {
        self.round_mod += delta;
    }

    pub fn modify_battle(&mut self, delta: i8) {
        self.battle_mod += delta;
    }

    pub fn next_turn(&mut self) {
        self.turn_mod = 0;
    }

    pub fn next_round(&mut self) {
        self.next_turn();
        self.round_mod = 0;
    }

    pub fn next_battle(&mut self) {
        self.next_round();
        self.battle_mod = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let state = GameState::default();

        assert_eq!(state.get_round_num(), 1);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(state.get_inspiration_usable());
    }

    #[test]
    fn test_exclusion() {
        let mut state = GameState::default();
        state.push_special(SpecialAction::new("Test", "Lorem ipsum"));

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
        state.push_special(SpecialAction::new("Test", "Lorem ipsum"));

        state.use_primary();
        state.use_secondary();
        state.use_special("Test");
        state.use_inspiration();

        assert!(!state.get_primary_usable());
        assert!(!state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(!state.get_inspiration_usable());
        assert_eq!(state.get_round_num(), 1);

        state.next_turn();
        state.next_turn();

        assert_eq!(state.get_round_num(), 2);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(!state.get_inspiration_usable());

        state.push_special(SpecialAction::new("Test2", "Lorem ipsum"));
        assert!(state.get_any_special_usable());
    }

    #[test]
    fn test_next_battle() {
        let mut state = GameState::default();
        state.push_special(SpecialAction::new("Test", "Lorem ipsum"));

        state.use_primary();
        state.use_secondary();
        state.use_special("Test");
        state.use_inspiration();

        assert!(!state.get_primary_usable());
        assert!(!state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(!state.get_inspiration_usable());
        assert_eq!(state.get_round_num(), 1);

        state.next_turn();
        state.next_turn();

        assert_eq!(state.get_round_num(), 2);
        assert!(state.get_primary_usable());
        assert!(state.get_secondary_usable());
        assert!(!state.get_any_special_usable());
        assert!(!state.get_inspiration_usable());

        state.push_special(SpecialAction::new("Test2", "Lorem ipsum"));
        assert!(state.get_any_special_usable());

        state.next_battle();

        assert_eq!(state.get_round_num(), 1);
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

        state.push_special(SpecialAction::new("Test", "Lorem ipsum"));

        assert_eq!(state.get_special_actions().len(), 1);
        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test");
        assert!(special.is_some());
        assert!(special.unwrap().is_usable());
        assert!(state.get_any_special_usable());

        state.use_special("Test");

        assert_eq!(state.get_special_actions().len(), 1);
        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test");
        assert!(special.is_some());
        assert!(!special.unwrap().is_usable());
        assert!(!state.get_any_special_usable());

        state.next_turn();
        state.next_turn();

        assert_eq!(state.get_special_actions().len(), 1);
        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test");
        assert!(special.is_some());
        assert!(!special.unwrap().is_usable());
        assert!(!state.get_any_special_usable());

        state.push_special(SpecialAction::new("Test2", "Lorem ipsum"));

        assert_eq!(state.get_special_actions().len(), 2);
        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test2");
        assert!(special.is_some());
        assert!(special.unwrap().is_usable());
        assert!(state.get_any_special_usable());

        state.use_special("Test2");

        assert_eq!(state.get_special_actions().len(), 2);
        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test2");
        assert!(special.is_some());
        assert!(!special.unwrap().is_usable());
        assert!(!state.get_any_special_usable());

        state.next_turn();
        state.next_turn();

        assert_eq!(state.get_special_actions().len(), 2);
        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test2");
        assert!(special.is_some());
        assert!(!special.unwrap().is_usable());
        assert!(!state.get_any_special_usable());
    }

    #[test]
    fn test_exhaust_specials() {
        let mut state = GameState::default();

        for i in 0..500 {
            state.push_special(SpecialAction::new(format!("Test{i}"), "Lorem ipsum"));
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
        state.push_special(SpecialAction::new("Test", "Lorem ipsum"));

        state.use_special("Test");

        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test");
        assert!(special.is_some());
        assert!(!special.unwrap().is_usable());

        state.next_turn();

        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test");
        assert!(special.is_some());
        assert!(!special.unwrap().is_usable());

        state.refresh_special("Test");

        let special = state
            .get_special_actions()
            .iter()
            .find(|action| action.get_name() == "Test");
        assert!(special.is_some());
        assert!(special.unwrap().is_usable());
    }
}
