use axum::{
    body::{Body, HttpBody},
    http::{
        header::HeaderValue,
        uri::{Authority, Parts, Scheme},
        Request, Response, StatusCode, Uri, header,
        Error as HttpError,
    },
};
use cookie::Cookie;
use hyper_util::client::legacy::{
    connect::HttpConnector,
    Client,
};
use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Semaphore;
use tower::Service;

/// Specify how the request should be processed.
#[derive(Debug)]
pub enum Strategy {
    /// Use the file at this relative path to cache/load the request.
    Cache,
    /// Acquire the semaphore before passing through the request to the underlying service.
    Semaphore,
    /// Pass through the request to the underlying service unhindered.
    Passthru,
    /// Assume the resource is not found.
    NotFound,
}

#[derive(Debug)]
pub struct Proxy<B> {
    client: Arc<Client<HttpConnector, B>>,
    authority: Authority,
    semaphore: Arc<Semaphore>,
    uri_base: &'static str,
}

impl<B> Clone for Proxy<B> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            authority: self.authority.clone(),
            semaphore: self.semaphore.clone(),
            uri_base: self.uri_base,
        }
    }
}

impl<B> Proxy<B>
where
    B: HttpBody + Send + std::fmt::Debug,
    <B as HttpBody>::Data: Send,
{
    /// Create a proxy to a PMR2 instance served at the provided base `Uri`.
    ///
    /// This implementation assumes the provided `Uri` has the scheme of `http` and that no query is
    /// provided as that will be ignored.
    pub fn new<T>(base: T, permits: usize) -> Self
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: std::fmt::Debug,
    {
        let Parts { authority, path_and_query, .. } = base
            .try_into()
            .expect("provided base uri must be valid")
            .into_parts();
        let authority = authority
            .expect("authority must be provided by the base uri");
        // Assume the service exists for the duration of the program, leaking this for convenience.
        let uri_base = Box::leak(
            path_and_query
                .expect("path must be provided by the provided base uri")
                .path()
                .trim_end_matches('/')
                .to_owned()
                .into_boxed_str()
        );
        let client = Arc::new(Client::builder(hyper_util::rt::TokioExecutor::new()).build_http());
        let semaphore = Arc::new(Semaphore::new(permits));
        Self { client, authority, semaphore, uri_base }
    }

    pub fn strategize(&self, request: &Request<B>) -> Strategy {
        if let Some(Ok(cookie)) = request
            .headers()
            .get(header::COOKIE)
            .map(HeaderValue::to_str)
        {
            if Cookie::split_parse(cookie).any(|r| r.map(|c| c.name() == "__ac").unwrap_or(false)) {
                // TODO Should follow this up with our own session management.
                return Strategy::Passthru;
            }
        }
        // Any other requests assumed to require holding the semaphore.
        Strategy::Semaphore
    }

    pub fn patch_request(&self, request: &mut Request<B>) -> Result<(), HttpError> {
        let request_uri = request.uri();
        let request_path = request_uri.path();
        let request_query = request_uri.query();
        let mut target = String::with_capacity(self.uri_base.len()
            + request_path.len()
            + request_query
                .map(|q| q.len() + 1)
                .unwrap_or(0)
        );
        target.push_str(self.uri_base);
        target.push_str(request_path);
        if let Some(request_query) = request_query {
            target.push('?');
            target.push_str(request_query);
        }

        let uri = Uri::builder()
            .scheme(Scheme::HTTP)
            .authority(self.authority.clone())
            .path_and_query(target)
            .build()?;

        *request.uri_mut() = uri;
        Ok(())
    }
}

impl<ReqBody> Service<Request<ReqBody>> for Proxy<ReqBody>
where
    ReqBody: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <ReqBody as HttpBody>::Data: Send,
    <ReqBody as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let service = self.clone();
        Box::pin(async move {
            let strategy = service.strategize(&req);
            #[cfg(feature = "trace")]
            tracing::debug!("strategy={strategy:?}");
            // FIXME An error here should trigger a 503 response and the appropriate log.
            let _ = service.patch_request(&mut req);
            let resp = service.client.as_ref().request(req).await
                .map(|resp| {
                    let (parts, incoming) = resp.into_parts();
                    let body = Body::new(incoming);
                    Response::from_parts(parts, body)
                })
                .unwrap_or_else(|_| {
                    // TODO Build a themed response off this.
                    Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from("<html><body><h1>503 Servce Unavailable</h1></body></html>"))
                        .expect("statically constructed response shouldn't error")
                });
            Ok(resp)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn proxy_patch_request() {
        let service = Proxy::new("http://example.com/pmr/");
        let mut req = Request::builder()
            .method(Method::POST)
            .uri("/workspace/123")
            .body(Body::empty())
            .expect("valid request");
        service.patch_request(&mut req)
            .expect("request should have been successfully patched");
        assert_eq!(req.uri(), "http://example.com/pmr/workspace/123");
    }
}
