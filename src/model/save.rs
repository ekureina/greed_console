use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

#[derive(Serialize, Deserialize, Default, Eq, PartialEq, Clone)]
pub struct Save {
    campaign_name: String,
    battle_number: u16,
    turn_number: u16,
    character: Character,
}

impl Save {
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Save, SaveFromFileError> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        Ok(from_str(&contents)?)
    }

    pub async fn to_file(&self, path: impl AsRef<Path>) -> Result<(), SaveToFileError> {
        let mut file = File::create(path).await?;
        file.write_all(to_string(&self)?.as_bytes()).await?;
        Ok(())
    }
}
