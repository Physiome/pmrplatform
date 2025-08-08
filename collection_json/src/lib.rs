//! A very basic Collection+JSON implementation
//!
//! This library simply captures the basic specification and put them
//! into Rust types and the serialization/deserialization done using
//! `serde` and `serde_json`.
//!
//! Since the specification is not expected to undergo further changes,
//! only version 1.0 of the Collection+JSON spec is assumed in the
//! current design.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;
pub use url::ParseError;

// using this intermediate need to include as the key "collection"
#[derive(Deserialize, Serialize)]
enum Container {
    #[serde(rename = "collection")]
    Collection {
        #[serde(serialize_with = "emit_version")]
        #[serde(deserialize_with = "validate_version")]
        version: (),
        href: Url,
        #[serde(skip_serializing_if = "Option::is_none")]
        links: Option<Vec<Link>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        items: Option<Vec<Item>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        queries: Option<Vec<Query>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        template: Option<Template>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Error>,
    }
}

impl From<Container> for Collection {
    fn from(value: Container) -> Self {
        match value {
            Container::Collection {
                version,
                href,
                links,
                items,
                queries,
                template,
                error,
            } => Self {
                version,
                href,
                links,
                items,
                queries,
                template,
                error,
            }
        }
    }
}

impl From<Collection> for Container {
    fn from(value: Collection) -> Self {
        let Collection {
            version,
            href,
            links,
            items,
            queries,
            template,
            error,
        } = value;
        Container::Collection {
            version,
            href,
            links,
            items,
            queries,
            template,
            error,
        }
    }
}

/// Represents the Collection+JSON top-level document property as outlined in
/// section 2.1 collection.
// TODO probably include some example code for this.
#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(from = "Container")]
#[serde(into = "Container")]
pub struct Collection {
    version: (),
    pub href: Url,
    pub links: Option<Vec<Link>>,
    pub items: Option<Vec<Item>>,
    pub queries: Option<Vec<Query>>,
    pub template: Option<Template>,
    pub error: Option<Error>,
}

fn emit_version<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("1.0")
}

fn validate_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
    let s = String::deserialize(deserializer)?;
    if s == "1.0" {
        Ok(())
    } else {
        Err(serde::de::Error::custom(format!("invalid version {s:?}, only \"1.0\" is supported")))
    }
}

/// Captures the any of the five possible properties of an object in a
/// links array as per section 3.4.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Link {
    pub href: Url,
    pub rel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render: Option<Render>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

/// The render type as per section 4.7.
///
/// This is implemented as an `enum` to restrict the possible values.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Render {
    Image,
    Link,
}

/// Represents an element inside the item array as per section 3.1.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Item {
    pub href: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Data>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Captures the any of the three possible properties of an object in a
/// data array as per section 3.2.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Data {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Captures the any of the five possible properties of an object in a
/// queries array as per section 3.3.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Query {
    pub href: Url,
    pub rel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Data>>,
}

/// Represents the template object as per section 2.3.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Template {
    // should be present, so it might be optional, but also if it's
    // present it must contain one element, so skip it if None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Data>>,
}

/// Represents the error object as per section 2.2.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Error {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Collection {
    /// Create a new Collection+JSON structure
    pub fn new(uri: &str) -> Result<Self, ParseError> {
        Ok(Self {
            version: (),
            href: Url::parse(uri)?,
            links: None,
            items: None,
            queries: None,
            template: None,
            error: None,
        })
    }

    // TODO maybe provide some sort of builder pattern to allow easy building of
    // a Collection+JSON document.
}

mod fmt {
    use std::fmt::{Debug, Display, Error, Formatter, Result};
    use super::Collection;

    impl Debug for Collection {
        fn fmt(&self, f: &mut Formatter) -> Result {
            Display::fmt(
                &serde_json::to_string_pretty(&self)
                    // types inside the Collection shouldn't fail to serialize, but
                    // if it does return a `std::fmt::Error` instead of panic.
                    .map_err(|_| Error)?,
                f,
            )
        }
    }

    impl Display for Collection {
        fn fmt(&self, f: &mut Formatter) -> Result {
            Display::fmt(
                &serde_json::to_string(&self)
                    // types inside the Collection shouldn't fail to serialize, but
                    // if it does return a `std::fmt::Error` instead of panic.
                    .map_err(|_| Error)?,
                f,
            )
        }
    }
}

mod from_str {
    use std::str::FromStr;
    use super::Collection;

    impl FromStr for Collection {
        type Err = serde_json::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            serde_json::from_str(s)
        }
    }
}
