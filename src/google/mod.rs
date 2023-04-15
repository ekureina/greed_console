use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};
use crate::model::classes::{Class, ClassCache, ClassPassive, ClassUtility};

use google_docs1::api::Document;
use google_docs1::oauth2::{self, ServiceAccountAuthenticator};
use google_docs1::{hyper, hyper_rustls, Docs};

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

fn get_creds() -> oauth2::ServiceAccountKey {
    oauth2::parse_service_account_key(include_bytes!("../../credentials.json")).unwrap()
}

async fn get_authenticator(
) -> oauth2::authenticator::Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>
{
    let creds = get_creds();

    ServiceAccountAuthenticator::builder(creds)
        .build()
        .await
        .unwrap()
}

async fn get_document() -> google_docs1::api::Document {
    let sa = get_authenticator().await;

    let hub = Docs::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        sa,
    );

    hub.documents()
        .get("1154Ep1n8AuiG5iQVxNmahIzjb69BQD28C3QmLfta1n4")
        .doit()
        .await
        .unwrap()
        .1
}

#[allow(clippy::let_underscore_drop)]
fn get_class(first_line: &str, mut paragraphs: impl Iterator<Item = String>) -> Class {
    let class_name = first_line.split('(').collect::<Vec<&str>>()[0]
        .trim()
        .to_string();
    let class_requirements = if first_line.contains("Req:") {
        first_line.split("Req:").collect::<Vec<&str>>()[1]
            .trim()
            .split(',')
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|requirement_class| requirement_class.trim().to_string())
            .collect()
    } else {
        vec![]
    };

    paragraphs.next().unwrap();
    let utility_name = paragraphs.next().unwrap().trim_end().to_owned();
    let utility_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Passive"))
        .collect::<String>()
        .trim_end()
        .to_owned();
    let passive_name = paragraphs.next().unwrap().trim_end().to_owned();
    let passive_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Primary"))
        .collect::<String>()
        .trim_end()
        .to_owned();
    let primary_name = paragraphs.next().unwrap().trim_end().to_owned();
    let primary_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Secondary"))
        .collect::<String>()
        .trim_end()
        .to_owned();
    let secondary_name = paragraphs.next().unwrap().trim_end().to_owned();
    let secondary_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Special"))
        .collect::<String>()
        .trim_end()
        .to_owned();
    let special_name = paragraphs.next().unwrap().trim_end().to_owned();
    let special_description = paragraphs
        .by_ref()
        .take_while(|line| !line.starts_with("Subclasses"))
        .collect::<String>()
        .trim_end()
        .to_owned();
    Class::new(
        class_name,
        ClassUtility::new(utility_name, utility_description),
        ClassPassive::new(passive_name, passive_description),
        PrimaryAction::new(primary_name, primary_description),
        SecondaryAction::new(secondary_name, secondary_description),
        SpecialAction::new(special_name, special_description),
        class_requirements,
    )
}

#[allow(clippy::let_underscore_drop)]
fn get_origin(first_line: &str, mut paragraphs: impl Iterator<Item = String>) -> Class {
    let origin_name = first_line.split_whitespace().collect::<Vec<&str>>()[0].to_string();
    if origin_name == "Human" {
        Class::new(
            String::from("Human"),
            ClassUtility::new(String::new(), String::new()),
            ClassPassive::new(String::new(), String::new()),
            PrimaryAction::new(String::new(), String::new()),
            SecondaryAction::new(String::new(), String::new()),
            SpecialAction::new(String::new(), String::new()),
            vec![],
        )
    } else {
        paragraphs.next().unwrap();
        let utility_name = paragraphs.next().unwrap().trim_end().to_owned();
        let utility_description = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Passive"))
            .collect::<String>()
            .trim_end()
            .to_owned();
        let passive_name = paragraphs.next().unwrap().trim_end().to_owned();
        let passive_description = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Primary"))
            .collect::<String>()
            .trim_end()
            .to_owned();
        let primary_name = paragraphs.next().unwrap().trim_end().to_owned();
        let primary_description = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Secondary"))
            .collect::<String>()
            .trim_end()
            .to_owned();
        let secondary_name = paragraphs.next().unwrap().trim_end().to_owned();
        let secondary_description = paragraphs
            .by_ref()
            .take_while(|line| !line.starts_with("Special"))
            .collect::<String>()
            .trim_end()
            .to_owned();
        let special_name = paragraphs.next().unwrap().trim_end().to_owned();
        let special_description = paragraphs
            .by_ref()
            .take_while(|line| !line.trim().is_empty())
            .collect::<String>()
            .trim_end()
            .to_owned();
        Class::new(
            origin_name,
            ClassUtility::new(utility_name, utility_description),
            ClassPassive::new(passive_name, passive_description),
            PrimaryAction::new(primary_name, primary_description),
            SecondaryAction::new(secondary_name, secondary_description),
            SpecialAction::new(special_name, special_description),
            vec![],
        )
    }
}

fn convert_to_lines(doc: Document) -> Vec<String> {
    let content = doc.body.unwrap().content.unwrap();

    content
        .iter()
        .filter_map(|element| {
            element.paragraph.as_ref().and_then(|p| {
                p.elements.as_ref().map(|e| {
                    e.iter()
                        .filter_map(|pe| pe.text_run.as_ref().and_then(|tr| tr.content.clone()))
                        .collect::<String>()
                })
            })
        })
        .skip_while(|paragraph| !paragraph.starts_with("Origins"))
        .skip(1)
        .collect()
}

#[allow(clippy::skip_while_next)]
pub async fn get_origins_and_classes() -> ClassCache {
    let document = get_document().await;

    let mut lines = convert_to_lines(document).into_iter();

    let mut origins = Vec::<Class>::new();
    let mut line = lines.next();
    while !line.as_ref().unwrap().starts_with("Template") {
        let origin = get_origin(&line.unwrap(), lines.by_ref());
        if origin.clone().get_name() == "Human" {
            line = lines
                .by_ref()
                .skip_while(|paragraph| !paragraph.starts_with("Dwarf"))
                .next();
        } else {
            line = lines
                .by_ref()
                .skip_while(|paragraph| paragraph.trim().is_empty())
                .next();
        }
        origins.push(origin);
    }

    let _ = lines
        .by_ref()
        .skip_while(|paragraph| !paragraph.starts_with("Template"));
    line = lines.by_ref().skip_while(|line| !line.contains('(')).next();

    let mut classes = Vec::<Class>::new();
    while line.is_some() {
        let class = get_class(&line.unwrap(), lines.by_ref());
        classes.push(class);
        line = lines.by_ref().skip_while(|line| !line.contains('(')).next();
    }
    ClassCache::new(origins, classes)
}
