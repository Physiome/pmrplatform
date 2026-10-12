use axum::{
    extract::{FromRequestParts, Path},
    http::{Request, Response, StatusCode, header},
};
use std::{
    borrow::Cow,
    convert::Infallible,
    future::Future,
    io::{Cursor, Seek as _, SeekFrom, Write as _},
    pin::Pin,
    process::Stdio,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::RwLock,
};
use tower::Service;
use tower_http::services::fs::AsyncReadBody;

#[derive(Debug, Default)]
struct OutcomeInner {
    stdout: Cursor<Vec<u8>>,
    status_code: Option<Option<i32>>,
}

#[derive(Clone, Debug, Default)]
pub struct Outcome(Arc<RwLock<OutcomeInner>>);

impl Outcome {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(OutcomeInner::default())))
    }

    pub async fn cursor(&self) -> Cursor<Vec<u8>> {
        let mut cursor = self.0.read().await.stdout.clone();
        cursor.seek(SeekFrom::Start(0))
            .expect("seek shouldn't fail for a Vec");
        cursor
    }
}

#[derive(Clone, Debug)]
pub struct CommandRunner {
    cmd: Cow<'static, str>,
    outcomes: Arc<RwLock<Vec<Outcome>>>,
}

impl CommandRunner {
    pub fn new(cmd: Cow<'static, str>) -> Self {
        CommandRunner {
            cmd,
            outcomes: Default::default(),
        }
    }

    // Return the 1-indexed item.
    async fn read(&self, idx: u64) -> Option<Outcome> {
        let outcomes = self.outcomes.read().await;
        if let Some(outcome) = outcomes.get((idx - 1) as usize) {
            Some(outcome.clone())
        } else {
            None
        }
    }

    async fn spawn(&self) -> usize {
        let outcome = Outcome::new();
        let idx = {
            let mut outcomes = self.outcomes.write().await;
            outcomes.push(outcome.clone());
            outcomes.len()
        };

        // TODO figure out how to merge stderr/stdio?
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
                let mut outcome = outcome.0.write().await;
                (*outcome).status_code = Some(status.code());
            }
        });
        tokio::spawn({
            let outcome = outcome.clone();
            let mut reader = BufReader::new(stdout).split(b'\n');
            async move {
                while let Some(line) = reader.next_segment().await.unwrap() {
                    let mut outcome = outcome.0.write().await;
                    outcome.stdout.write(&line).unwrap();
                    outcome.stdout.write(&[b'\n']).unwrap();
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
    // TODO maybe also figure out how to type-erase the response body type.
    type Response = Response<AsyncReadBody<Cursor<Vec<u8>>>>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response<AsyncReadBody<Cursor<Vec<u8>>>>, Infallible>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let (mut parts, _body) = req.into_parts();
        let runner = self.clone();
        Box::pin(async move {
            let stream = if let Ok(Path(idx)) = Path::<u64>::from_request_parts(&mut parts, &()).await {
                if let Some(outcome) = runner.read(idx).await {
                    AsyncReadBody::with_capacity(outcome.cursor().await, 1)
                } else {
                    AsyncReadBody::with_capacity(Cursor::new(Vec::new()), 1)
                }
            } else {
                let idx = runner.spawn().await;
                AsyncReadBody::with_capacity(Cursor::new(format!("{idx}").into()), 1)
            };
            let resp = Response::builder()
                .header(header::CONTENT_TYPE, "text/plain")
                .status(StatusCode::OK)
                .body(stream)
                .unwrap();

            Ok(resp)
        })
    }
}
