// shouldn't this be schemaorg module?
use serde::Serialize;
use serde_with::skip_serializing_none;


#[derive(Default, Serialize)]
struct ContextTermSimple {
    iri: String,
}

#[derive(Default, Serialize)]
struct ContextTermExpanded {
    #[serde(rename = "@id")]
    id: Option<String>,
    #[serde(rename = "@type")]
    r#type: Option<String>,
    #[serde(rename = "@language")]
    language: Option<String>,
    #[serde(rename = "@context")]
    context: Option<String>,
    #[serde(rename = "@prefix")]
    prefix: Option<String>,
    #[serde(rename = "@propagate")]
    propagate: Option<String>,
    #[serde(rename = "@protected")]
    protected: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(untagged)]
enum ContextTerm {
    String(String),
    ContextTermExpanded(ContextTermExpanded),
}


#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Thing {
    #[serde(rename = "@context")]
    context: Option<ContextTerm>,  // XXX shouldn't be optional, but defaults...

    additional_type: Option<String>,  // URL
    alternative_name: Option<String>,
    description: Option<String>,
    disambiguating_description: Option<String>,
    identifier: Option<String>,  // PropertyValue or Text or URL
    image: Option<String>,  // ImageObject or Text or URL
    main_entity_of_page: Option<String>,   // CreativeWork or URL
    name: Option<String>,
    potential_action: Option<String>,  // Action
    same_as: Option<String>,  // URL
    subject_of: Option<String>,  // CreativeWork or Event
    url: Option<String>,  // URL
}

impl Thing {}

#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListItem {
    // item should be required
    item: Option<Box<dyn SchemaOrg>>,
    next_item: Option<Box<ListItem>>,
    position: Option<u64>,  // could be Text?
    previous_item: Option<Box<ListItem>>,
    #[serde(flatten)]
    thing: Thing,
}

impl ListItem {}

#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemList {
    item_list_element: Vec<ListItem>,
    item_list_order: Option<String>,
    number_of_items: u64,
    #[serde(flatten)]
    thing: Thing,
}

impl ItemList {}

#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreativeWork {
    r#abstract: Option<String>,
    #[serde(flatten)]
    thing: Thing,
}

impl CreativeWork {}

trait SchemaOrg: erased_serde::Serialize {}
impl SchemaOrg for Thing {}
impl SchemaOrg for ListItem {}
impl SchemaOrg for ItemList {}
impl SchemaOrg for CreativeWork {}
erased_serde::serialize_trait_object!(SchemaOrg);

#[derive(Serialize)]
#[serde(tag = "@type")]
enum Schema {
    Thing(Thing),
    ListItem(ListItem),
    ItemList(ItemList),
    CreativeWork(CreativeWork),
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_thing() {
        let thing = Schema::Thing ( Thing {
            context: Some(ContextTerm::String("https://schema.org".to_string())),
            name: Some("A Thing".to_string()),
            ..Default::default()
        });

        assert_eq!(
            serde_json::to_string_pretty(&thing).unwrap(),
            r#"{
  "@type": "Thing",
  "@context": "https://schema.org",
  "name": "A Thing"
}"#,
        );
    }

    #[test]
    fn test_abstract() {
        let thing = Schema::CreativeWork ( CreativeWork {
            r#abstract: Some("The abstract of the CreativeWork".to_string()),
            thing: Thing {
                context: Some(ContextTerm::String("https://schema.org".to_string())),
                name: Some("Some piece of work".to_string()),
                ..Default::default()
            },
            ..Default::default()
        });
        assert_eq!(
            serde_json::to_string_pretty(&thing).unwrap(),
            r#"{
  "@type": "CreativeWork",
  "abstract": "The abstract of the CreativeWork",
  "@context": "https://schema.org",
  "name": "Some piece of work"
}"#,
        );
    }

}
