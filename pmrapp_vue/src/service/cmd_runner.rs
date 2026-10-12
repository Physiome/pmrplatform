use axum::{
    body::Body,
    extract::{FromRequestParts, Path},
    http::{Method, Request, Response, StatusCode, header},
    Router,
};
use std::{
    borrow::Cow,
    convert::Infallible,
    future::Future,
    pin::Pin,
    process::Stdio,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader, ReadBuf},
    process::Command,
};
use tower::Service;
use tower_http::services::fs::AsyncReadBody;

#[derive(Debug, Default)]
struct OutcomeInner {
    stdout: Vec<u8>,
    status_code: Option<Option<i32>>,
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    inner: Arc<RwLock<OutcomeInner>>,
    pos: usize,
}

impl Outcome {
    pub fn new() -> Self {
        Self::default()
    }
}

// Technically this could implement Body, but AsyncRead is more generic for more flexible usage locations.
impl AsyncRead for Outcome {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let inner = self.inner.clone();
        let inner = inner.read().unwrap();
        let amt = std::cmp::min(inner.stdout.len() - self.pos, buf.remaining());
        let slice = &inner.stdout[self.pos..self.pos + amt];
        buf.put_slice(slice);
        self.pos = self.pos + amt;
        match inner.status_code {
            None => {
                if slice.len() > 0 {
                    Poll::Ready(Ok(()))
                } else {
                    let waker = cx.waker().clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        waker.wake_by_ref();
                    });
                    Poll::Pending
                }
            }
            _ => {
                Poll::Ready(Ok(()))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CommandRunner {
    bearer: Option<Cow<'static, str>>,
    prefix: Cow<'static, str>,
    cmd: Cow<'static, str>,
    outcomes: Arc<RwLock<Vec<Outcome>>>,
}

impl CommandRunner {
    pub fn new(
        cmd: Cow<'static, str>,
    ) -> Self {
        CommandRunner {
            bearer: None,
            prefix: "/".into(),
            cmd,
            outcomes: Default::default(),
        }
    }

    pub fn bearer(mut self, bearer: Cow<'static, str>) -> Self {
        self.bearer = Some(bearer);
        self
    }

    pub fn prefix(mut self, prefix: Cow<'static, str>) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn setup_router(self, router: Router) -> Router {
        let prefix = &self.prefix;
        router
            .route_service(&format!("{prefix}"), self.clone())
            .route_service(&format!("{prefix}{{idx}}"), self)
    }

    // Return the 1-indexed item.
    async fn read(&self, idx: u64) -> Option<Outcome> {
        let outcomes = self.outcomes.read().unwrap();
        if let Some(outcome) = outcomes.get((idx - 1) as usize) {
            Some(outcome.clone())
        } else {
            None
        }
    }

    async fn spawn(&self) -> usize {
        let outcome = Outcome::new();
        let idx = {
            let mut outcomes = self.outcomes.write().unwrap();
            outcomes.push(outcome.clone());
            outcomes.len()
        };

        // FIXME - this should be a HTTP 500 error.
        let mut child = Command::new(&*self.cmd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().expect("failed to get subprcess stdout");
        tokio::spawn({
            let outcome = outcome.clone();
            async move {
                let status = child.wait().await.unwrap();
                let mut outcome = outcome.inner.write().unwrap();
                (*outcome).status_code = Some(status.code());
            }
        });
        tokio::spawn({
            let outcome = outcome.clone();
            let mut reader = BufReader::new(stdout).split(b'\n');
            async move {
                while let Some(line) = reader.next_segment().await.unwrap() {
                    let mut outcome = outcome.inner.write().unwrap();
                    outcome.stdout.extend(line);
                    outcome.stdout.push(b'\n');
                }
            }
        });
        idx
    }
}

impl<ReqBody> Service<Request<ReqBody>> for CommandRunner
where
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let (mut parts, _body) = req.into_parts();
        let runner = self.clone();
        let prefix = self.prefix.clone();
        let bearer = self.bearer.clone().map(|bearer| {
            header::HeaderValue::from_str(&format!("Bearer {}", bearer)).unwrap()
        });
        Box::pin(async move {
            let mut resp = Response::builder()
                .header(header::CONTENT_TYPE, "text/plain");
            if bearer.is_some() && parts.headers.get(header::AUTHORIZATION) != bearer.as_ref() {
                return Ok(resp.status(StatusCode::FORBIDDEN).body(String::new().into()).unwrap());
            }
            let body = if let Ok(Path(idx)) = Path::<u64>::from_request_parts(&mut parts, &()).await {
                if let Some(outcome) = runner.read(idx).await {
                    resp = resp.status(StatusCode::OK);
                    Some(Body::new(AsyncReadBody::with_capacity(outcome, 1)))
                } else {
                    resp = resp.status(StatusCode::NOT_FOUND);
                    None
                }
            } else if parts.method == Method::POST {
                let idx = runner.spawn().await;
                resp = resp.header(header::LOCATION, format!("{prefix}{idx}"))
                    .status(StatusCode::SEE_OTHER);
                None
            } else {
                resp = resp.status(StatusCode::NOT_FOUND);
                None
            };
            let resp = resp
                .body(body.unwrap_or_else(|| String::new().into()))
                .unwrap();
            Ok(resp)
        })
    }
}
