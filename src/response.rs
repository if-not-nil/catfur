use std::{collections::HashMap, fmt::Write, pin::Pin};

use async_net::TcpStream;
use smol::io::AsyncWriteExt;

use crate::meta::{Headers, StatusCode};

pub enum Body {
    Text(String),
    Bytes(Vec<u8>),
    Stream(Pin<Box<dyn Fn(TcpStream) -> ResultFuture + Send + Sync>>),
}
pub type ResultFuture = Pin<Box<dyn Future<Output = std::io::Result<()>> + Send>>;
pub type VoidFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(arg0) => f.debug_tuple("Text").field(arg0).finish(),
            Self::Bytes(arg0) => f.debug_tuple("Bytes").field(arg0).finish(),
            Body::Stream(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Headers,
    pub body: Option<Body>,
}

impl Response {
    // base
    pub fn text(s: impl Into<String>) -> Self {
        Self::new_with_body(Body::Text(s.into())).header("Content-Type", "text/plain")
    }

    pub fn html(s: impl Into<String>) -> Self {
        Self::new_with_body(Body::Text(s.into())).header("Content-Type", "text/html")
    }

    pub fn json(s: impl Into<String>) -> Self {
        Self::new_with_body(Body::Text(s.into())).header("Content-Type", "application/json")
    }

    pub fn bytes(bytes: Vec<u8>, content_type: &str) -> Self {
        Self::new_with_body(Body::Bytes(bytes)).header("Content-Type", content_type)
    }

    pub fn stream<F>(stream: F) -> Self
    where
        F: Fn(TcpStream) -> ResultFuture + Send + Sync + 'static,
    {
        Self::new_with_body(Body::Stream(Box::pin(stream)))
            .header("Transfer-Encoding", "chunked")
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Content-Type", "text/plain")
    }

    pub fn sse<F, Fut>(f: F) -> Response
    where
        F: Fn(SseSink) -> Fut + Send + 'static + std::marker::Sync,
        Fut: Future<Output = std::io::Result<()>> + Send + 'static,
    {
        Response::stream(move |stream| {
            let sink = SseSink { stream };
            Box::pin(f(sink))
        })
    }

    pub fn empty() -> Self {
        Self {
            status: StatusCode::NoContent,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn error(status: StatusCode) -> Self {
        Self::text(status.as_str()).status(status)
    }

    // modifiers

    #[must_use]
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    #[must_use]
    pub fn header(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.headers.insert(key.into(), val.into());
        self
    }

    // internal

    fn new_with_body(body: Body) -> Self {
        Self {
            status: StatusCode::Ok,
            headers: HashMap::from([("Connection".into(), "keep-alive".into())]),
            body: Some(body),
        }
    }

    pub fn finalize(mut self) -> Self {
        if let Some(Body::Text(s)) = &self.body {
            self.headers
                .entry("Content-Length".into())
                .or_insert(s.len().to_string());
        } else if let Some(Body::Bytes(b)) = &self.body {
            self.headers
                .entry("Content-Length".into())
                .or_insert(b.len().to_string());
        }
        self
    }

    pub async fn write_to(&self, mut stream: TcpStream) -> std::io::Result<()> {
        let mut header_str = format!("HTTP/1.1 {}\r\n", self.status.as_str());
        for (k, v) in &self.headers {
            write!(&mut header_str, "{}: {}\r\n", k, v).unwrap();
        }
        header_str.push_str("\r\n");
        stream.write_all(header_str.as_bytes()).await?;
        stream.flush().await?;

        match &self.body {
            Some(Body::Text(s)) => stream.write_all(s.as_bytes()).await?,
            Some(Body::Bytes(b)) => stream.write_all(b).await?,
            Some(Body::Stream(body_stream)) => {
                let stream = stream.clone();
                smol::spawn(body_stream(stream)).detach();
            }
            None => {}
        }

        stream.flush().await
    }
}

impl Default for Response {
    fn default() -> Self {
        Response::text("")
    }
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        Response::text(value)
    }
}

impl From<&str> for Response {
    fn from(value: &str) -> Self {
        Response::text(value)
    }
}

pub struct SseSink {
    stream: smol::net::TcpStream,
}

impl SseSink {
    pub async fn send(&mut self, data: &str) -> std::io::Result<()> {
        // it needs a double newline at the end
        let payload = format!("data: {}\n\n", data);
        self.send_chunk(payload.as_bytes()).await
    }

    pub async fn send_event(&mut self, name: &str, data: impl Into<String>) -> std::io::Result<()> {
        let payload = format!("event: {}\ndata: {}\n\n", name, data.into());
        self.send_chunk(payload.as_bytes()).await
    }

    // internal send bytes and wrap them nicely
    async fn send_chunk(&mut self, data: &[u8]) -> std::io::Result<()> {
        let header = format!("{:X}\r\n", data.len());
        self.stream.write_all(header.as_bytes()).await?;
        self.stream.write_all(data).await?;
        self.stream.write_all(b"\r\n").await?;
        self.stream.flush().await
    }
}
