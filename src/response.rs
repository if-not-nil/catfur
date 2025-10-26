use std::{collections::HashMap, io::Write, net::TcpStream};

use crate::{
    builders::ResponseBuilder,
    meta::{Headers, StatusCode},
};

pub enum Body {
    Text(String),
    Bytes(Vec<u8>),
    Stream(Box<dyn Fn(&mut TcpStream) -> std::io::Result<()> + Send + Sync>),
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(arg0) => f.debug_tuple("Text").field(arg0).finish(),
            Self::Bytes(arg0) => f.debug_tuple("Bytes").field(arg0).finish(),
            Self::Stream(_) => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Headers,
    pub body: Option<Body>,
}

// not sure how i feel about having a weird response-responsebuilder relationship
impl Response {
    pub fn new() -> Self {
        let mut s = Self {
            status: StatusCode::Ok,
            headers: HashMap::new(),
            body: None,
        };
        s.set_header("Connection", "keep-alive");
        s
    }

    pub fn new_sse<F>(streamer: F) -> Response
    where
        F: Fn(&mut TcpStream) -> std::io::Result<()> + Send + Sync + 'static,
    {
        let mut headers = Headers::new();
        headers.insert("Content-Type".into(), "text/event-stream".into());
        headers.insert("Cache-Control".into(), "no-cache".into());
        headers.insert("Connection".into(), "keep-alive".into());

        Response {
            status: StatusCode::Ok,
            headers,
            body: Some(Body::Stream(Box::new(streamer))),
        }
    }

    pub fn new_err(status: StatusCode) -> Self {
        ResponseBuilder::new(status).text(status.as_str()).build()
    }

    pub fn new_html(s: impl Into<String>) -> Self {
        ResponseBuilder::ok()
            .text(s)
            .header("Content-Type", "text/html")
            .build()
    }

    pub fn new_bytes(bytes: Vec<u8>, content_type: &str) -> Self {
        ResponseBuilder::ok().bytes(bytes, content_type).build()
    }

    pub fn new_json(s: impl Into<String>) -> Self {
        ResponseBuilder::ok().json(s).build()
    }

    pub fn new_text(s: impl Into<String>) -> Self {
        ResponseBuilder::ok().text(s).build()
    }

    pub fn set_header(&mut self, key: &str, val: &str) {
        _ = self.headers.insert(key.to_string(), val.to_string());
    }

    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut header_str = format!("HTTP/1.1 {}\r\n", self.status.as_str());
        for (k, v) in &self.headers {
            header_str.push_str(&format!("{}: {}\r\n", k, v));
        }

        if !matches!(self.body, Some(Body::Stream(_))) {
            let body_bytes = match &self.body {
                Some(Body::Text(s)) => s.as_bytes().len(),
                Some(Body::Bytes(b)) => b.len(),
                Some(Body::Stream(_)) => 0, // will never fire
                None => 0,
            };
            header_str.push_str(&format!("Content-Length: {}\r\n", body_bytes));
        }

        header_str.push_str("\r\n");
        stream.write_all(header_str.as_bytes())?;

        match &self.body {
            Some(Body::Text(s)) => {
                stream.write_all(s.as_bytes())?;
                stream.flush()?;
            }
            Some(Body::Bytes(b)) => {
                stream.write_all(b)?;
                stream.flush()?;
            }
            Some(Body::Stream(func)) => {
                // streamer controls flush
                func(stream)?;
            }
            None => {}
        }

        Ok(())
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}
