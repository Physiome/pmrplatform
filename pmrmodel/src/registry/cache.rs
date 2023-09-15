use parking_lot::{
    Mutex,
    MutexGuard,
};
use pmrcore::task_template::{
    MapToArgRef,
    TaskTemplateArg,
};
use std::{
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use crate::error::LookupError;
use crate::registry::ChoiceRegistry;

pub struct ChoiceRegistryCache<'a, T> {
    registry: &'a dyn ChoiceRegistry<T>,
    // punting the cache for the arg to the task_template_arg's unique id.
    arg_list: Arc<Mutex<HashMap<i64, Option<Vec<&'a str>>>>>,
    arg_map: Arc<Mutex<HashMap<i64, Option<MapToArgRef<'a>>>>>,
    name_list: Arc<Mutex<HashMap<String, Option<Vec<&'a str>>>>>,
    name_map: Arc<Mutex<HashMap<String, Option<MapToArgRef<'a>>>>>,
    none_list: Arc<Mutex<Option<Vec<&'a str>>>>,
    none_map: Arc<Mutex<Option<MapToArgRef<'a>>>>,
}

impl<'a, T> From<&'a dyn ChoiceRegistry<T>> for ChoiceRegistryCache<'a, T> {
    fn from(registry: &'a dyn ChoiceRegistry<T>) -> Self {
        Self {
            registry: registry,
            arg_list: Arc::new(Mutex::new(HashMap::new())),
            arg_map: Arc::new(Mutex::new(HashMap::new())),
            name_list: Arc::new(Mutex::new(HashMap::new())),
            name_map: Arc::new(Mutex::new(HashMap::new())),
            none_list: Arc::new(Mutex::new(None)),
            none_map: Arc::new(Mutex::new(None)),
        }
    }
}

impl<'a, T> ChoiceRegistryCache<'a, T> {
    // generate a list of user choices for the provided arg
    pub fn list(
        &'a self,
        tta: &'a TaskTemplateArg,
    ) -> Result<
        impl Deref<Target = Option<Vec<&'a str>>>,
        LookupError,
    > {
        match &tta.choice_source.as_deref() {
            None => Ok(MutexGuard::map(self.none_list.lock(), |x| x)),
            Some("") => {
                Ok(MutexGuard::map(
                    self.arg_list.lock(),
                    |arg| arg
                        .entry(tta.id)
                        .or_insert_with(|| {
                            tta.choices.as_ref().map(|choices| {
                                choices.iter()
                                    .map(|choice| choice.label.as_ref())
                                    .collect::<Vec<_>>()
                            })
                        })
                ))
            }
            Some(source) => {
                let result = MutexGuard::map(
                    self.name_list.lock(),
                    |name| name
                        .entry(source.to_string())
                        .or_insert_with(|| {
                            self.registry.lookup(source)
                                .map(|c| c.into())
                        })
                );
                if result.as_ref().is_none() {
                    Err(LookupError::RegistryMissing(
                        tta.id, source.to_string()))
                }
                else {
                    Ok(result)
                }
            }
        }
    }

    // lookup the mapping to resolve the choices for the provided arg
    pub fn lookup(
        &'a self,
        tta: &'a TaskTemplateArg,
    ) -> Result<
        impl Deref<Target = Option<MapToArgRef<'a>>> + '_,
        LookupError,
    > {
        match &tta.choice_source.as_deref() {
            None => Ok(MutexGuard::map(self.none_map.lock(), |x| x)),
            Some("") => {
                Ok(MutexGuard::map(
                    self.arg_map.lock(),
                    |arg| arg
                        .entry(tta.id)
                        .or_insert_with(|| {
                            tta.choices.as_ref().map(|c| c.into())
                        })
                ))
            }
            Some(source) => {
                let result = MutexGuard::map(
                    self.name_map.lock(),
                    |name| name
                        .entry(source.to_string())
                        .or_insert_with(|| {
                            self.registry.lookup(source)
                                .map(|c| c.into())
                        })
                );
                if result.as_ref().is_none() {
                    Err(LookupError::RegistryMissing(
                        tta.id, source.to_string()))
                }
                else {
                    Ok(result)
                }
            }
        }
    }
}
