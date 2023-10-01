use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};
use crate::model::classes::{
    AndClassRequirement, Class, ClassCache, ClassPassive, ClassRequirement, ClassUtility,
    LevelPrefixRequirement, SuperClassRequirement,
};
use crate::util::from_roman;
use std::path::Path;

use google_docs1::api::Document;
use google_docs1::oauth2::{self, InstalledFlowAuthenticator};
use google_docs1::{hyper, hyper_rustls, Docs};
use google_drive3::DriveHub;
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

fn get_creds() -> std::io::Result<oauth2::ApplicationSecret> {
    oauth2::parse_application_secret(include_bytes!("../../credentials.json"))
}

async fn get_authenticator() -> Result<
    oauth2::authenticator::Authenticator<
        hyper_rustls::HttpsConnector<hyper::client::HttpConnector>,
    >,
    AuthenticatorError,
> {
    let creds = get_creds()?;

    let auth =
        InstalledFlowAuthenticator::builder(creds, oauth2::InstalledFlowReturnMethod::HTTPRedirect)
            .flow_delegate(Box::new(WebbrowserFlowDelegate))
            .persist_tokens_to_disk(
                Path::new(&eframe::storage_dir("Greed Console").unwrap()).join("oauth2"),
            )
            .build()
            .await?;
    let scopes = &[
        google_docs1::api::Scope::DocumentReadonly.as_ref(),
        google_drive3::api::Scope::MetadataReadonly.as_ref(),
    ];
    auth.token(scopes).await.unwrap();
    Ok(auth)
}

async fn get_document() -> Result<google_docs1::api::Document, GetOriginsAndClassesError> {
    let auth = get_authenticator().await?;

    let hub = Docs::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    );

    Ok(hub
        .documents()
        .get("1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4")
        .doit()
        .await?
        .1)
}

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
        if line.starts_with('\t') || line.starts_with('-') {
            utility_description.push_str(&line);
        } else {
            utility_names.push(line.trim_end().to_owned());
            if !utility_description.is_empty() {
                utility_descriptions.push(utility_description.trim_end().to_owned());
                utility_description.clear();
            }
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
        if line.starts_with('\t') || line.starts_with('-') {
            passive_description.push_str(&line);
        } else {
            passive_names.push(line.trim_end().to_owned());
            if !passive_description.is_empty() {
                passive_descriptions.push(passive_description.trim_end().to_owned());
                passive_description.clear();
            }
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
        .trim_end()
        .to_owned();
    let primary_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Secondary"))
        .collect::<String>()
        .trim_end()
        .to_owned();
    let secondary_name = paragraphs
        .next()
        .ok_or(GetOriginsAndClassesError::ClassParse)?
        .trim_end()
        .to_owned();
    let secondary_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Special"))
        .collect::<String>()
        .trim_end()
        .to_owned();
    let special_name = paragraphs
        .next()
        .ok_or(GetOriginsAndClassesError::ClassParse)?
        .trim_end()
        .to_owned();
    let special_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Subclasses"))
        .collect::<String>()
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
        let second_requirement = determine_class_requirements(second.get(1..).unwrap());
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
            if line.starts_with('\t') || line.starts_with('-') {
                utility_description.push_str(&line);
            } else {
                utility_names.push(line.trim_end().to_owned());
                if !utility_description.is_empty() {
                    utility_descriptions.push(utility_description.trim_end().to_owned());
                    utility_description.clear();
                }
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
            if line.starts_with('\t') || line.starts_with('-') {
                passive_description.push_str(&line);
            } else {
                passive_names.push(line.trim_end().to_owned());
                if !passive_description.is_empty() {
                    passive_descriptions.push(passive_description.trim_end().to_owned());
                    passive_description.clear();
                }
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
            .trim_end()
            .to_owned();
        let primary_description = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Secondary"))
            .collect::<String>()
            .trim_end()
            .to_owned();
        let secondary_name = paragraphs
            .next()
            .ok_or(GetOriginsAndClassesError::OriginParse)?
            .trim_end()
            .to_owned();
        let secondary_description = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Special"))
            .collect::<String>()
            .trim_end()
            .to_owned();
        let special_name = paragraphs
            .next()
            .ok_or(GetOriginsAndClassesError::OriginParse)?
            .trim_end()
            .to_owned();
        let special_description = paragraphs
            .by_ref()
            .take_while(|line| !line.trim().is_empty())
            .collect::<String>()
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

fn convert_to_lines(doc: Document) -> Result<Vec<String>, GetOriginsAndClassesError> {
    let content = doc
        .body
        .ok_or(GetOriginsAndClassesError::MissingDoc)?
        .content
        .ok_or(GetOriginsAndClassesError::MissingDoc)?;

    Ok(content
        .iter()
        .filter_map(|element| {
            element.paragraph.as_ref().and_then(|p| {
                let bullet_text = p.bullet.clone().map_or_else(String::new, |b| {
                    b.nesting_level.map_or_else(String::new, |il| {
                        "\t".repeat((il - 1).try_into().unwrap_or(0)) + "- "
                    })
                });
                p.elements
                    .as_ref()
                    .map(|e| {
                        e.iter()
                            .filter_map(|pe| pe.text_run.as_ref().and_then(|tr| tr.content.clone()))
                            .collect::<String>()
                    })
                    .map(|text| bullet_text + &text)
            })
        })
        .skip_while(|paragraph| !paragraph.starts_with("Origins"))
        .skip(1)
        .collect())
}

#[allow(clippy::skip_while_next)]
pub async fn get_origins_and_classes() -> Result<ClassCache, GetOriginsAndClassesError> {
    let document = get_document().await?;

    let lines = convert_to_lines(document)?;
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
    let auth = get_authenticator().await?;

    let hub = DriveHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    );

    let timestamp = hub
        .files()
        .get("1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4")
        .param("fields", "modifiedTime")
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
    #[error("Error getting authenticator: {0}")]
    Authenticator(#[from] AuthenticatorError),
    #[error("Error in the Google Client: {0}")]
    Google(#[from] google_docs1::Error),
    #[error("Missing rules document")]
    MissingDoc,
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
    #[error("Error getting authenticator: {0}")]
    Authenticator(#[from] AuthenticatorError),
    #[error("Error in the Google Client: {0}")]
    Google(#[from] google_drive3::Error),
}

#[derive(Debug, Error)]
pub enum AuthenticatorError {
    #[error("IO error when getting Authenticator: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error in the Oauth2 process: {0}")]
    Oauth2(#[from] google_docs1::oauth2::Error),
}

#[derive(Debug, Clone)]
struct WebbrowserFlowDelegate;

impl oauth2::authenticator_delegate::InstalledFlowDelegate for WebbrowserFlowDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        _need_code: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'a>>
    {
        Box::pin(async {
            webbrowser::open(url).map_err(|err| err.to_string())?;
            Ok(String::new())
        })
    }
}
