use async_net::{TcpListener, TcpStream};
use smol::{fs::File, io::AsyncReadExt};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

use crate::{
    meta::{guess_content_type, print_banner, Handler, Method, StatusCode},
    middleware::Middleware,
    request::Request,
    response::*,
};

pub struct Server {
    routes: Arc<HashMap<Method, Vec<Route>>>,
    middleware: Arc<Vec<Middleware>>,
    addr: SocketAddr,
}

pub struct Route {
    pub segments: Vec<RouteSegment>,
    pub handler: Arc<Handler>,
}

pub enum RouteSegment {
    Static(String),
    Param(String),
}

impl Server {
    pub fn new<A: std::net::ToSocketAddrs>(addr: A) -> Self {
        Self {
            addr: addr
                .to_socket_addrs()
                .expect("failed to resolve address!")
                .next()
                .expect("no valid addresses?"),
            routes: Arc::new(HashMap::new()),
            middleware: Arc::new(Vec::new()),
        }
    }
    fn add_middleware<F>(&mut self, mw: F)
    where
        F: Fn(Handler) -> Handler + Send + Sync + 'static,
    {
        Arc::get_mut(&mut self.middleware)
            .expect("cannot add middleware after cloning")
            .push(Box::new(mw))
    }

    // do not call this after calling serve()
    fn add_route<F>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        let segments = path
            .trim_start_matches('/')
            .split('/')
            .map(|s| {
                if s.starts_with('{') && s.ends_with('}') {
                    RouteSegment::Param(s[1..s.len() - 1].to_string())
                } else {
                    RouteSegment::Static(s.to_string())
                }
            })
            .collect();

        let route = Route {
            segments,
            handler: Arc::new(Box::new(handler)),
        };

        Arc::get_mut(&mut self.routes)
            .expect("cannot add routes after cloning")
            .entry(method)
            .or_default()
            .push(route);
    }

    fn add_route_static(&mut self, route: &str, dir_path: &str) {
        let dir_path = dir_path.to_string();

        self.add_route(
            Method::GET,
            &format!("{}/{{filepath}}", route),
            move |req: &Request| {
                let dir_path = dir_path.clone();

                let file_path = req
                    .path_params
                    .get("filepath")
                    .map(|s| s.as_str())
                    .unwrap_or("");

                if file_path.contains("..") {
                    return Response::error(StatusCode::ImATeapot); // pathbuf should protect u
                    // anyways but idk
                }

                let full_path = PathBuf::from(&dir_path).join(file_path);

                match smol::block_on(async {
                    if full_path.is_dir() {
                        let index_path = full_path.join("index.html");
                        let mut file = File::open(&index_path).await?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents).await?;
                        Ok::<_, std::io::Error>((contents, index_path))
                    } else {
                        let mut file = File::open(&full_path).await?;
                        let mut contents = Vec::new();
                        file.read_to_end(&mut contents).await?;
                        Ok((contents, full_path))
                    }
                }) {
                    Ok((contents, path)) => {
                        let content_type = guess_content_type(&path);
                        Response::bytes(contents, content_type)
                    }
                    Err(_) => Response::error(StatusCode::NotFound),
                }
            },
        );
    }

    fn match_route<'a>(
        routes: &'a [Route],
        path: &str,
    ) -> Option<(&'a Arc<Handler>, HashMap<String, String>)> {
        let req_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        for route in routes {
            if route.segments.len() != req_segments.len() {
                continue;
            }

            let mut params = HashMap::new();
            let mut matched = true;

            for (seg, req_seg) in route.segments.iter().zip(req_segments.iter()) {
                match seg {
                    RouteSegment::Static(s) if s != req_seg => {
                        matched = false;
                        break;
                    }
                    RouteSegment::Param(name) => {
                        params.insert(name.clone(), req_seg.to_string());
                    }
                    _ => {}
                }
            }

            if matched {
                return Some((&route.handler, params));
            }
        }

        None
    }

    async fn handle_connection(
        mut stream: TcpStream,
        routes: Arc<HashMap<Method, Vec<Route>>>,
        middleware: Arc<Vec<Middleware>>,
    ) {
        let mut request = match Request::from_stream(&mut stream).await {
            Ok(req) => req,
            Err(err) => {
                eprintln!("failed to parse request: {}", err);
                let _ = Response::error(StatusCode::BadRequest)
                    .write_to(&mut stream)
                    .await;
                return;
            }
        };

        let default_handler: Arc<Handler> = Arc::new(Box::new(|_req: &Request| {
            Response::error(StatusCode::NotFound)
        }));

        let (handler, path_params) = routes
            .get(&request.method)
            .and_then(|routes| Self::match_route(routes, &request.route))
            .unwrap_or_else(|| (&default_handler, HashMap::new()));

        request.path_params = path_params;

        // cant be moved in2 the closure if u dont clone it
        let handler = Arc::clone(handler);

        // start w the base handler and go through the mw backwards
        let mut h: Handler = Box::new(move |req: &Request| handler(req));
        for mw in middleware.iter().rev() {
            h = mw(h);
        }

        let response = h(&request);
        if let Err(err) = response.finalize().write_to(&mut stream).await {
            match err.kind() {
                std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset => {}
                _ => eprintln!("failed to write response: {}", err),
            }
        }
    }

    async fn serve_async(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        print_banner(self.addr.to_string());

        let routes = Arc::clone(&self.routes);
        let middleware = Arc::clone(&self.middleware);

        loop {
            let (stream, _) = listener.accept().await?;

            let routes = Arc::clone(&routes);
            let middleware = Arc::clone(&middleware);

            smol::spawn(Self::handle_connection(stream, routes, middleware)).detach();
        }
    }

    // chainable methods
    pub fn at(addr: &str) -> Self {
        let addr = if addr.starts_with(':') {
            format!("0.0.0.0{}", addr)
        } else {
            addr.to_string()
        };
        Self::new(addr.as_str())
    }

    pub fn mw<F>(mut self, ware: F) -> Self
    where
        F: Fn(Handler) -> Handler + Send + Sync + 'static,
    {
        self.add_middleware(ware);
        self
    }
    pub fn route<F>(mut self, method: Method, route: &str, handler: F) -> Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.add_route(method, route, handler);
        self
    }
    pub fn get<F>(mut self, route: &str, handler: F) -> Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.add_route(Method::GET, route, handler);
        self
    }
    pub fn post<F>(mut self, route: &str, handler: F) -> Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.add_route(Method::POST, route, handler);
        self
    }

    pub fn static_route(mut self, route: &str, path: &str) -> Self {
        self.add_route_static(route, path);
        self
    }

    pub fn serve(&self) -> std::io::Result<()> {
        smol::block_on(self.serve_async())
    }
}
