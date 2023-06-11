use pmrmodel_base::task_template::MapToArgRef;

mod prepared;
mod cache;

pub use crate::registry::prepared::PreparedChoiceRegistry;
pub use crate::registry::cache::ChoiceRegistryCache;

pub trait ChoiceRegistry<T> {
    fn register(&mut self, name: &str, registry: T);
    fn lookup<'a>(&'a self, name: &str) -> Option<MapToArgRef<'a>>;
}

#[cfg(test)]
mod test {
    use pmrmodel_base::task_template::{
        MapToArgRef,
        TaskTemplateArg,
    };
    use crate::registry::{
        ChoiceRegistry,
        PreparedChoiceRegistry,
        ChoiceRegistryCache,
    };

    #[test]
    fn test_registry_cache() {
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("static_file_list", vec![
            "file_a",
            "file_b",
            "file_c",
        ].into());
        let mut cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let arg_ext = TaskTemplateArg {
            choice_source: Some("static_file_list".into()),
            .. Default::default()
        };
        assert_eq!(
            &Some("file_a"),
            cache.lookup(&arg_ext).unwrap().get("file_a").unwrap(),
        );
    }
}
