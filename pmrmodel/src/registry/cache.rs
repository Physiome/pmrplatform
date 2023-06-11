use pmrmodel_base::task_template::{
    MapToArgRef,
    TaskTemplateArg,
};
use std::{
    cell::{
        RefCell,
        RefMut,
    },
    collections::HashMap,
    ops::Deref,
};

use crate::registry::ChoiceRegistry;

pub struct ChoiceRegistryCache<'a, T> {
    registry: &'a dyn ChoiceRegistry<T>,
    // punting the cache for the arg to the task_template_arg's unique id.
    arg: RefCell<HashMap<i64, Option<MapToArgRef<'a>>>>,
    name: RefCell<HashMap<String, Option<MapToArgRef<'a>>>>,
    none: RefCell<Option<MapToArgRef<'a>>>,
}

impl<'a, T> From<&'a dyn ChoiceRegistry<T>> for ChoiceRegistryCache<'a, T> {
    fn from(registry: &'a dyn ChoiceRegistry<T>) -> Self {
        Self {
            registry: registry,
            arg: RefCell::new(HashMap::new()),
            name: RefCell::new(HashMap::new()),
            none: RefCell::new(None),
        }
    }
}

impl<'a, T> ChoiceRegistryCache<'a, T> {
    pub fn lookup(
        &'a self,
        tta: &'a TaskTemplateArg,
    ) -> impl Deref<Target = Option<MapToArgRef<'a>>> + '_ {
        match &tta.choice_source.as_deref() {
            None => RefMut::from(self.none.borrow_mut()),
            Some("") => {
                RefMut::map(
                    self.arg.borrow_mut(),
                    |arg| arg
                        .entry(tta.id)
                        .or_insert_with(|| {
                            tta.choices.as_ref().map(|c| c.into())
                        })
                )
            }
            Some(source) => {
                RefMut::map(
                    self.name.borrow_mut(),
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
