// shouldn't this be schemaorg module?
use serde::Serialize;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Thing {
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
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListItem {
    item: Box<dyn SchemaOrg>,
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

trait SchemaOrg: erased_serde::Serialize {}
impl SchemaOrg for Thing {}
impl SchemaOrg for ListItem {}
impl SchemaOrg for ItemList {}
erased_serde::serialize_trait_object!(SchemaOrg);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_thing() {
        let thing = Thing {
            name: Some("A Thing".to_string()),
            ..Default::default()
        };

        assert_eq!(
            serde_json::to_string(&thing).unwrap(),
            "{\"name\":\"A Thing\"}",
        );
    }

}
