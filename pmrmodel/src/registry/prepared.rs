use pmrcore::task_template::{
    MapToArgRef,
    TaskTemplateArgChoices,
};
use std::collections::HashMap;

use crate::registry::ChoiceRegistry;

pub enum SizedMapToArgRef {
    ArgChoices(TaskTemplateArgChoices),
    HashMapStringOptionString(HashMap<String, Option<String>>),
    HashMapStringString(HashMap<String, String>),
    VecString(Vec<String>),
    VecStaticStr(Vec<&'static str>),
}

impl<'a> From<&'a SizedMapToArgRef> for MapToArgRef<'a> {
    fn from(value: &'a SizedMapToArgRef) -> Self {
        match value {
            SizedMapToArgRef::ArgChoices(v) => v.into(),
            SizedMapToArgRef::HashMapStringOptionString(v) => v.into(),
            SizedMapToArgRef::HashMapStringString(v) => v.into(),
            SizedMapToArgRef::VecString(v) => v.into(),
            SizedMapToArgRef::VecStaticStr(v) => v.into(),
        }
    }
}

impl From<TaskTemplateArgChoices> for SizedMapToArgRef {
    fn from(value: TaskTemplateArgChoices) -> Self {
        Self::ArgChoices(value)
    }
}

impl From<HashMap<String, Option<String>>> for SizedMapToArgRef {
    fn from(value: HashMap<String, Option<String>>) -> Self {
        Self::HashMapStringOptionString(value)
    }
}

impl From<HashMap<String, String>> for SizedMapToArgRef {
    fn from(value: HashMap<String, String>) -> Self {
        Self::HashMapStringString(value)
    }
}

impl From<Vec<String>> for SizedMapToArgRef {
    fn from(value: Vec<String>) -> Self {
        Self::VecString(value)
    }
}

impl From<Vec<&'static str>> for SizedMapToArgRef {
    fn from(value: Vec<&'static str>) -> Self {
        Self::VecStaticStr(value)
    }
}

pub struct PreparedChoiceRegistry {
    table: HashMap<String, SizedMapToArgRef>,
    selected_keys: HashMap<String, Vec<String>>,
}

impl ChoiceRegistry<SizedMapToArgRef> for PreparedChoiceRegistry {
    fn register(&mut self, name: &str, registry: SizedMapToArgRef) {
        self.table.insert(name.to_string(), registry);
    }

    fn select_keys(&mut self, name: &str, keys: Vec<String>) {
        self.selected_keys.insert(name.to_string(), keys);
    }

    fn lookup<'a>(&'a self, name: &str) -> Option<MapToArgRef<'a>> {
        self.table
            .get(name)
            .map(|v| {
                let mut result: MapToArgRef = v.into();
                result.select_keys(
                    self.selected_keys
                        .get(name)
                        .map(|v| v.as_slice())
                        .unwrap_or(&[])
                        .into_iter()
                        .map(|s| s.as_ref())
                );
                result
            })
    }
}

impl PreparedChoiceRegistry {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            selected_keys: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use pmrcore::task_template::{
        UserChoiceRef,
        UserChoiceRefs,
    };
    use crate::registry::ChoiceRegistry;
    use crate::registry::PreparedChoiceRegistry;

    #[test]
    fn test_prepared_choice_registry() {
        let mut registry = PreparedChoiceRegistry::new();
        let files = vec![
            "file_a".to_string(),
            "file_b".to_string(),
            "file_c".to_string(),
        ];

        registry.register("files", files.into());
        let lookup = registry.lookup("files")
            .expect("registry returned");
        // nested some may be confusing, but there is one for whether
        // the term exists, and then whether or not the term resolves
        // to a &str or a usable None.
        assert_eq!(lookup.get("file_a"), Some(&Some("file_a")));

        let choices: UserChoiceRefs = (&lookup).into();
        assert_eq!(choices, [
            UserChoiceRef("file_a", false),
            UserChoiceRef("file_b", false),
            UserChoiceRef("file_c", false),
        ].into());

        let choices: UserChoiceRefs = lookup.into();
        assert_eq!(choices, [
            UserChoiceRef("file_a", false),
            UserChoiceRef("file_b", false),
            UserChoiceRef("file_c", false),
        ].into());

    }

    #[test]
    fn test_prepared_choice_registry_select_keys() {
        let mut registry = PreparedChoiceRegistry::new();
        let files = vec![
            "file_a".to_string(),
            "file_b".to_string(),
            "file_c".to_string(),
        ];

        registry.register("files", files.into());
        registry.select_keys("files", vec![
            "a".to_string(),
            "file_a".to_string(),
            "file_d".to_string(),
        ]);

        let choices: UserChoiceRefs = registry.lookup("files")
            .expect("registry returned")
            .into();
        assert_eq!(choices, [
            UserChoiceRef("file_a", true),
            UserChoiceRef("file_b", false),
            UserChoiceRef("file_c", false),
        ].into());
    }
}
