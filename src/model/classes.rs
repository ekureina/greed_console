use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Class {
    name: String,
    primary_action: PrimaryAction,
    secondary_action: SecondaryAction,
    special_action: SpecialAction,
    class_requirements: Vec<String>,
}

impl Class {
    pub fn new<N: Into<String>>(
        name: N,
        primary_action: PrimaryAction,
        secondary_action: SecondaryAction,
        special_action: SpecialAction,
        class_requirements: Vec<String>,
    ) -> Class {
        Class {
            name: name.into(),
            primary_action,
            secondary_action,
            special_action,
            class_requirements,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
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
