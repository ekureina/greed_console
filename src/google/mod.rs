use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};
use crate::model::classes::{
    AndClassRequirement, Class, ClassCache, ClassPassive, ClassRequirement, ClassUtility,
    LevelPrefixRequirement, SuperClassRequirement,
};
use crate::util::from_roman;

use google_drive3::client::NoToken;
use google_drive3::hyper::body::to_bytes;
use google_drive3::hyper::client::HttpConnector;
use google_drive3::hyper_rustls::HttpsConnector;
use google_drive3::DriveHub;
use google_drive3::{hyper, hyper_rustls};
use log::info;
use thiserror::Error;

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

static API_KEY: &str = "AIzaSyBRY2-Rb0_OBRRmiq4duFOksNh9_0FQPGQ";
static GREED_RULES_DOC_ID: &str = "1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4";
static RULES_EXPORT_FORMAT: &str = "text/plain";

#[allow(let_underscore_drop, clippy::too_many_lines)]
fn get_class(
    first_line: &str,
    mut paragraphs: impl Iterator<Item = String>,
) -> Result<Class, GetOriginsAndClassesError> {
    let class_components = first_line.split('(').collect::<Vec<&str>>();
    let class_name = class_components[0].trim().to_owned();
    let class_level = if class_components.len() > 1 {
        from_roman(class_components[1].split(')').collect::<Vec<&str>>()[0].trim())
    } else {
        None
    };

    let class_requirements = if first_line.contains("Req:") {
        Some(determine_class_requirements(
            first_line.split("Req:").collect::<Vec<&str>>()[1].trim(),
        ))
    } else {
        None
    };

    paragraphs
        .next()
        .ok_or(GetOriginsAndClassesError::FormatChange)?;
    let utility_data = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Passive"))
        .collect::<Vec<String>>();
    let mut utility_description = String::new();
    let mut utility_names = vec![];
    let mut utility_descriptions = vec![];
    for line in utility_data {
        if line.starts_with('*') {
            utility_names.push(line.trim_start_matches('*').trim().to_owned());
            if !utility_description.is_empty() {
                utility_descriptions.push(utility_description.trim_end().to_owned());
                utility_description.clear();
            }
        } else {
            if !utility_description.is_empty() {
                utility_description.push('\n');
            }
            utility_description.push_str(&line);
        }
    }
    if !utility_description.is_empty() {
        utility_descriptions.push(utility_description.trim_end().to_owned());
    }
    let utilities = utility_names
        .iter()
        .zip(utility_descriptions)
        .map(|(name, description)| ClassUtility::new(name, description));
    let passive_data = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Primary"))
        .collect::<Vec<String>>();
    let mut passive_description = String::new();
    let mut passive_names = vec![];
    let mut passive_descriptions = vec![];
    for line in passive_data {
        if line.starts_with('*') {
            passive_names.push(line.trim_start_matches('*').trim().to_owned());
            if !passive_description.is_empty() {
                passive_descriptions.push(passive_description.trim_end().to_owned());
                passive_description.clear();
            }
        } else {
            if !passive_description.is_empty() {
                passive_description.push('\n');
            }
            passive_description.push_str(&line);
        }
    }
    if !passive_description.is_empty() {
        passive_descriptions.push(passive_description.trim_end().to_owned());
    }
    let passives = passive_names
        .iter()
        .zip(passive_descriptions)
        .map(|(name, description)| ClassPassive::new(name, description));
    let primary_name = paragraphs
        .next()
        .ok_or(GetOriginsAndClassesError::ClassParse)?
        .trim_start_matches('*')
        .trim()
        .to_owned();
    let primary_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Secondary"))
        .collect::<Vec<String>>()
        .join("\n")
        .trim_end()
        .to_owned();
    let secondary_name = paragraphs
        .next()
        .ok_or(GetOriginsAndClassesError::ClassParse)?
        .trim_start_matches('*')
        .trim()
        .to_owned();
    let secondary_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Special"))
        .collect::<Vec<String>>()
        .join("\n")
        .trim_end()
        .to_owned();
    let special_name = paragraphs
        .next()
        .ok_or(GetOriginsAndClassesError::ClassParse)?
        .trim_start_matches('*')
        .trim()
        .to_owned();
    let special_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Subclasses"))
        .collect::<Vec<String>>()
        .join("\n")
        .trim_end()
        .to_owned();
    Ok(Class::new(
        class_name,
        class_level,
        utilities.collect(),
        passives.collect(),
        PrimaryAction::new(primary_name, primary_description),
        SecondaryAction::new(secondary_name, secondary_description),
        SpecialAction::new(special_name, special_description),
        class_requirements,
    ))
}

