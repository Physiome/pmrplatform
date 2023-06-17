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
    use pmrmodel_base::task_template::TaskTemplateArg;
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
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let arg = TaskTemplateArg {
            .. Default::default()
        };
        let arg_int = TaskTemplateArg {
            id: 1,
            choice_source: Some("".into()),
            choices: serde_json::from_str(r#"[
                {
                    "to_arg": null,
                    "label": "omit"
                },
                {
                    "to_arg": "the target string",
                    "label": "keep"
                }
            ]"#).unwrap(),
            .. Default::default()
        };
        let arg_ext = TaskTemplateArg {
            id: 2,
            choice_source: Some("static_file_list".into()),
            .. Default::default()
        };

        assert_eq!(
            true,
            cache
                .lookup(&arg)
                .unwrap()
                .as_ref()
                .is_none(),
        );
        assert_eq!(
            None,
            cache
                .lookup(&arg_int)
                .unwrap()
                .as_ref()
                .unwrap()
                .get("no such value"),
        );
        assert_eq!(
            &None,
            cache
                .lookup(&arg_int)
                .unwrap()
                .as_ref()
                .unwrap()
                .get("omit")
                .unwrap(),
        );
        assert_eq!(
            &Some("the target string"),
            cache
                .lookup(&arg_int)
                .unwrap()
                .as_ref()
                .unwrap()
                .get("keep")
                .unwrap(),
        );
        assert_eq!(
            &Some("file_a"),
            cache
                .lookup(&arg_ext)
                .unwrap()
                .as_ref()
                .unwrap()
                .get("file_a")
                .unwrap(),
        );
    }
}
