use crate::{
    meta::{Handler, Method},
    request::Request,
    response::*,
    server::Server,
};

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
    pub fn get<F>(mut self, route: &str, handler: F) -> Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.server.add_route(Method::GET, route, handler);
        self
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
