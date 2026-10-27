use lol_html::{
    errors::RewritingError,
    html_content::ContentType,
    element, HtmlRewriter, Settings,
};

static TEMPLATE: &str = include_str!("template.html");

pub(crate) fn message(msg: &str) -> Result<Vec<u8>, RewritingError> {
    let mut output = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings::new()
            .append_element_content_handler(element!("div#content-core", |el| {
                el.set_inner_content(msg, ContentType::Html);
                Ok(())
            })),
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(TEMPLATE.as_bytes())?;
    // .expect("input shouldn't fail to parse and processing shouldn't fail here");
    rewriter.end()?;
    Ok(output)
}
