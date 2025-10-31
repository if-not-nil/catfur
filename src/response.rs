use std::{collections::HashMap, fmt::Write, pin::Pin};

use async_net::TcpStream;
use smol::io::AsyncWriteExt;

use crate::meta::{Headers, StatusCode};

pub enum Body {
    Text(String),
    Bytes(Vec<u8>),
    Stream(Pin<Box<dyn Fn(TcpStream) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>),
}

pub type ResultFuture = Pin<Box<dyn Future<Output = smol::io::Result<()>> + Send>>;
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
        F: Fn(TcpStream) -> VoidFuture + Send + Sync + 'static,
    {
        Self::new_with_body(Body::Stream(Box::pin(stream)))
            .header("Transfer-Encoding", "chunked")
            .header("Content-Type", "text/plain")
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
                let body_stream = body_stream;
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
