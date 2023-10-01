use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use std::any::Any;

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

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Class {
    name: String,
    #[serde(default)]
    level: Option<usize>,
    utilities: Vec<ClassUtility>,
    passives: Vec<ClassPassive>,
    primary_action: PrimaryAction,
    secondary_action: SecondaryAction,
    special_action: SpecialAction,
    class_requirements: Option<Box<dyn ClassRequirement>>,
}

impl Class {
    #[allow(clippy::too_many_arguments)]
    pub fn new<N: Into<String>>(
        name: N,
        level: Option<usize>,
        utilities: Vec<ClassUtility>,
        passives: Vec<ClassPassive>,
        primary_action: PrimaryAction,
        secondary_action: SecondaryAction,
        special_action: SpecialAction,
        class_requirements: Option<Box<dyn ClassRequirement>>,
    ) -> Class {
        Class {
            name: name.into(),
            level,
            utilities,
            passives,
            primary_action,
            secondary_action,
            special_action,
            class_requirements,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_utilities(&self) -> &Vec<ClassUtility> {
        &self.utilities
    }

    pub fn get_passives(&self) -> &Vec<ClassPassive> {
        &self.passives
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

    pub fn get_class_available(&self, current_classes: &[Class]) -> bool {
        match &self.class_requirements {
            Some(requirements) => requirements.meets_requirement(current_classes),
            None => true,
        }
    }

    pub fn get_level(&self) -> &Option<usize> {
        &self.level
    }
}

#[typetag::serde]
pub trait ClassRequirement: std::fmt::Debug + std::marker::Send {
    fn meets_requirement(&self, current_classes: &[Class]) -> bool;
    fn clone_dyn(&self) -> Box<dyn ClassRequirement>;
    #[allow(clippy::borrowed_box)]
    fn partial_eq_dyn(&self, other: &Box<dyn ClassRequirement>) -> bool;
}

impl Clone for Box<dyn ClassRequirement> {
    fn clone(&self) -> Box<dyn ClassRequirement> {
        self.clone_dyn()
    }
}

impl PartialEq for Box<dyn ClassRequirement> {
    fn eq(&self, other: &Box<dyn ClassRequirement>) -> bool {
        self.partial_eq_dyn(other)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SuperClassRequirement {
    class_name: String,
}

#[typetag::serde]
impl ClassRequirement for SuperClassRequirement {
    fn meets_requirement(&self, current_classes: &[Class]) -> bool {
        current_classes
            .iter()
            .map(Class::get_name)
            .any(|current_class_name| current_class_name == self.class_name)
    }

    fn clone_dyn(&self) -> Box<dyn ClassRequirement> {
        Box::new(self.clone())
    }

    fn partial_eq_dyn(&self, other: &Box<dyn ClassRequirement>) -> bool {
        if <dyn Any>::is::<&Self>(other) {
            self.eq((other as &dyn Any).downcast_ref().unwrap())
        } else {
            false
        }
    }
}

impl SuperClassRequirement {
    pub fn new(class_name: impl Into<String>) -> Self {
        SuperClassRequirement {
            class_name: class_name.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AndClassRequirement {
    left: Box<dyn ClassRequirement>,
    right: Box<dyn ClassRequirement>,
}

impl PartialEq for AndClassRequirement {
    fn eq(&self, other: &Self) -> bool {
        self.left.eq(&other.left) && self.right.eq(&other.right)
    }
}

#[typetag::serde]
impl ClassRequirement for AndClassRequirement {
    fn meets_requirement(&self, current_classes: &[Class]) -> bool {
        self.left.meets_requirement(current_classes)
            && self.right.meets_requirement(current_classes)
    }

    fn clone_dyn(&self) -> Box<dyn ClassRequirement> {
        Box::new(self.clone())
    }

    fn partial_eq_dyn(&self, other: &Box<dyn ClassRequirement>) -> bool {
        if <dyn Any>::is::<&Self>(other) {
            self.eq((other as &dyn Any).downcast_ref().unwrap())
        } else {
            false
        }
    }
}

impl AndClassRequirement {
    pub fn new(left: Box<dyn ClassRequirement>, right: Box<dyn ClassRequirement>) -> Self {
        AndClassRequirement { left, right }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct LevelPrefixRequirement {
    level: usize,
    prefix: String,
}

#[typetag::serde]
impl ClassRequirement for LevelPrefixRequirement {
    fn meets_requirement(&self, current_classes: &[Class]) -> bool {
        current_classes.iter().any(|current_class| {
            current_class
                .get_level()
                .is_some_and(|level| level >= self.level)
                && current_class.get_name().starts_with(&self.prefix)
        })
    }

    fn clone_dyn(&self) -> Box<dyn ClassRequirement> {
        Box::new(self.clone())
    }

    fn partial_eq_dyn(&self, other: &Box<dyn ClassRequirement>) -> bool {
        if <dyn Any>::is::<&Self>(other) {
            self.eq((other as &dyn Any).downcast_ref().unwrap())
        } else {
            false
        }
    }
}

impl LevelPrefixRequirement {
    pub fn new(level: usize, prefix: impl Into<String>) -> Self {
        LevelPrefixRequirement {
            level,
            prefix: prefix.into(),
        }
    }
}

#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct ClassCache {
    origins: IndexMap<String, Class>,
    classes: IndexMap<String, Class>,
    #[serde(default)]
    cache_update_time: Option<i64>,
}

impl ClassCache {
    pub fn new(
        origins: Vec<Class>,
        classes: Vec<Class>,
        cache_update_time: Option<i64>,
    ) -> ClassCache {
        ClassCache {
            origins: origins
                .into_iter()
                .map(|origin| (origin.get_name(), origin))
                .collect(),
            classes: classes
                .into_iter()
                .map(|class| (class.get_name(), class))
                .collect(),
            cache_update_time,
        }
    }

    pub fn get_origins(&self) -> Vec<&Class> {
        self.origins.values().collect()
    }

    pub fn get_origin<'a, N: Into<&'a str>>(&self, origin_name: N) -> Option<&Class> {
        self.origins.get(origin_name.into())
    }

    pub fn get_classes(&self) -> Vec<&Class> {
        self.classes.values().collect()
    }

    pub fn get_class_cache_count(&self) -> usize {
        self.classes.len()
    }

    pub fn get_cache_update_time(&self) -> Option<i64> {
        self.cache_update_time
    }

    pub fn map_to_concrete_classes(&self, class_names: &[String]) -> Vec<Class> {
        class_names
            .iter()
            .filter_map(|class_name| self.classes.get(class_name))
            .cloned()
            .collect()
    }
}
