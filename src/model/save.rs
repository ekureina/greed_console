use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::num::Wrapping;
use std::path::Path;
use thiserror::Error;

use super::sheets::Character;

#[derive(Error, Debug)]
pub enum SaveFromFileError {
    #[error("Error when serializing save: {0}")]
    SerdeError(#[from] ron::error::SpannedError),
    #[error("Error when reading from file: {0}")]
    ReadError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum SaveToFileError {
    #[error("Error when deserializing save: {0}")]
    SerdeError(#[from] ron::error::Error),
    #[error("Error when writing to file: {0}")]
    WriteError(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Save {
    campaign_name: String,
    battle_number: Wrapping<u16>,
    round_number: u8,
    character: Character,
    used_specials: HashSet<String>,
}

impl Save {
    pub fn new<N: Into<String>>(name: N) -> Save {
        Save {
            campaign_name: name.into(),
            ..Default::default()
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Save, SaveFromFileError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(from_str(&contents)?)
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), SaveToFileError> {
        let mut file = File::create(path)?;
        file.write_all(to_string(&self)?.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    pub fn get_character(&self) -> Character {
        self.character.clone()
    }

    pub fn get_character_mut(&mut self) -> &mut Character {
        &mut self.character
    }

    pub fn get_campaign_name(&self) -> String {
        self.campaign_name.clone()
    }

    pub fn get_battle(&self) -> u16 {
        self.battle_number.0
    }

    pub fn inc_battle(&mut self) {
        self.battle_number += 1;
        if self.battle_number == Wrapping(0) {
            self.battle_number = Wrapping(1);
        }
    }

    pub fn get_round(&self) -> u8 {
        self.round_number
    }

    pub fn set_round(&mut self, round: u8) {
        self.round_number = round;
    }

    pub fn use_special<N: Into<String>>(&mut self, name: N) {
        self.used_specials.insert(name.into());
    }

    pub fn refresh_special<N: Into<String>>(&mut self, name: N) {
        self.used_specials.remove(&name.into());
    }

    pub fn refresh_specials(&mut self) {
        self.used_specials.clear();
    }

    pub fn get_used_specials(&self) -> HashSet<String> {
        self.used_specials.clone()
    }
}

impl Default for Save {
    fn default() -> Save {
        Save {
            campaign_name: String::default(),
            battle_number: Wrapping(1),
            round_number: 1,
            character: Character::default(),
            used_specials: HashSet::default(),
        }
    }
}
