use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::Deref,
};

use super::{
    TaskTemplateArgChoices,
    UserChoice,
    UserChoices,
};

pub struct MapToArgRef<'a> {
    selected_keys: Vec<&'a str>,
    table: HashMap<&'a str, Option<&'a str>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct UserChoiceRef<'a>(pub &'a str, pub bool);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct UserChoiceRefs<'a>(Vec<UserChoiceRef<'a>>);

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

impl<'a> From<&'a UserChoice> for UserChoiceRef<'a> {
    fn from(value: &'a UserChoice) -> Self {
        Self(value.0.as_ref(), value.1)
    }
}

impl From<&UserChoiceRef<'_>> for UserChoice {
    fn from(value: &UserChoiceRef<'_>) -> Self {
        Self(value.0.to_string(), value.1)
    }
}

impl From<&UserChoiceRefs<'_>> for UserChoices {
    fn from(values: &UserChoiceRefs<'_>) -> Self {
        Self(
            values.iter()
                .map(|v| v.into())
                .collect::<Vec<_>>()
        )
    }
}

impl<'a> From<MapToArgRef<'a>> for UserChoiceRefs<'a> {
    fn from(value: MapToArgRef<'a>) -> Self {
        let mut selected_keys = value.selected_keys.iter().peekable();
        let mut keys = value.table.into_keys()
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.iter()
            .map(|s| UserChoiceRef(s, (selected_keys.peek() == Some(&&s))
                .then(|| selected_keys.next())
                .is_some()
            ))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<'a> From<&MapToArgRef<'a>> for UserChoiceRefs<'a> {
    fn from(value: &MapToArgRef<'a>) -> Self {
        let mut selected_keys = value.selected_keys.iter().peekable();
        let mut keys = value.table.keys()
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.iter()
            .map(|s| UserChoiceRef(s, (selected_keys.peek() == Some(&&s))
                .then(|| selected_keys.next())
                .is_some()
            ))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<'a> From<Vec<UserChoiceRef<'a>>> for UserChoiceRefs<'a> {
    fn from(args: Vec<UserChoiceRef<'a>>) -> Self {
        Self(args)
    }
}

impl<'a, const N: usize> From<[UserChoiceRef<'a>; N]> for UserChoiceRefs<'a> {
    fn from(args: [UserChoiceRef<'a>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a> Deref for UserChoiceRefs<'a> {
    type Target = Vec<UserChoiceRef<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
        let choices: UserChoiceRefs = lookup.into();

        // sorted listing by label
        let UserChoiceRef(choice, selected) = choices[2];
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
        let choices: UserChoiceRefs = lookup.into();

        assert_eq!(choices, [
            UserChoiceRef("four", false),
            UserChoiceRef("one", true),
            UserChoiceRef("three", true),
            UserChoiceRef("two", false),
        ].into());

        Ok(())
    }

}
