use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppState {
    campaign_path_history: VecDeque<String>,
}

impl AppState {
    pub fn new() -> Self {
        AppState::default()
    }

    pub fn get_campaign_path_history(&self) -> &VecDeque<String> {
        &self.campaign_path_history
    }

    pub fn add_new_path_to_history<P: Into<String>>(&mut self, path: P) {
        self.campaign_path_history.push_front(path.into());
    }

    pub fn use_path_more_recently(&mut self, pos: usize) {
        if pos < self.campaign_path_history.len() {
            self.campaign_path_history.swap(pos, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let app_state = AppState::new();

        assert!(app_state.campaign_path_history.is_empty());
    }
}
