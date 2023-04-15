use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClassUtility {
    name: String,
    description: String,
}

impl ClassUtility {
    pub fn new<N: Into<String>, D: Into<String>>(name: N, description: D) -> Self {
        ClassUtility {
            name: name.into(),
            description: description.into(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClassPassive {
    name: String,
    description: String,
}

impl ClassPassive {
    pub fn new<N: Into<String>, D: Into<String>>(name: N, description: D) -> Self {
        ClassPassive {
            name: name.into(),
            description: description.into(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Class {
    name: String,
    utility: ClassUtility,
    passive: ClassPassive,
    primary_action: PrimaryAction,
    secondary_action: SecondaryAction,
    special_action: SpecialAction,
    class_requirements: Vec<String>,
}

impl Class {
    pub fn new<N: Into<String>>(
        name: N,
        utility: ClassUtility,
        passive: ClassPassive,
        primary_action: PrimaryAction,
        secondary_action: SecondaryAction,
        special_action: SpecialAction,
        class_requirements: Vec<String>,
    ) -> Class {
        Class {
            name: name.into(),
            utility,
            passive,
            primary_action,
            secondary_action,
            special_action,
            class_requirements,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_utility(&self) -> ClassUtility {
        self.utility.clone()
    }

    pub fn get_passive(&self) -> ClassPassive {
        self.passive.clone()
    }

    pub fn get_primary_action(&self) -> PrimaryAction {
        self.primary_action.clone()
    }

    pub fn get_secondary_action(&self) -> SecondaryAction {
        self.secondary_action.clone()
    }

    pub fn get_special_action(&self) -> SpecialAction {
        self.special_action.clone()
    }

    pub fn get_class_available(&self, current_class_names: &[String]) -> bool {
        self.class_requirements
            .iter()
            .all(|class_requirement| current_class_names.contains(class_requirement))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClassCache {
    origins: Vec<Class>,
    classes: Vec<Class>,
}

impl ClassCache {
    pub fn new(origins: Vec<Class>, classes: Vec<Class>) -> ClassCache {
        ClassCache { origins, classes }
    }

    pub fn get_origins(&self) -> Vec<Class> {
        self.origins.clone()
    }

    pub fn get_classes(&self) -> Vec<Class> {
        self.classes.clone()
    }
}
