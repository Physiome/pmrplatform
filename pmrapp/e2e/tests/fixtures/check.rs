use crate::fixtures::find;
use anyhow::{Error, Ok, Result};
use fantoccini::Client;
use pretty_assertions::assert_eq;

// pub async fn text_with_id_is(client: &Client, expected: &str, id: &str) -> Result<()> {
//     let actual = find::text_with_id(client, id).await?;
//     assert_eq!(&actual, expected);
//     Ok(())
// }

pub async fn text_with_selector_is(client: &Client, expected: &str, selector: &str) -> Result<()> {
    let actual = find::text_with_selector(client, selector).await?;
    assert_eq!(&actual, expected);
    Ok(())
}

pub async fn link_exists(client: &Client, text: &str) -> Result<()> {
    find::link_with_text(client, text).await?;
    Ok(())
}

pub async fn link_text_is_aria_current(client: &Client, text: &str) -> Result<()> {
    let link = find::link_with_text(client, text).await?;
    link.attr("aria-current").await?
        .ok_or_else(|| Error::msg(format!("aria-current missing for {text}")))?;
    Ok(())
}
