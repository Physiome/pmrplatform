use crate::{Collection, Link, ParseError, Render};

#[cfg(doc)]
use url::Url;

// TODO a BoundedLinkBuilder<'a> that links with an underlying collection
pub struct LinkBuilder {
    pub href: String,
    pub rel: String,
    pub name: Option<String>,
    pub render: Option<Render>,
    pub prompt: Option<String>,
}

impl LinkBuilder {
    /// The LinkBuilder
    ///
    /// Provide `href` and `rel`.  The `href` is simply a string in this case,
    /// as the [`Url`] type requires an absolute path, while the builder allows
    /// a relative link which may be joined with the final collection document.
    pub fn new(href: String, rel: String) -> Self {
        Self {
            href,
            rel,
            name: None,
            render: None,
            prompt: None,
        }
    }

    /// Sets value for name
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets value for render
    pub fn render(mut self, render: Render) -> Self {
        self.render = Some(render);
        self
    }

    /// Sets value for prompt
    pub fn prompt(mut self, prompt: String) -> Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn build_with(self, collection: &Collection) -> Result<Link, ParseError> {
        let href = collection.href.join(&self.href)?;
        let Self {
            rel,
            name,
            render,
            prompt,
            ..
        } = self;
        Ok(Link {
            href,
            rel,
            name,
            render,
            prompt,
        })
    }
}
