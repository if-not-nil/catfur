use std::collections::HashMap;

use crate::{
    meta::{Handler, Headers, Method, StatusCode},
    request::Request,
    response::*,
    server::Server,
};

pub struct ResponseBuilder {
    status: StatusCode,
    headers: Headers,
    body: Option<Body>,
}

impl ResponseBuilder {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }
    pub fn ok() -> Self {
        ResponseBuilder::new(StatusCode::Ok)
    }

    pub fn text(mut self, s: impl Into<String>) -> Self {
        self.body = Some(Body::Text(s.into()));
        self
    }

    pub fn bytes(mut self, b: Vec<u8>, content_type: &str) -> Self {
        self.body = Some(Body::Bytes(b));
        self.headers
            .insert("Content-Type".into(), content_type.into());
        self
    }

    pub fn json(mut self, s: impl Into<String>) -> Self {
        self.body = Some(Body::Text(s.into()));
        self.headers
            .insert("Content-Type".into(), "application/json".into());
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    pub fn build(self) -> Response {
        let mut res = Response {
            status: self.status,
            headers: self.headers,
            body: self.body,
        };

        if let Some(Body::Text(s)) = &res.body {
            if !res.headers.contains_key("Content-Length") {
                res.set_header("Content-Length", &s.as_bytes().len().to_string());
            }
        } else if let Some(Body::Bytes(b)) = &res.body {
            if !res.headers.contains_key("Content-Length") {
                res.set_header("Content-Length", &b.len().to_string());
            }
        }

        res
    }
}

impl From<ResponseBuilder> for Response {
    fn from(value: ResponseBuilder) -> Self {
        value.build()
    }
}


pub struct ServerBuilder {
    server: Server,
}

impl ServerBuilder {
    pub fn new<A: std::net::ToSocketAddrs>(addr: A) -> Self {
        Self {
            server: Server::new(addr),
        }
    }

    pub fn mw<F>(mut self, ware: F) -> Self
    where
        F: Fn(Handler) -> Handler + Send + Sync + 'static,
    {
        self.server.add_middleware(ware);
        self
    }
    pub fn route<F>(mut self, method: Method, route: &str, handler: F) -> Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.server.add_route(method, route, handler);
        self
    }
    pub fn get<F>(self, route: &str, handler: F) -> Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.route(Method::GET, route, handler)
    }
    pub fn post<F>(mut self, route: &str, handler: F) -> Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.server.add_route(Method::POST, route, handler);
        self
    }

    pub fn static_route(mut self, route: &str, path: &str) -> Self {
        self.server.add_route_static(route, path);
        self
    }

    pub fn build(self) -> Server {
        self.server
    }
}
