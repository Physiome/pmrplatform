use pmrmodel_base::task_template::{
    MapToArgRef,
    TaskTemplateArg,
};
use std::collections::HashMap;

use crate::registry::ChoiceRegistry;

pub struct ChoiceRegistryCache<'a, T> {
    registry: &'a dyn ChoiceRegistry<T>,
    // punting the cache for the arg to the task_template_arg's unique id.
    arg: HashMap<i64, Option<MapToArgRef<'a>>>,
    name: HashMap<String, Option<MapToArgRef<'a>>>,
}

impl<'a, T> From<&'a dyn ChoiceRegistry<T>> for ChoiceRegistryCache<'a, T> {
    fn from(registry: &'a dyn ChoiceRegistry<T>) -> Self {
        Self {
            registry: registry,
            arg: HashMap::new(),
            name: HashMap::new(),
        }
    }
}

impl<'a, T> ChoiceRegistryCache<'a, T> {
    pub fn lookup(
        &'a mut self,
        tta: &'a TaskTemplateArg,
    ) -> Option<&MapToArgRef<'a>> {
        match &tta.choice_source {
            None => None,
            Some(source) => {
                if source == "" {
                    self.arg.entry(tta.id).or_insert_with(|| {
                        tta.choices.as_ref().map(|c| c.into())
                    }).as_ref()
                }
                else {
                    self.name.entry(source.to_string()).or_insert_with(|| {
                        self.registry.lookup(source).map(|c| c.into())
                    }).as_ref()
                }
            }
        }
    }
}
