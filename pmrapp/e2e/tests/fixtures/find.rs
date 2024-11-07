use anyhow::{Ok, Result};
use fantoccini::{elements::Element, Client, Locator};

// pub async fn text_with_id(client: &Client, id: &str) -> Result<String> {
//     let element = client
//         .wait()
//         .for_element(Locator::Id(id))
//         .await
//         .expect(format!("loaded message not found by id `{id}`").as_str());
//     let text = element.text().await?;
//     Ok(text)
// }

pub async fn text_with_selector(client: &Client, selector: &str) -> Result<String> {
    let element = client
        .wait()
        .for_element(Locator::Css(selector))
        .await
        .expect(format!("loaded message not found by selector `{selector}`").as_str());
    let text = element.text().await?;
    Ok(text)
}

pub async fn link_with_text(client: &Client, text: &str) -> Result<Element> {
    let link = client
        .wait()
        .for_element(Locator::LinkText(text))
        .await
        .expect(format!("Link with text `{text}` not found").as_str());
    Ok(link)
}