fn determine_class_requirements(line: &str) -> Box<dyn ClassRequirement> {
    if let Some(index) = line.find(',') {
        let (first, second) = line.split_at(index);
        let first_requirement = determine_class_requirements(first);
        let second_requirement =
            determine_class_requirements(second.get(1..).unwrap().trim_start());
        Box::new(AndClassRequirement::new(
            first_requirement,
            second_requirement,
        ))
    } else if let Some(index) = line.find("Any Level") {
        let info = line.get((index + "Any Level".len())..).unwrap().trim();
        let level = from_roman(info.split_whitespace().collect::<Vec<&str>>()[0]);
        let start_name = info.split('"').collect::<Vec<&str>>()[1];
        Box::new(LevelPrefixRequirement::new(level.unwrap_or(0), start_name))
    } else {
        Box::new(SuperClassRequirement::new(line))
    }
}

#[allow(let_underscore_drop, clippy::too_many_lines)]
fn get_origin(
    first_line: &str,
    mut paragraphs: impl Iterator<Item = String>,
) -> Result<Class, GetOriginsAndClassesError> {
    let origin_name = first_line.split_whitespace().collect::<Vec<&str>>()[0].to_string();
    if origin_name == "Human" {
        Ok(Class::new(
            String::from("Human"),
            None,
            vec![ClassUtility::new(String::new(), String::new())],
            vec![ClassPassive::new(String::new(), String::new())],
            PrimaryAction::new(String::new(), String::new()),
            SecondaryAction::new(String::new(), String::new()),
            SpecialAction::new(String::new(), String::new()),
            None,
        ))
    } else {
        paragraphs
            .next()
            .ok_or(GetOriginsAndClassesError::OriginParse)?;
        let utility_data = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Passive"))
            .collect::<Vec<String>>();

        let mut utility_description = String::new();
        let mut utility_names = vec![];
        let mut utility_descriptions = vec![];
        for line in utility_data {
            if line.starts_with('*') {
                utility_names.push(line.trim_start_matches('*').trim().to_owned());
                if !utility_description.is_empty() {
                    utility_descriptions.push(utility_description.trim_end().to_owned());
                    utility_description.clear();
                }
            } else {
                if !utility_description.is_empty() {
                    utility_description.push('\n');
                }
                utility_description.push_str(&line);
            }
        }
        if !utility_description.is_empty() {
            utility_descriptions.push(utility_description.trim_end().to_owned());
        }
        let utilities = utility_names
            .iter()
            .zip(utility_descriptions)
            .map(|(name, description)| ClassUtility::new(name, description));
        let passive_data = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Primary"))
            .collect::<Vec<String>>();
        let mut passive_description = String::new();
        let mut passive_names = vec![];
        let mut passive_descriptions = vec![];
        for line in passive_data {
            if line.starts_with('*') {
                passive_names.push(line.trim_start_matches('*').trim().to_owned());
                if !passive_description.is_empty() {
                    passive_descriptions.push(passive_description.trim_end().to_owned());
                    passive_description.clear();
                }
            } else {
                if !passive_description.is_empty() {
                    passive_description.push('\n');
                }
                passive_description.push_str(&line);
            }
        }
        if !passive_description.is_empty() {
            passive_descriptions.push(passive_description.trim_end().to_owned());
        }
        let passives = passive_names
            .iter()
            .zip(passive_descriptions)
            .map(|(name, description)| ClassPassive::new(name, description));
        let primary_name = paragraphs
            .next()
            .ok_or(GetOriginsAndClassesError::OriginParse)?
            .trim_start_matches('*')
            .trim()
            .to_owned();
        let primary_description = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Secondary"))
            .collect::<Vec<String>>()
            .join("\n")
            .trim_end()
            .to_owned();
        let secondary_name = paragraphs
            .next()
            .ok_or(GetOriginsAndClassesError::OriginParse)?
            .trim_start_matches('*')
            .trim()
            .to_owned();
        let secondary_description = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Special"))
            .collect::<Vec<String>>()
            .join("\n")
            .trim_end()
            .to_owned();
        let special_name = paragraphs
            .next()
            .ok_or(GetOriginsAndClassesError::OriginParse)?
            .trim_start_matches('*')
            .trim()
            .to_owned();
        let special_description = paragraphs
            .by_ref()
            .take_while(|line| !line.trim().is_empty())
            .collect::<Vec<String>>()
            .join("\n")
            .trim_end()
            .to_owned();
        Ok(Class::new(
            origin_name,
            None,
            utilities.collect(),
            passives.collect(),
            PrimaryAction::new(primary_name, primary_description),
            SecondaryAction::new(secondary_name, secondary_description),
            SpecialAction::new(special_name, special_description),
            None,
        ))
    }
}

