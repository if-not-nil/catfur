use async_net::TcpStream;
use smol::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use crate::meta::{Headers, Method};

#[derive(Debug, Clone)]
pub struct Context(Arc<RwLock<HashMap<String, String>>>);

impl Context {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn set(&self, key: impl Into<String>, value: impl Into<String>) {
        self.0.write().unwrap().insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.0.read().unwrap().get(key).cloned()
    }
}

fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub route: String,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub peer_addr: SocketAddr,
    pub context: Context,
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl Request {
    pub fn text(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers
            .get(&key.to_ascii_lowercase())
            .map(|s| s.as_str())
    }
    pub fn param(&self, key: &str) -> Option<&str> {
        self.path_params.get(key).map(|s| s.as_str())
    }
    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query_params.get(key).map(|s| s.as_str())
    }
    pub async fn from_stream(stream: &mut TcpStream) -> std::io::Result<Self> {
        let peer_addr = stream.peer_addr()?;
        let mut reader = BufReader::new(stream);
        //
        // request line
        //
        let (route, method, query_params) = {
            let mut request_line = String::new();
            reader.read_line(&mut request_line).await?;
            if request_line.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "empty request line",
                ));
            };
            let mut parts = request_line.trim_end().split_whitespace();
            let method_str = parts.next().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "missing HTTP method")
            })?;
            let method = method_str.parse::<Method>().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid method")
            })?;

            let route = parts.next().unwrap_or("/").to_string();
            let (path, query_params) = if let Some((path, query)) = route.split_once('?') {
                (path.to_string(), parse_query_params(query))
            } else {
                (route.clone(), HashMap::new())
            };
            (path, method, query_params)
        };
        //
        // headers
        //
        let mut headers: Headers = HashMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            let line = line.trim_end();
            if line.is_empty() {
                break; // end of headers
            }
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
            };
        }
        //
        // body
        //
        let body = {
            let content_length = headers
                .get("content-length")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);

            let mut body = vec![0u8; content_length];
            if content_length > 0 {
                reader.read_exact(&mut body).await?;
            };
            body
        };
        Ok(Request {
            method,
            query_params,
            route,
            headers,
            body,
            peer_addr,
            context: Context::new(),
            path_params: HashMap::new(),
        })
    }
}
