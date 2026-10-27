use axum::{
    body::{Body, Bytes, HttpBody},
    http::{Request, Response, header},
};
use http_body_util::BodyExt;
use lol_html::{
    html_content::{ContentType, Element},
    element, HtmlRewriter, Settings,
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// A layer that rewrites various anchors.
#[derive(Clone, Debug)]
pub struct RewriteHrefLayer {
    from: String,
    to: String,
}

impl RewriteHrefLayer {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl<S> Layer<S> for RewriteHrefLayer {
    type Service = RewriteHref<S>;

    fn layer(&self, service: S) -> Self::Service {
        RewriteHref::new(&self.from, &self.to, service)
    }
}

/// A service that rewrites various anchors.
#[derive(Clone, Debug)]
pub struct RewriteHref<S> {
    from: String,
    to: String,
    inner: S,
}

impl<S> RewriteHref<S> {
    pub fn new(from: impl Into<String>, to: impl Into<String>, inner: S) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            inner,
        }
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
        let from = self.from.clone();
        let to = self.to.clone();

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
                        ))
                        ,
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