fn get_hub() -> DriveHub<HttpsConnector<HttpConnector>> {
    DriveHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_only()
                .enable_http2()
                .build(),
        ),
        NoToken,
    )
}

async fn get_rules() -> Result<Vec<String>, GetOriginsAndClassesError> {
    let content = String::from_utf8(
        to_bytes(
            get_hub()
                .files()
                .export(GREED_RULES_DOC_ID, RULES_EXPORT_FORMAT)
                .param("key", API_KEY)
                .doit()
                .await?,
        )
        .await?
        .to_vec(),
    )?;

    info!("Rules Document: {content}");

    Ok(content
        .lines()
        .skip_while(|paragraph| !paragraph.starts_with("Origins"))
        .skip(1)
        .map(String::from)
        .collect())
}

#[allow(clippy::skip_while_next)]
pub async fn get_origins_and_classes() -> Result<ClassCache, GetOriginsAndClassesError> {
    let lines = get_rules().await?;
    let mut origin_lines = lines.clone().into_iter();

    let mut origins = Vec::<Class>::new();
    let mut line = origin_lines.next();
    while !line
        .as_ref()
        .ok_or(GetOriginsAndClassesError::FormatChange)?
        .contains('(')
    {
        let origin = get_origin(
            &line.ok_or(GetOriginsAndClassesError::OriginParse)?,
            origin_lines.by_ref(),
        )?;
        if origin.clone().get_name() == "Human" {
            line = origin_lines
                .by_ref()
                .skip_while(|paragraph| !paragraph.starts_with("Dwarf"))
                .next();
        } else {
            line = origin_lines
                .by_ref()
                .skip_while(|paragraph| paragraph.trim().is_empty())
                .skip_while(|paragraph| paragraph.starts_with('_'))
                .skip_while(|paragraph| paragraph.trim().is_empty())
                .next();
        }
        origins.push(origin);
    }

    let class_lines = lines.into_iter();
    let mut class_lines = class_lines.skip_while(|paragraph| !paragraph.contains("(I)"));

    line = class_lines.by_ref().next();

    let mut classes = Vec::<Class>::new();
    while line.is_some() {
        let class = get_class(
            &line.ok_or(GetOriginsAndClassesError::FormatChange)?,
            class_lines.by_ref(),
        )?;
        classes.push(class);
        line = class_lines
            .by_ref()
            .skip_while(|line| !line.contains('('))
            .next();
    }
    Ok(ClassCache::new(origins, classes, get_update_time().await?))
}

pub async fn get_update_time() -> Result<Option<i64>, GetUpdateTimeError> {
    let hub = get_hub();

    let timestamp = hub
        .files()
        .get("1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4")
        .param("fields", "modifiedTime")
        .param("key", API_KEY)
        .doit()
        .await?
        .1
        .modified_time
        .map(|time| time.timestamp());
    info!("New timestamp: {timestamp:?}");
    Ok(timestamp)
}

#[derive(Debug, Error)]
pub enum GetOriginsAndClassesError {
    #[error("Error in the Google Client: {0}")]
    Google(#[from] google_drive3::Error),
    #[error("Error parsing rules document: {0}")]
    Parse(#[from] google_drive3::hyper::Error),
    #[error("Error parsing rules document as utf8: {0}")]
    Utf8Parse(#[from] std::string::FromUtf8Error),
    #[error("The format of the rules doc has changed, please fix parsing")]
    FormatChange,
    #[error("Failed to parse class")]
    ClassParse,
    #[error("Failed to parse orign")]
    OriginParse,
    #[error("Failed to get Update Time: {0}")]
    UpdateTimeError(#[from] GetUpdateTimeError),
}

#[derive(Debug, Error)]
pub enum GetUpdateTimeError {
    #[error("Error in the Google Client: {0}")]
    Google(#[from] google_drive3::Error),
}
