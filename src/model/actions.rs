/**
 * Struct defining a Greed Special Action
 */
#[derive(Debug, Clone)]
pub struct SpecialAction {
    name: String,
    description: String,
    usable: bool,
}

impl SpecialAction {
    pub fn new<N: Into<String>, D: Into<String>>(name: N, description: D) -> Self {
        SpecialAction {
            name: name.into(),
            description: description.into(),
            usable: true,
        }
    }

    pub fn is_named<S: Into<String>>(&self, name: S) -> bool {
        self.name == name.into()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    pub fn is_usable(&self) -> bool {
        self.usable
    }

    pub fn use_action(&mut self) {
        self.usable = false;
    }

    pub fn refresh(&mut self) {
        self.usable = true;
    }
}

/**
 * Impl of ``PartialEq`` for ``SpecialAction``, only compare names
 */
impl PartialEq for SpecialAction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// ``SpecialAction`` satisfies ``Eq``, so mark it
impl Eq for SpecialAction {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_action_eq_guarantees() {
        let special1 = SpecialAction::new("Test", "");
        let special2 = SpecialAction::new("Test", "");
        let special3 = SpecialAction::new("Test", "");
        let special4 = SpecialAction::new("OtherTest", "");

        // Reflexive
        assert_eq!(special1, special1);

        // Symmetric
        assert_eq!(special1, special2);
        assert_eq!(special2, special1);

        // Transitive
        assert_eq!(special2, special3);
        assert_eq!(special1, special3);

        assert_ne!(special1, special4);
    }

    #[test]
    fn test_special_action_eq_with_usable() {
        let special1 = SpecialAction::new("Test", "");
        let mut special2 = SpecialAction::new("Test", "");

        assert!(special1.is_usable());
        assert!(special2.is_usable());

        assert_eq!(special1, special2);

        special2.use_action();

        assert!(special1.is_usable());
        assert!(!special2.is_usable());

        assert_eq!(special1, special2);
    }

    #[test]
    fn test_use_and_refresh() {
        let mut special = SpecialAction::new("Test", "");

        assert!(special.is_usable());

        special.use_action();

        assert!(!special.is_usable());

        special.refresh();

        assert!(special.is_usable());
    }

    #[test]
    fn test_is_named() {
        let special = SpecialAction::new("Test", "");

        assert!(special.is_named("Test"));
        assert!(!special.is_named("Prod"));
    }

    #[test]
    fn test_name() {
        let special = SpecialAction::new("Test", "");

        assert_eq!(special.get_name(), "Test");
    }

    #[test]
    fn test_description() {
        let special = SpecialAction::new("Test", "Lorem ipsum");

        assert_eq!(special.get_description(), "Lorem ipsum");
    }
}
