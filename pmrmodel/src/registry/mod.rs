use pmrmodel_base::task_template::MapToArgRef;

mod prepared;

pub use crate::registry::prepared::PreparedChoiceRegistry;

pub trait ChoiceRegistry<T> {
    fn register(&mut self, name: &str, registry: T);
    fn lookup(&self, name: &str) -> Option<MapToArgRef>;
}
