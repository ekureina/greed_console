use std::collections::VecDeque;
use std::ffi::OsString;

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
    campaign_path_history: VecDeque<OsString>,
}

impl AppState {
    pub fn new() -> Self {
        AppState::default()
    }

    pub fn get_campaign_path_history(&self) -> &VecDeque<OsString> {
        &self.campaign_path_history
    }

    pub fn add_new_path_to_history<P: Into<OsString>>(&mut self, path: P) {
        self.campaign_path_history.push_front(path.into());
    }

    pub fn use_path_more_recently(&mut self, pos: usize) {
        if let Some(path) = self.campaign_path_history.remove(pos) {
            self.campaign_path_history.push_front(path)
        }
    }

    pub fn is_campaign_history_empty(&self) -> bool {
        self.campaign_path_history.is_empty()
    }

    pub fn get_most_recent_campaign_path(&self) -> Option<&OsString> {
        self.campaign_path_history.front()
    }

    pub fn remove_entry(&mut self, entry_pos: usize) {
        if entry_pos < self.campaign_path_history.len() {
            self.campaign_path_history.remove(entry_pos);
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
