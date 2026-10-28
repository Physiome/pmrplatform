use axum::{
    body::{Body, Bytes, HttpBody},
    http::{HeaderName, HeaderValue, Request, Response, header},
};
use chrono::Datelike;
use http_body_util::BodyExt;
use lol_html::{
    html_content::{ContentType, Element},
    element, HtmlRewriter, Settings,
};
use std::{
    borrow::Cow,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// A layer that rewrites various anchors.
#[derive(Clone, Debug)]
pub struct RewriteHrefLayer {
    from: Cow<'static, str>,
    to: Cow<'static, str>,
}

impl RewriteHrefLayer {
    /// Create a new `RewriteHrefLayer`, with anchors to be rewritten `from` the original authority `to` the
    /// target authority.
    pub fn new(from: impl Into<Cow<'static, str>>, to: impl Into<Cow<'static, str>>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl<S> Layer<S> for RewriteHrefLayer {
    type Service = RewriteHref<S>;

    fn layer(&self, service: S) -> Self::Service {
        #[cfg(feature = "trace")]
        tracing::trace!("created RewriteHrefLayer from={:?} to={:?}", self.from, self.to);
        RewriteHref::new(self.from.clone(), self.to.clone(), service)
    }
}

/// A service that rewrites various anchors.
#[derive(Clone, Debug)]
pub struct RewriteHref<S> {
    from: Cow<'static, str>,
    to: Cow<'static, str>,
    inner: S,
}

impl<S> RewriteHref<S> {
    /// Create a new `RewriteHref` service, with anchors to be rewritten `from` the original authority `to` the
    /// target authority.
    pub fn new(from: impl Into<Cow<'static, str>>, to: impl Into<Cow<'static, str>>, inner: S) -> Self {
        let from = from.into();
        let to = to.into();
        #[cfg(feature = "trace")]
        tracing::trace!("created RewriteHref from={from} to={to}");
        Self { from, to, inner }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for RewriteHref<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + BodyExt + 'static,
    <ResBody as HttpBody>::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Because the inner service can panic until ready, we need to ensure we only
        // use the ready service.
        //
        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let from = self.from.to_string();
        let to = self.to.to_string();

        Box::pin(async move {
            let res = inner.call(req).await?;
            let (mut parts, body) = res.into_parts();
            let res = if let Some(content_type) = parts.headers.get(header::CONTENT_TYPE)
                && content_type.as_bytes().starts_with(b"text/html")
            {
                // While the response is streamed, the length will need to be recalculated so the
                // entirety of response will need to be buffered.
                let mut output = Vec::new();
                let bytes = body.collect()
                    .await
                    .map_err(|_| ())
                    .expect("body be fully streamed")
                    .to_bytes();
                let mut rewriter = HtmlRewriter::new(
                    Settings::new()
                        .append_element_content_handler(element!(
                            r#"
                            base,
                            link[rel="alternate"],
                            #portal-advanced-search a,
                            #tabs a,
                            a#user,
                            #breadcrumbs a,
                            #siteactions a,
                            a.btn,
                            .actionMenuHeader a,
                            .actionMenuContent a,
                            .contentViews a,
                            .portletWrapper a,
                            .summary a,
                            .workspace-manifest a
                            "#,
                            |el| {
                                if let Some(attr) = el.get_attribute("href") {
                                    let attr = attr.replace(&from, &to);
                                    el.set_attribute("href", &attr).unwrap();
                                }
                                Ok(())
                            }
                        ))
                        .append_element_content_handler(element!(
                            r#"style[type="text/css"]"#,
                            |el: &mut Element| {
                                el.set_inner_content("", ContentType::Html);
                                let start = el.source_location().bytes().end;
                                if let Some(handlers) = el.end_tag_handlers() {
                                    let bytes = bytes.clone();
                                    let from = from.clone();
                                    let to = to.clone();
                                    handlers.push(Box::new(move |tag| {
                                        let end = tag.source_location().bytes().start;
                                        tag.before(
                                            &str::from_utf8(&bytes[start..end])
                                                .expect("no invalid utf-8 within style tags")
                                                .replace(&from, &to),
                                            ContentType::Html,
                                        );
                                        Ok(())
                                    }));
                                }
                                Ok(())
                            }
                        ))
                        .append_element_content_handler(element!(
                            "script",
                            |el| {
                                if let Some(attr) = el.get_attribute("src") {
                                    let attr = attr.replace(&from, &to);
                                    el.set_attribute("src", &attr).unwrap();
                                }
                                Ok(())
                            }
                        ))
                        .append_element_content_handler(element!(
                            r"body, .workspace-manifest a",
                            |el| {
                                if let Some(attr) = el.get_attribute("data-base-url") {
                                    let attr = attr.replace(&from, &to);
                                    el.set_attribute("data-base-url", &attr).unwrap();
                                }
                                if let Some(attr) = el.get_attribute("data-portal-url") {
                                    let attr = attr.replace(&from, &to);
                                    el.set_attribute("data-portal-url", &attr).unwrap();
                                }
                                if let Some(attr) = el.get_attribute("data-src") {
                                    let attr = attr.replace(&from, &to);
                                    el.set_attribute("data-src", &attr).unwrap();
                                }
                                Ok(())
                            }
                        ))
                        .append_element_content_handler(element!(
                            r"form",
                            |el| {
                                if let Some(attr) = el.get_attribute("action") {
                                    let attr = attr.replace(&from, &to);
                                    el.set_attribute("action", &attr).unwrap();
                                }
                                Ok(())
                            }
                        ))
                        .append_element_content_handler(element!(
                            "input.clone-command",
                            |el| {
                                if let Some(attr) = el.get_attribute("value") {
                                    let attr = attr.replace(&from, &to);
                                    el.set_attribute("value", &attr).unwrap();
                                }
                                Ok(())
                            }
                        )),
                    |c: &[u8]| output.extend_from_slice(c),
                );
                rewriter.write(&bytes).unwrap();
                rewriter.end().unwrap();
                parts.headers.insert(header::CONTENT_LENGTH, output.len().into());
                Response::from_parts(parts, Body::from(output))
            } else {
                Response::from_parts(parts, Body::new(body))
            };

            Ok(res)
        })
    }
}

/// A layer that rewrites the template of the base site to the target site along with the anchors.
#[derive(Clone, Debug)]
pub enum RewriteTemplateLayer {
    Base,
    CellML(RewriteHrefLayer),
    FieldML(RewriteHrefLayer),
}

impl RewriteTemplateLayer {
    /// Create the base layer with minimal rewriting.
    pub fn base() -> Self {
        Self::Base
    }

    /// Create a new layer that will rewrite from the original template href to the default CellML Model
    /// Repository template and href (`https://models.cellml.org`).
    pub fn cellml(from: impl Into<Cow<'static, str>>) -> Self {
        Self::CellML(RewriteHrefLayer::new(from, "https://models.cellml.org/"))
    }

    /// Create a new layer that will rewrite from the original template href to the default FieldML Model
    /// Repository template and href (`https://models.fieldml.org`).
    pub fn fieldml(from: impl Into<Cow<'static, str>>) -> Self {
        Self::FieldML(RewriteHrefLayer::new(from, "https://models.fieldml.org/"))
    }
}

impl<S> Layer<S> for RewriteTemplateLayer {
    type Service = RewriteTemplate<S>;

    fn layer(&self, service: S) -> Self::Service {
        RewriteTemplate::new(&self, service)
    }
}

/// A service that rewrites the template fo the base site to the target site along with the anchors.
#[derive(Clone, Debug)]
pub enum RewriteTemplate<S> {
    Base(S),
    CellML(RewriteHref<S>),
    FieldML(RewriteHref<S>),
}

impl<S> RewriteTemplate<S> {
    pub fn new(layer: &RewriteTemplateLayer, inner: S) -> Self {
        match layer {
            RewriteTemplateLayer::Base => {
                Self::Base(inner)
            }
            RewriteTemplateLayer::CellML(layer) => {
                Self::CellML(RewriteHref::new(layer.from.clone(), layer.to.clone(), inner))
            }
            RewriteTemplateLayer::FieldML(layer) => {
                Self::FieldML(RewriteHref::new(layer.from.clone(), layer.to.clone(), inner))
            }
        }
    }

    pub fn base(inner: S) -> Self {
        Self::Base(inner)
    }

    pub fn cellml(from: impl Into<Cow<'static, str>>, inner: S) -> Self {
        Self::CellML(RewriteHref::new(from, "https://models.cellml.org/", inner))
    }

    pub fn fieldml(from: impl Into<Cow<'static, str>>, inner: S) -> Self {
        Self::FieldML(RewriteHref::new(from, "https://models.fieldml.org/", inner))
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for RewriteTemplate<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + BodyExt + 'static,
    <ResBody as HttpBody>::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        match self {
            Self::Base(inner) => inner.poll_ready(cx),
            Self::CellML(inner) => inner.poll_ready(cx),
            Self::FieldML(inner) => inner.poll_ready(cx),
        }
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let clone = self.clone();
        match clone {
            Self::Base(mut inner) => Box::pin(async move {
                let (mut parts, body) = inner.call(req)
                    .await?
                    .into_parts();
                let res = if let Some(content_type) = parts.headers.get(header::CONTENT_TYPE)
                    && content_type.as_bytes().starts_with(b"text/html")
                {
                    // While the response is streamed, the length will need to be recalculated so the
                    // entirety of response will need to be buffered.
                    let mut output = Vec::new();
                    let bytes = body.collect()
                        .await
                        .map_err(|_| ())
                        .expect("body be fully streamed")
                        .to_bytes();
                    let mut rewriter = HtmlRewriter::new(
                        Settings::new()
                            .append_element_content_handler(element!(
                                "#copyright",
                                |el| {
                                    let content = format!(
                                        "<acronym title=\"Copyright\">©</acronym> 2001-{} - \
                                        IUPS Physiome Project.",
                                        chrono::Utc::now().year(),
                                    );
                                    el.set_inner_content(&content, ContentType::Html);
                                    Ok(())
                                }
                            )),
                        |c: &[u8]| output.extend_from_slice(c),
                    );
                    rewriter.write(&bytes).unwrap();
                    rewriter.end().unwrap();
                    parts.headers.insert(header::CONTENT_LENGTH, output.len().into());
                    Response::from_parts(parts, Body::from(output))
                } else {
                    Response::from_parts(parts, Body::new(body))
                };
                Ok(res)
            }),
            Self::CellML(mut inner) => Box::pin(async move {
                let (mut parts, body) = inner.call(req)
                    .await?
                    .into_parts();

                let res = if let Some(content_type) = parts.headers.get(header::CONTENT_TYPE)
                    && content_type.as_bytes().starts_with(b"text/html")
                {
                    // While the response is streamed, the length will need to be recalculated so the
                    // entirety of response will need to be buffered.
                    let mut output = Vec::new();
                    let bytes = body.collect()
                        .await
                        .map_err(|_| ())
                        .expect("body be fully streamed")
                        .to_bytes();
                    let mut rewriter = HtmlRewriter::new(
                        Settings::new()
                            .append_element_content_handler(element!(
                                "#logo",
                                |el| {
                                    el.set_inner_content(
                                        "\n   \
                                        <a accesskey=\"1\" href=\"https://www.cellml.org/\"><img \
                                            alt=\"\" title=\"CellML\" \
                                            src=\"https://models.cellml.org/++theme++cellml.theme/assets/\
                                            images/logo-cellml.png\" />\
                                        </a>\n",
                                        ContentType::Html,
                                    );
                                    Ok(())
                                }
                            ))
                            .append_element_content_handler(element!(
                                "#PartOfLogo",
                                |el| {
                                    el.set_inner_content(
                                        "\n  \
                                        <a href=\"http://www.physiomeproject.org\"><img \
                                            alt=\"IUPS Physiome Project\" \
                                            title=\"cellml.org is part of the IUPS Physiome Project\" \
                                            src=\"https://models.cellml.org/++theme++cellml.theme/assets/\
                                                  images/logo-iups-physiome.png\" />\
                                        </a>\n",
                                        ContentType::Html,
                                    );
                                    Ok(())
                                }
                            ))
                            .append_element_content_handler(element!(
                                "div#portal-searchbox",
                                |el| {
                                    el.set_attribute("id", "model-searchbox")
                                        .expect("static attribute is valid");
                                    Ok(())
                                }
                            ))
                            .append_element_content_handler(element!(
                                "form#searchGadget_form",
                                |el| {
                                    el.start_tag().replace("\
                                        <form name=\"searchmodelform\" \
                                            action=\"https://models.cellml.org/cellml_search\" \
                                            class=\"searchModel\" id=\"searchmodelform\">",
                                        ContentType::Html,
                                    );
                                    Ok(())
                                }
                            ))
                            .append_element_content_handler(element!(
                                "form#searchGadget_form .LSBox",
                                |el| {
                                    el.replace("\
                                        <label for=\"SearchCellML\" class=\"hiddenStructure\">\
                                            Search Models\
                                        </label>\n\n        \
                                        <input name=\"SearchableText\" \
                                            id=\"liveSearchCellML\" \
                                            type=\"text\" \
                                            size=\"18\" \
                                            title=\"Search\" \
                                            placeholder=\"Search Models\" \
                                            accesskey=\"4\" \
                                            class=\"inputLabel\" />\
                                        ",
                                        ContentType::Html,
                                    );
                                    Ok(())
                                }
                            ))
                            .append_element_content_handler(element!(
                                "div#portal-advanced-search",
                                |el| {
                                    el.remove();
                                    Ok(())
                                }
                            ))
                            .append_element_content_handler(element!(
                                "#copyright",
                                |el| {
                                    let content = format!(
                                        "<acronym title=\"Copyright\">©</acronym> 2001-{} - \
                                        The CellML Project.",
                                        chrono::Utc::now().year(),
                                    );
                                    el.set_inner_content(&content, ContentType::Html);
                                    Ok(())
                                }
                            )),
                        |c: &[u8]| output.extend_from_slice(c),
                    );
                    rewriter.write(&bytes).unwrap();
                    rewriter.end().unwrap();
                    parts.headers.insert(header::CONTENT_LENGTH, output.len().into());
                    Response::from_parts(parts, Body::from(output))
                } else {
                    Response::from_parts(parts, Body::new(body))
                };
                Ok(res)
            }),
            Self::FieldML(mut inner) => Box::pin(async move {
                let (mut parts, body) = inner.call(req)
                    .await?
                    .into_parts();
                let res = if let Some(content_type) = parts.headers.get(header::CONTENT_TYPE)
                    && content_type.as_bytes().starts_with(b"text/html")
                {
                    // While the response is streamed, the length will need to be recalculated so the
                    // entirety of response will need to be buffered.
                    let mut output = Vec::new();
                    let bytes = body.collect()
                        .await
                        .map_err(|_| ())
                        .expect("body be fully streamed")
                        .to_bytes();
                    let mut rewriter = HtmlRewriter::new(
                        Settings::new()
                            .append_element_content_handler(element!(
                                "#logo",
                                |el| {
                                    el.set_inner_content(
                                    "\n  \
                                    <a accesskey=\"1\" href=\"https://physiomeproject.org/\"><img \
                                    alt=\"\" title=\"IUPS Physiome Project\" \
                                    src=\"https://models.physiomeproject.org/++theme++cellml.theme/assets/\
                                    images/logo-physiome.png\" /></a>\n",
                                        ContentType::Html,
                                    );
                                    Ok(())
                                }
                            ))
                            .append_element_content_handler(element!(
                                "#copyright",
                                |el| {
                                    let content = format!(
                                        "<acronym title=\"Copyright\">©</acronym> 2001-{} - \
                                        IUPS Physiome Project.",
                                        chrono::Utc::now().year(),
                                    );
                                    el.set_inner_content(&content, ContentType::Html);
                                    Ok(())
                                }
                            )),
                        |c: &[u8]| output.extend_from_slice(c),
                    );
                    rewriter.write(&bytes).unwrap();
                    rewriter.end().unwrap();
                    parts.headers.insert(header::CONTENT_LENGTH, output.len().into());
                    Response::from_parts(parts, Body::from(output))
                } else {
                    Response::from_parts(parts, Body::new(body))
                };
                Ok(res)
            }),
        }
    }
}

/// The rewrite layer for all of PMR2.
///
/// Using this will set up all the underlying services as intended for the production service.
#[derive(Clone, Debug)]
pub struct RewritePmr2Layer {
    from: Cow<'static, str>,
}

impl RewritePmr2Layer {
    /// Sets up the layer for rewriting `from` uri.
    pub fn new(from: impl Into<Cow<'static, str>>) -> Self {
        Self {
            from: from.into(),
        }
    }
}

impl<S> Layer<S> for RewritePmr2Layer
where
    S: Clone,
{
    type Service = RewritePmr2<S>;

    fn layer(&self, service: S) -> Self::Service {
        RewritePmr2::new(self.from.clone(), service)
    }
}

/// A service that provides rewriting for PMR2.
#[derive(Clone, Debug)]
pub struct RewritePmr2<S> {
    base: RewriteTemplate<S>,
    cellml: RewriteTemplate<S>,
    fieldml: RewriteTemplate<S>,
}

impl<S> RewritePmr2<S> {
    /// Create the service to rewrite the cached content of PMR2.
    pub fn new(from: impl Into<Cow<'static, str>>, inner: S) -> Self
    where
        S: Clone,
    {
        let from = from.into();
        Self {
            base: RewriteTemplate::base(inner.clone()),
            cellml: RewriteTemplate::cellml(from.clone(), inner.clone()),
            fieldml: RewriteTemplate::fieldml(from.clone(), inner.clone()),
        }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for RewritePmr2<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + BodyExt + 'static,
    <ResBody as HttpBody>::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        if let Poll::Pending = self.fieldml.poll_ready(cx) {
            Poll::Pending
        } else if let Poll::Pending = self.cellml.poll_ready(cx) {
            Poll::Pending
        } else {
            self.base.poll_ready(cx)
        }
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        match req.headers().get(HeaderName::from_static("x-forwarded-host")) {
            Some(s) if s == HeaderValue::from_static("models.cellml.org") => {
                let clone = self.cellml.clone();
                let mut inner = std::mem::replace(&mut self.cellml, clone);
                Box::pin(async move {
                    inner.call(req).await
                })
            }
            Some(s) if s == HeaderValue::from_static("models.fieldml.org") => {
                let clone = self.fieldml.clone();
                let mut inner = std::mem::replace(&mut self.fieldml, clone);
                Box::pin(async move {
                    inner.call(req).await
                })
            }
            _ => {
                let clone = self.base.clone();
                let mut inner = std::mem::replace(&mut self.base, clone);
                Box::pin(async move {
                    inner.call(req).await
                })
            }
        }
    }
}
