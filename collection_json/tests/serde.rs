use std::str::FromStr;
use collection_json::Collection;

static BASIC: &'static str = r#"{
  "collection": {
    "version": "1.0",
    "href": "http://example.com/"
  }
}"#;

#[test]
fn basic() -> anyhow::Result<()> {
    let collection = Collection::new("http://example.com")?;
    let s = collection.to_string();
    assert_eq!(s, BASIC);
    assert_eq!(Collection::from_str(BASIC)?, collection);
    Ok(())
}

static UNSUPPORTED_VERSION_STR: &'static str = r#"{
  "collection": {
    "version": "0.0",
    "href": "http://example.com/"
  }
}"#;

static UNSUPPORTED_VERSION_TYPE: &'static str = r#"{
  "collection": {
    "version": null,
    "href": "http://example.com/"
  }
}"#;

#[test]
fn unsupported_version() -> anyhow::Result<()> {
    assert_eq!(
        format!(
            "{}",
            serde_json::from_str::<Collection>(UNSUPPORTED_VERSION_STR)
                .unwrap_err(),
        ),
        r#"invalid version "0.0", only "1.0" is supported at line 3 column 20"#,
    );
    assert_eq!(
        format!(
            "{}",
            serde_json::from_str::<Collection>(UNSUPPORTED_VERSION_TYPE)
                .unwrap_err(),
        ),
        r#"invalid type: null, expected a string at line 3 column 19"#,
    );
    Ok(())
}

static INVALID_URL: &'static str = r#"{
  "collection": {
    "version": "1.0",
    "href": "not_a_url"
  }
}"#;

#[test]
fn invalid_url() -> anyhow::Result<()> {
    assert_eq!(
        format!(
            "{}",
            serde_json::from_str::<Collection>(INVALID_URL)
                .unwrap_err(),
        ),
        r#"relative URL without a base: "not_a_url" at line 4 column 23"#,
    );
    Ok(())
}

// example 1 copied verbatim from the specification
static EXAMPLE_1: &'static str = r#"
{ "collection" :
  {
    "version" : "1.0",
    "href" : "http://example.org/friends/",

    "links" : [
      {"rel" : "feed", "href" : "http://example.org/friends/rss"}
    ],

    "items" : [
      {
        "href" : "http://example.org/friends/jdoe",
        "data" : [
          {"name" : "full-name", "value" : "J. Doe", "prompt" : "Full Name"},
          {"name" : "email", "value" : "jdoe@example.org", "prompt" : "Email"}
        ],
        "links" : [
          {"rel" : "blog", "href" : "http://examples.org/blogs/jdoe", "prompt" : "Blog"},
          {"rel" : "avatar", "href" : "http://examples.org/images/jdoe", "prompt" : "Avatar", "render" : "image"}
        ]
      },

      {
        "href" : "http://example.org/friends/msmith",
        "data" : [
          {"name" : "full-name", "value" : "M. Smith", "prompt" : "Full Name"},
          {"name" : "email", "value" : "msmith@example.org", "prompt" : "Email"}
        ],
        "links" : [
          {"rel" : "blog", "href" : "http://examples.org/blogs/msmith", "prompt" : "Blog"},
          {"rel" : "avatar", "href" : "http://examples.org/images/msmith", "prompt" : "Avatar", "render" : "image"}
        ]
      },

      {
        "href" : "http://example.org/friends/rwilliams",
        "data" : [
          {"name" : "full-name", "value" : "R. Williams", "prompt" : "Full Name"},
          {"name" : "email", "value" : "rwilliams@example.org", "prompt" : "Email"}
        ],
        "links" : [
          {"rel" : "blog", "href" : "http://examples.org/blogs/rwilliams", "prompt" : "Blog"},
          {"rel" : "avatar", "href" : "http://examples.org/images/rwilliams", "prompt" : "Avatar", "render" : "image"}
        ]
      }
    ],

    "queries" : [
      {"rel" : "search", "href" : "http://example.org/friends/search", "prompt" : "Search",
        "data" : [
          {"name" : "search", "value" : ""}
        ]
      }
    ],

    "template" : {
      "data" : [
        {"name" : "full-name", "value" : "", "prompt" : "Full Name"},
        {"name" : "email", "value" : "", "prompt" : "Email"},
        {"name" : "blog", "value" : "", "prompt" : "Blog"},
        {"name" : "avatar", "value" : "", "prompt" : "Avatar"}

      ]
    }
  }
}"#;

#[test]
fn parse_example1() -> anyhow::Result<()> {
    let collection: Collection = serde_json::from_str(EXAMPLE_1)?;
    assert_eq!(collection.href.to_string(), "http://example.org/friends/");
    assert_eq!(collection.links.as_ref().unwrap().len(), 1);
    assert_eq!(collection.items.as_ref().unwrap().len(), 3);
    assert_eq!(collection.queries.as_ref().unwrap().len(), 1);
    assert_eq!(collection.template.as_ref().unwrap().data.as_ref().unwrap().len(), 4);
    assert_eq!(collection.error, None);

    // full round-trip test for good measure
    let s1 = collection.to_string();
    let collection2 = Collection::from_str(&s1)?;
    assert_eq!(collection, collection2);
    let s2 = collection2.to_string();
    assert_eq!(s1, s2);
    Ok(())
}
