use async_net::TcpStream;
use smol::io::AsyncWriteExt;

use crate::{
    meta::{Headers, StatusCode},
    response::{Body, Response},
};

pub struct SseSender<'a> {
    stream: &'a mut TcpStream,
    buf: String,
}

pub enum SseSendResult {
    Ok,
    Disconnected,
    Err(std::io::Error),
}

impl<'a> SseSender<'a> {
    pub fn event(&mut self, name: &str) -> &mut Self {
        self.buf.push_str(&format!("event: {}\n", name));
        self
    }

    pub fn data(&mut self, data: &str) -> &mut Self {
        self.buf.push_str(&format!("data: {}\n", data));
        self
    }

    pub fn id(&mut self, id: &str) -> &mut Self {
        self.buf.push_str(&format!("id: {}\n", id));
        self
    }

    pub fn retry(&mut self, ms: u64) -> &mut Self {
        self.buf.push_str(&format!("retry: {}\n", ms));
        self
    }

    pub fn ping(&mut self) -> &mut Self {
        self.event("ping").data("heartbeat")
    }

    pub fn comment(&mut self, text: &str) -> &mut Self {
        self.buf.push_str(&format!(": {}\n", text));
        self
    }

    pub async fn send(&mut self) -> SseSendResult {
        self.buf.push_str("\n");

        if let Err(e) = self.stream.write_all(self.buf.as_bytes()).await {
            return match e.kind() {
                // when the client disconnects themselves
                std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset => {
                    SseSendResult::Disconnected
                }
                _ => SseSendResult::Err(e),
            };
        }

        if let Err(e) = self.stream.flush().await {
            return match e.kind() {
                // when the client disconnects themselves
                std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset => {
                    SseSendResult::Disconnected
                }
                _ => SseSendResult::Err(e),
            };
        }

        self.buf.clear();
        SseSendResult::Ok
    }
}

impl Response {
    pub fn sse<F>(handler: F) -> Response
    where
        F: Fn(SseSender) + Send + Sync + 'static,
    {
        let mut headers = Headers::new();
        headers.insert("Content-Type".into(), "text/event-stream".into());
        headers.insert("Cache-Control".into(), "no-cache".into());
        headers.insert("Connection".into(), "keep-alive".into());

        Response {
            status: StatusCode::Ok,
            headers,
            body: Some(Body::Stream(Box::new(move |stream| {
                let sender = SseSender {
                    stream,
                    buf: String::new(),
                };
                handler(sender);
                Ok(())
            }))),
        }
    }
}
