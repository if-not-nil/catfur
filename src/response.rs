use std::{collections::HashMap, io::Write, net::TcpStream};

use crate::meta::Headers;

#[derive(Debug)]
pub struct Response {
    pub(crate) status: u16,
    pub(crate) headers: Headers,
    pub(crate) body: Option<String>,
}

impl Response {
    pub fn new() -> Self {
        let mut s = Self {
            status: 200,
            headers: HashMap::new(),
            body: None,
        };
        s.set_header("Connection", "close");
        s
    }
    pub fn new_404() -> Self {
        let mut s = Self {
            status: 404,
            headers: HashMap::new(),
            body: None,
        };
        s.set_header("Connection", "close");
        s.set_body_plain("not found!".to_string());
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

    pub fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let status_line = match self.status {
            200 => "HTTP/1.1 200 OK",
            404 => "HTTP/1.1 404 Not Found",
            500 => "HTTP/1.1 500 Internal Server Error",
            _ => "HTTP/1.1 200 OK",
        };
        let mut res = format!("{}\r\n", status_line);
        for (k, v) in &self.headers {
            res.push_str(format!("{}: {}\r\n", k.as_str(), v.as_str()).as_str());
        }

        if let Some(body) = &self.body {
            res.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
            res.push_str(body.as_str());
        }
        stream.write_all(res.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}
