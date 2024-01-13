use log::error;
use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::OsString;
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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Save {
    campaign_name: String,
    battle_number: Wrapping<u16>,
    round_number: u8,
    character: Character,
    used_specials: HashSet<String>,
    #[serde(default)]
    battle_power: i8,
    #[serde(default)]
    battle_defense: i8,
    #[serde(default)]
    notes: String,
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
        self.battle_power = 0;
    }

    pub fn get_round(&self) -> u8 {
        self.round_number
    }

    pub fn set_round(&mut self, round: u8) {
        self.round_number = round;
    }

    pub fn get_battle_power(&self) -> i8 {
        self.battle_power
    }

    pub fn set_battle_power(&mut self, power: i8) {
        self.battle_power = power;
    }

    pub fn get_battle_defense(&self) -> i8 {
        self.battle_defense
    }

    pub fn set_battle_defense(&mut self, defense: i8) {
        self.battle_defense = defense;
    }

    pub fn use_special<N: Into<String>>(&mut self, name: N) {
        self.used_specials.insert(name.into());
    }

    pub fn refresh_specials(&mut self) {
        self.used_specials.clear();
    }

    pub fn get_used_specials(&self) -> HashSet<String> {
        self.used_specials.clone()
    }

    pub fn get_notes_mut(&mut self) -> &mut String {
        &mut self.notes
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
            battle_power: 0,
            battle_defense: 0,
            notes: String::default(),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct SaveWithPath {
    path: Option<OsString>,
    save: Save,
}

impl SaveWithPath {
    pub fn new(save: Save) -> SaveWithPath {
        SaveWithPath { path: None, save }
    }

    pub fn from_path<P: Into<OsString>>(path: P) -> Result<SaveWithPath, SaveFromFileError> {
        let os_str_path = path.into();
        Ok(SaveWithPath {
            path: Some(os_str_path.clone()),
            save: Save::from_file(os_str_path)?,
        })
    }

    pub fn save(&self) -> Option<Result<(), SaveToFileError>> {
        self.path.as_ref().map(|path| self.save.to_file(path))
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), SaveToFileError> {
        self.save.to_file(path)
    }

    pub fn set_path<P: Into<OsString>>(&mut self, path: P) -> Option<OsString> {
        let out = self.path.clone();
        self.path = Some(path.into());
        out
    }

    pub fn get_path(&self) -> &Option<OsString> {
        &self.path
    }

    pub fn get_save(&self) -> &Save {
        &self.save
    }

    pub fn get_save_mut(&mut self) -> &mut Save {
        &mut self.save
    }

    pub fn is_dirty(&self) -> bool {
        if self.path.is_none() {
            return true;
        }

        match Save::from_file(self.path.as_ref().unwrap()) {
            Ok(old_save) => old_save != self.save,
            Err(err) => {
                error!(
                    "Unable to get save at {}: {err}",
                    self.path.as_ref().unwrap().to_string_lossy()
                );
                true
            }
        }
    }
}
