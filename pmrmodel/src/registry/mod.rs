use pmrcore::task_template::MapToArgRef;

mod prepared;
mod cache;

pub use crate::registry::prepared::PreparedChoiceRegistry;
pub use crate::registry::cache::ChoiceRegistryCache;

pub trait ChoiceRegistry<T>: Send + Sync {
    fn register(&mut self, name: &str, registry: T);
    fn select_keys(&mut self, name: &str, keys: Vec<String>);
    fn lookup<'a>(&'a self, name: &str) -> Option<MapToArgRef<'a>>;
}

pub type PreparedChoiceRegistryCache<'a> = ChoiceRegistryCache<'a, prepared::SizedMapToArgRef>;

#[cfg(test)]
mod test {
    use pmrcore::task_template::TaskTemplateArg;
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

    #[test]
    fn test_layered_registry_cache() -> anyhow::Result<()> {
        let mut r1 = PreparedChoiceRegistry::new();
        r1.register("layer", vec!["r1"].into());
        let c1 = ChoiceRegistryCache::from(&r1 as &dyn ChoiceRegistry<_>);

        let mut r2 = PreparedChoiceRegistry::new();
        r2.register("layer", vec!["r2"].into());
        let c2 = ChoiceRegistryCache::from(&r2 as &dyn ChoiceRegistry<_>);

        let layer_checker = TaskTemplateArg {
            id: 3,
            choice_source: Some("layer".into()),
            .. Default::default()
        };

        let c1_c2 = ChoiceRegistryCache::from(&[&c1, &c2]);
        let c2_c1 = ChoiceRegistryCache::from(vec![&c2, &c1]);

        // base test
        assert_eq!(
            &Some("r1"),
            c1.lookup(&layer_checker).unwrap()
                .as_ref().unwrap()
                .get("r1").unwrap()
        );
        assert_eq!(
            &Some("r2"),
            c2.lookup(&layer_checker).unwrap()
                .as_ref().unwrap()
                .get("r2").unwrap()
        );

        // layered test
        assert_eq!(
            &Some("r1"),
            c1_c2.lookup(&layer_checker).unwrap()
                .as_ref().unwrap()
                .get("r1").unwrap()
        );
        assert_eq!(
            &Some("r2"),
            c2_c1.lookup(&layer_checker).unwrap()
                .as_ref().unwrap()
                .get("r2").unwrap()
        );

        Ok(())
    }
}
