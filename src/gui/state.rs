use crate::model::sheets::Character;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

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
    current_campaign: Option<String>,
    campaigns: HashMap<String, Character>,
}

impl AppState {
    pub fn new() -> Self {
        AppState::default()
    }

    pub fn get_current_campaign_name(&self) -> Option<String> {
        self.current_campaign.clone()
    }

    pub fn get_current_campaign(&self) -> Option<&Character> {
        if let Some(name) = &self.current_campaign {
            self.get_campaign(name)
        } else {
            None
        }
    }

    pub fn get_current_campaign_mut(&mut self) -> Option<&mut Character> {
        if let Some(name) = self.current_campaign.clone() {
            self.get_campaign_mut(name)
        } else {
            None
        }
    }

    pub fn get_campaign<K: Into<String>>(&self, campaign_name: K) -> Option<&Character> {
        self.campaigns.get(&campaign_name.into())
    }

    pub fn get_campaign_mut<K: Into<String>>(
        &mut self,
        campaign_name: K,
    ) -> Option<&mut Character> {
        self.campaigns.get_mut(&campaign_name.into())
    }

    pub fn create_campaign<K: Into<String>>(&mut self, campaign_name: K) {
        self.campaigns
            .insert(campaign_name.into(), Character::default());
    }

    pub fn set_current_campaign<K: Into<String>>(&mut self, campaign_name: K) {
        self.current_campaign = Some(campaign_name.into());
    }

    pub fn get_campaign_exists<K: Into<String>>(&self, campaign_name: K) -> bool {
        self.campaigns.contains_key(&campaign_name.into())
    }

    pub fn get_campaign_names(&self) -> Vec<String> {
        self.campaigns.keys().map(Clone::clone).collect()
    }

    pub fn remove_campaign<K: Into<String>>(&mut self, campaign_name: K) {
        self.campaigns.remove(&campaign_name.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let app_state = AppState::new();

        assert!(app_state.campaigns.is_empty());
    }
}
