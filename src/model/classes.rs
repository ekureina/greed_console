use crate::model::actions::{PrimaryAction, SecondaryAction, SpecialAction};

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn get_class_available(&self, current_class_names: Vec<String>) -> bool {
        self.class_requirements
            .iter()
            .all(|class_requirement| current_class_names.contains(class_requirement))
    }
}
