use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::Deref,
};

use super::TaskTemplateArgChoices;

pub struct MapToArgRef<'a> {
    selected_keys: Vec<&'a str>,
    table: HashMap<&'a str, Option<&'a str>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ChoiceRef<'a>(pub &'a str, pub bool);

impl<'a> Deref for MapToArgRef<'a> {
    type Target = HashMap<&'a str, Option<&'a str>>;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl<'a> From<HashMap<&'a str, Option<&'a str>>> for MapToArgRef<'a> {
    fn from(value: HashMap<&'a str, Option<&'a str>>) -> Self {
        Self {
            selected_keys: Vec::new(),
            table: value,
        }
    }
}

impl<'a> From<&'a HashMap<String, String>> for MapToArgRef<'a> {
    fn from(value: &'a HashMap<String, String>) -> Self {
        value.iter()
            .map(|(key, val)| (key.as_ref(), Some(val.as_ref())))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a> From<&'a HashMap<String, Option<String>>> for MapToArgRef<'a> {
    fn from(value: &'a HashMap<String, Option<String>>) -> Self {
        value.iter()
            .map(|(key, val)| (key.as_ref(), val.as_ref().map(|s| s.as_ref())))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a> From<&'a TaskTemplateArgChoices> for MapToArgRef<'a> {
    fn from(value: &'a TaskTemplateArgChoices) -> Self {
        value.iter()
            .map(|c| (c.label.as_ref(), c.to_arg.as_deref()))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a> From<&'a Vec<String>> for MapToArgRef<'a> {
    fn from(value: &'a Vec<String>) -> Self {
        value.iter()
            .map(|s| (s.as_ref(), Some(s.as_ref())))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a> From<&Vec<&'a str>> for MapToArgRef<'a> {
    fn from(value: &Vec<&'a str>) -> Self {
        value.iter()
            .map(|s| (*s, Some(*s)))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for MapToArgRef<'a> {
    fn from(value: [&'a str; N]) -> Self {
        value.iter()
            .map(|s| (*s, Some(*s)))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

// this extracts the choices that are user facing (displayed for users)
// into a vec.
impl<'a> From<MapToArgRef<'a>> for Vec<&'a str> {
    fn from(value: MapToArgRef<'a>) -> Self {
        let mut result: Self = value.table.into_keys()
            .collect();
        result.sort();
        result
    }
}

impl<'a> From<MapToArgRef<'a>> for Vec<ChoiceRef<'a>> {
    fn from(value: MapToArgRef<'a>) -> Self {
        let mut selected_keys = value.selected_keys.iter().peekable();
        let mut keys = value.table.into_keys()
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.iter()
            .map(|s| ChoiceRef(s, (selected_keys.peek() == Some(&&s))
                .then(|| selected_keys.next())
                .is_some()
            ))
            .collect()
    }
}

impl<'a> MapToArgRef<'a> {
    pub fn select_keys(
        &mut self,
        keys: impl Iterator<Item = &'a str>,
    ) {
        let mut selected_keys = keys.filter(|s| self.table.get(s).is_some())
            .collect::<Vec<_>>();
        selected_keys.sort_unstable();
        self.selected_keys = selected_keys;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_choiceref_basic() -> anyhow::Result<()> {
        let values: HashMap<String, String> = serde_json::from_str(r#"{
            "one": "1",
            "two": "2",
            "three": "3"
        }"#)?;
        let lookup: MapToArgRef = (&values).into();
        let choices: Vec<ChoiceRef> = lookup.into();

        // sorted listing by label
        let ChoiceRef(choice, selected) = choices[2];
        assert_eq!(choice, "two");
        assert_eq!(selected, false);

        Ok(())
    }

    #[test]
    fn test_choiceref_selected() -> anyhow::Result<()> {
        let values: HashMap<String, String> = serde_json::from_str(r#"{
            "one": "1",
            "two": "2",
            "three": "3",
            "four": "4"
        }"#)?;
        let mut lookup: MapToArgRef = (&values).into();
        // unknown keys silently ignored
        lookup.select_keys(["a", "one", "three"].into_iter());
        let choices: Vec<ChoiceRef> = lookup.into();

        assert_eq!(choices, [
            ChoiceRef("four", false),
            ChoiceRef("one", true),
            ChoiceRef("three", true),
            ChoiceRef("two", false),
        ]);

        Ok(())
    }

}
