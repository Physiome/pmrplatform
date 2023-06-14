use parking_lot::{
    Mutex,
    MutexGuard,
};
use pmrmodel_base::task_template::{
    MapToArgRef,
    TaskTemplateArg,
};
use std::{
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use crate::registry::ChoiceRegistry;

pub struct ChoiceRegistryCache<'a, T> {
    registry: &'a dyn ChoiceRegistry<T>,
    // punting the cache for the arg to the task_template_arg's unique id.
    arg: Arc<Mutex<HashMap<i64, Option<MapToArgRef<'a>>>>>,
    name: Arc<Mutex<HashMap<String, Option<MapToArgRef<'a>>>>>,
    none: Arc<Mutex<Option<MapToArgRef<'a>>>>,
}

impl<'a, T> From<&'a dyn ChoiceRegistry<T>> for ChoiceRegistryCache<'a, T> {
    fn from(registry: &'a dyn ChoiceRegistry<T>) -> Self {
        Self {
            registry: registry,
            arg: Arc::new(Mutex::new(HashMap::new())),
            name: Arc::new(Mutex::new(HashMap::new())),
            none: Arc::new(Mutex::new(None)),
        }
    }
}

impl<'a, T> ChoiceRegistryCache<'a, T> {
    pub fn lookup(
        &'a self,
        tta: &'a TaskTemplateArg,
    ) -> impl Deref<Target = Option<MapToArgRef<'a>>> + '_ {
        match &tta.choice_source.as_deref() {
            None => MutexGuard::map(self.none.lock(), |x| x),
            Some("") => {
                MutexGuard::map(
                    self.arg.lock(),
                    |arg| arg
                        .entry(tta.id)
                        .or_insert_with(|| {
                            tta.choices.as_ref().map(|c| c.into())
                        })
                )
            }
            Some(source) => {
                MutexGuard::map(
                    self.name.lock(),
                    |name| name
                        .entry(source.to_string())
                        .or_insert_with(|| {
                            self.registry.lookup(source).map(|c| c.into())
                        })
                )
            }
        }
    }
}
