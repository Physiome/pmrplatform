use crate::fixtures::{action, world::AppWorld};
use anyhow::{Ok, Result};
use cucumber::{given, when, gherkin::Step};

#[given("I see the app")]
#[given("I open the app")]
#[when("I open the app")]
async fn i_open_the_app(world: &mut AppWorld) -> Result<()> {
    let client = &world.client;
    action::goto_path(client, "").await?;
    Ok(())
}

#[given(regex = r"^I select the link (.*)$")]
#[when(regex = r"^I selct the link (.*)$")]
async fn i_select_the_link(world: &mut AppWorld, text: String) -> Result<()> {
    let client = &world.client;
    action::click_link_by_text(client, &text).await?;
    Ok(())
}

#[given(regex = r"^I enter the path (.*)$")]
#[when(regex = r"^I enter the path (.*)$")]
async fn i_enter_the_path(world: &mut AppWorld, text: String) -> Result<()> {
    let client = &world.client;
    action::click_link_by_text(client, &text).await?;
    Ok(())
}

#[given(regex = "^I (refresh|reload) the (browser|page)$")]
#[when(regex = "^I (refresh|reload) the (browser|page)$")]
async fn i_refresh_the_browser(world: &mut AppWorld) -> Result<()> {
    let client = &world.client;
    client.refresh().await?;
    Ok(())
}

#[given(expr = "I select the following links")]
#[when(expr = "I select the following links")]
async fn i_select_the_following_links(
    world: &mut AppWorld,
    step: &Step,
) -> Result<()> {
    let client = &world.client;

    if let Some(table) = step.table.as_ref() {
        for row in table.rows.iter() {
            action::click_link_by_text(client, &row[0]).await?;
        }
    }

    Ok(())
}
