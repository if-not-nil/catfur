use std::{
    any::Any, collections::HashMap, io::Read, net::{SocketAddr, TcpStream}
};

use crate::meta::{Headers, Method};

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub route: String,
    pub headers: Headers,
    pub body: String,
    pub peer_addr: SocketAddr,
    pub context: HashMap<String, Box<dyn Any + Send + Sync>>,
    pub path_params: HashMap<String, String>
}

impl Request {
    pub fn from_stream(stream: &mut TcpStream) -> Result<Self, std::io::Error> {
        // headers
        let mut buf = Vec::new();
        let mut tmpbuf = [0u8; 512];
        loop {
            let n = stream.read(&mut tmpbuf)?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmpbuf[..n]);

            // insane what rust can do
            if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
        let header_end = match buf.windows(4).position(|w| w == b"\r\n\r\n") {
            Some(pos) => pos,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid http request: missing headers",
                ));
            }
        };
        let header_bytes = &buf[..header_end];
        let headers_str = std::str::from_utf8(header_bytes).unwrap_or("");

        let mut headers: Headers = HashMap::new();
        for line in headers_str.lines().skip(1) {
            if let Some((name, value)) = line.split_once(": ") {
                headers.insert(name.to_string(), value.to_string());
            }
        }

        // body
        let content_length = headers
            .get("Content-Length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let body = if content_length > 0 {
            let body_bytes = {
                let mut body = vec![0u8; content_length];
                stream.read_exact(&mut body)?;
                body
            };
            String::from_utf8_lossy(&body_bytes).into_owned()
        } else {
            String::new()
        };

        // route & method
        let request_line = headers_str.lines().next().unwrap_or("");
        let mut parts = request_line.split_whitespace();
        let method_str = parts.next().unwrap_or("");
        let route_str = parts.next().unwrap_or("/");

        let method = method_str
            .parse::<Method>()
            .map_err(|_| std::io::ErrorKind::InvalidData)?;
        let route = route_str.to_string();

        Ok(Request {
            context: HashMap::new(),
            headers,
            body,
            method,
            route,
            peer_addr: stream.peer_addr().unwrap(),
            path_params: HashMap::new()
        })
    }
}
