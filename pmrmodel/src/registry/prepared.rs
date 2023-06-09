use pmrmodel_base::task_template::{
    MapToArgRef,
    TaskTemplateArgChoices,
};
use std::collections::HashMap;

use crate::registry::ChoiceRegistry;

enum SizedMapToArgRef {
    ArgChoices(TaskTemplateArgChoices),
    VecString(Vec<String>),
    VecStaticStr(Vec<&'static str>),
}

impl<'a> From<&'a SizedMapToArgRef> for MapToArgRef<'a> {
    fn from(value: &'a SizedMapToArgRef) -> Self {
        match value {
            SizedMapToArgRef::ArgChoices(v) => v.into(),
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

pub struct PreparedChoiceRegistry(HashMap<String, SizedMapToArgRef>);

impl ChoiceRegistry<SizedMapToArgRef> for PreparedChoiceRegistry {
    fn register(&mut self, name: &str, registry: SizedMapToArgRef) {
        self.0.insert(name.to_string(), registry);
    }

    fn lookup(&self, name: &str) -> Option<MapToArgRef> {
        self.0
            .get(name)
            .map(|v| v.into())
    }
}

impl PreparedChoiceRegistry {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}


#[cfg(test)]
mod test {
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
        let lookup = registry.lookup("files").unwrap();
        // nested some may be confusing, but there is one for whether
        // the term exists, and then whether or not the term resolves
        // to a &str or a usable None.
        assert_eq!(lookup.get("file_a"), Some(&Some("file_a")))
    }
}
