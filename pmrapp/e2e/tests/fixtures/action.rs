use super::{find, world::HOST};
use anyhow::Result;
use fantoccini::Client;
use std::result::Result::Ok;

pub async fn goto_path(client: &Client, path: &str) -> Result<()> {
    let uri = format!("{}{}", HOST, path);
    client.goto(&uri).await?;
    Ok(())
}

pub async fn click_link_by_text(client: &Client, text: &str) -> Result<()> {
    let link = find::link_with_text(client, text).await?;
    link.click().await?;
    Ok(())
}
