use std::{collections::HashMap, io::Write, net::TcpStream};

use crate::meta::{Headers, StatusCode};

#[derive(Debug)]
pub struct Response {
    pub(crate) status: StatusCode,
    pub(crate) headers: Headers,
    pub(crate) body: Option<String>,
}

impl Response {
    pub fn new() -> Self {
        let mut s = Self {
            status: 200.into(),
            headers: HashMap::new(),
            body: None,
        };
        s.set_header("Connection", "close");
        s
    }
    pub fn new_err(err: StatusCode) -> Self {
        let mut s = Self {
            status: err,
            headers: HashMap::new(),
            body: None,
        };
        s.set_header("Connection", "close");
        s.set_body_plain(err.as_str().into());
        s
    }

    pub fn new_html(s: String) -> Response {
        let mut res = Response::new();
        res.set_body_html(s);
        res
    }

    pub fn set_body_json(self: &mut Response, s: String) {
        self.body = Some(s.into());
        self.set_header("Content-Type", "text/json");
    }
    pub fn set_body_plain(self: &mut Response, s: String) {
        self.body = Some(s.into());
        self.set_header("Content-Type", "text/plain");
    }

    pub fn set_body_html(self: &mut Response, s: String) {
        self.body = Some(s.into());
        self.set_header("Content-Type", "text/html");
    }

    pub fn set_header(&mut self, key: &str, val: &str) {
        _ = self.headers.insert(key.to_string(), val.to_string());
    }

    pub fn write_to(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        let body_len = self.body.as_ref().map_or(0, |b| b.len());
        if !self.headers.contains_key("Content-Length") {
            self.set_header("Content-Length", &body_len.to_string());
        }

        let mut res = format!("HTTP/1.1 {}\r\n", self.status.as_str());

        for (k, v) in &self.headers {
            res.push_str(&format!("{}: {}\r\n", k, v));
        }

        res.push_str("\r\n");

        if let Some(body) = &self.body {
            res.push_str(body);
        }

        stream.write_all(res.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}
