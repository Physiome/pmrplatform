use axum::http::{Request, Response, StatusCode, header};
use std::{
    borrow::Cow,
    convert::Infallible,
    future::Future,
    pin::Pin,
    process::Stdio,
    task::{Context, Poll},
};
use tokio::process::{ChildStdout, Command};
use tower::Service;
use tower_http::services::fs::AsyncReadBody;

#[derive(Clone, Debug)]
pub struct CommandRunner {
    cmd: Cow<'static, str>,
}

impl CommandRunner {
    pub fn new(cmd: Cow<'static, str>) -> Self {
        Self { cmd }
    }
}

impl<ReqBody> Service<Request<ReqBody>> for CommandRunner
where
    ReqBody: Send + 'static,
{
    // TODO figure out how to merge stderr/stdio?
    // TODO maybe also figure out how to type-erase the response body type.
    type Response = Response<AsyncReadBody<ChildStdout>>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response<AsyncReadBody<ChildStdout>>, Infallible>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<ReqBody>) -> Self::Future {
        let mut child = Command::new(&*self.cmd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().expect("failed to get subprcess stdout");
        tokio::spawn(async move { child.wait().await.unwrap() });
        let body = AsyncReadBody::with_capacity(stdout, 18);
        Box::pin(async {
            let resp = Response::builder()
                .header(header::CONTENT_TYPE, "text/plain")
                .status(StatusCode::OK)
                .body(body)
                .unwrap();
            Ok(resp)
        })
    }
}
