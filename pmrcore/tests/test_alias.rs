use pmrcore::{
    alias::AliasEntry,
    workspace::{
        Workspace,
        traits::Workspace as _,
    },
    platform::MCPlatform,
};
use test_pmr::core::MockPlatform;

// like the other instance, it would be good if the testing crate and
// this crate that's being tested somehow also pretend the crate imports
// are identical, but since one is identified as crate and the other via
// its fully qualified import path, this external test will need to be
// done instead of the slightly more straightforward module level tests.
#[async_std::test]
async fn list_aliased_workspaces() -> anyhow::Result<()> {
    let mut platform = MockPlatform::new();
    platform.expect_aliases_by_kind()
        .times(1)
        .withf(|a| a == "workspace")
        .returning(move |_| Ok(vec![
            ("test_workspace_1".to_string(), 201),
            ("test_workspace_2".to_string(), 402),
            ("test_workspace_3".to_string(), 909),
        ]));
    platform.expect_workspace_list_workspace_by_ids()
        .times(1)
        .withf(|ids| ids == &[201, 402, 909])
        .returning(move |_| Ok(serde_json::from_str::<Vec<Workspace>>(r#"[{
                "id": 201,
                "url": "http://example.com/1",
                "created_ts": 0
            }, {
                "id": 402,
                "url": "http://example.com/2",
                "created_ts": 0
            }, {
                "id": 909,
                "url": "http://example.com/3",
                "created_ts": 0
            }]"#)
            .unwrap()
            .into()
        ));

    let results = platform.list_aliased_workspaces().await?;
    assert_eq!(results.kind(), "workspace");
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].alias, "test_workspace_1");
    assert_eq!(results[0].entity.id(), 201);
    assert_eq!(results[2].alias, "test_workspace_3");
    assert_eq!(results[2].entity.id(), 909);

    let converted = results.into_iter()
        .map(|entry| AliasEntry {
            alias: entry.alias,
            entity: entry.entity.into_inner(),
        })
        .collect::<Vec<_>>();

    assert_eq!(converted[0].entity.id, 201);

    Ok(())
}

