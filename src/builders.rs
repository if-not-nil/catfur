use std::collections::HashMap;

use crate::{
    meta::{Handler, Headers, Method},
    request::Request,
    response::*,
    server::Server,
};

pub struct ResponseBuilder {
    status: u16,
    headers: Headers,
    body: Option<String>,
}

impl ResponseBuilder {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }
    pub fn ok() -> Self {
        ResponseBuilder::new(200)
    }
    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
    pub fn header(mut self, k: String, v: String) -> Self {
        self.headers.insert(k, v);
        self
    }
    pub fn build(self) -> Response {
        Response {
            status: self.status,
            headers: self.headers,
            body: self.body,
        }
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
