use crate::fixtures::{check, world::AppWorld};
use anyhow::{Ok, Result};
use cucumber::{then, gherkin::Step};

#[then(regex = r"^I see the page title is (.*)$")]
async fn i_see_the_page_title_is(
    world: &mut AppWorld,
    text: String,
) -> Result<()> {
    let client = &world.client;
    check::text_with_selector_is(client, "h1", &text).await?;
    Ok(())
}

#[then(regex = r"^I see the link labeled (.*) is highlighted$")]
async fn i_see_the_following_link_highlighted(
    world: &mut AppWorld,
    text: String,
) -> Result<()> {
    let client = &world.client;
    check::link_text_is_aria_current(client, &text).await?;
    Ok(())
}

#[then(regex = r"^I can find the link (.*)$")]
async fn i_see_the_following_link(
    world: &mut AppWorld,
    text: String,
) -> Result<()> {
    let client = &world.client;
    check::link_exists(client, &text).await?;
    Ok(())
}

#[then(expr = "I see the following navigational links")]
#[then(expr = "I can find the following links")]
async fn i_see_the_following_links(
    world: &mut AppWorld,
    step: &Step,
) -> Result<()> {
    let client = &world.client;
    if let Some(table) = step.table.as_ref() {
        for row in table.rows.iter() {
            check::link_exists(client, &row[0]).await?;
        }
    }

    Ok(())
}
