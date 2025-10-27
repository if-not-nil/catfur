use async_net::{TcpListener, TcpStream};
use smol::{fs::File, io::AsyncReadExt};
use std::{collections::HashMap, net::SocketAddr, path::Path, sync::Arc};

use regex::Regex;

use crate::{
    meta::{Handler, Method, StatusCode},
    middleware::Middleware,
    request::Request,
    response::*,
};

pub struct Server {
    routes: Arc<HashMap<(Method, String), Arc<Handler>>>,
    middleware: Arc<Vec<Middleware>>,
    addr: SocketAddr,
}

fn route_to_regex(path: &str) -> (Regex, Vec<String>) {
    let mut pattern = String::from("^");
    let mut chars = path.chars().peekable();
    let mut param_names = Vec::new();

    while let Some(c) = chars.next() {
        if c == '(' && chars.peek() == Some(&'?') {
            chars.next(); // skip '?'
            let mut name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == ')' || ch == '*' {
                    break;
                }
                name.push(ch);
                chars.next();
            }

            let wildcard = if chars.peek() == Some(&'*') {
                chars.next(); // skip '*'
                true
            } else {
                false
            };

            assert_eq!(chars.next(), Some(')')); // consume ')'

            param_names.push(name.clone());

            if wildcard {
                pattern.push_str("(.*)");
            } else {
                pattern.push_str("([^/]+)");
            }
        } else {
            pattern.push_str(&regex::escape(&c.to_string()));
        }
    }

    pattern.push('$');
    (
        Regex::new(&pattern).expect(
            "invalid route regex. it can only contain named parameters. example: /users/(?id*)",
        ),
        param_names,
    )
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
    pub fn add_middleware<F>(&mut self, mw: F)
    where
        F: Fn(Handler) -> Handler + Send + Sync + 'static,
    {
        Arc::get_mut(&mut self.middleware)
            .expect("cannot add middleware after cloning")
            .push(Box::new(mw))
    }

    // do not call this after calling serve()
    pub fn add_route<F>(&mut self, method: Method, route: &str, handler: F)
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        let h: Handler = Box::new(handler);
        Arc::get_mut(&mut self.routes)
            .unwrap()
            .insert((method, route.to_string()), Arc::new(h));
    }

pub fn add_route_static(&mut self, route: &str, path: &str) {
    let path = path.to_string();
    if !Path::new(&path).exists() {
        panic!("path specified does not exist: {}", path);
    }

    Arc::get_mut(&mut self.routes).unwrap().insert(
        (Method::GET, route.into()),
        Arc::new(Box::new(move |req: &Request| {
            let path_clone = path.clone();
            smol::block_on(async move {
                let slice = req
                    .path_params
                    .get("file")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let mut fpath = Path::new(&path_clone).join(slice);

                if fpath.is_dir() {
                    fpath = fpath.join("index.html");
                }

                match File::open(&fpath).await {
                    Ok(mut file) => {
                        let mut contents = String::new();
                        if let Err(_) = file.read_to_string(&mut contents).await {
                            return Response::error(StatusCode::InternalServerError);
                        }
                        Response::html(contents)
                    }
                    Err(_) => Response::error(StatusCode::NotFound),
                }
            })
        })),
    );
}

    async fn handle_connection(
        mut stream: TcpStream,
        routes: Arc<HashMap<(Method, String), Arc<Handler>>>,
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

        let matched_route = routes.keys().find_map(|(method, path)| {
            if *method != request.method {
                return None;
            }
            let (re, param_names) = route_to_regex(path);
            if let Some(caps) = re.captures(&request.route) {
                for (i, name) in param_names.iter().enumerate() {
                    request
                        .path_params
                        .insert(name.clone(), caps[i + 1].to_string());
                }
                Some((path.clone(), re))
            } else {
                None
            }
        });

        let base_handler: Handler = if let Some((path, _)) = matched_route {
            if let Some(handler) = routes.get(&(request.method.clone(), path)) {
                let base_handler = Arc::clone(handler);
                Box::new(move |req: &Request| base_handler(req))
            } else {
                Box::new(|_req: &Request| Response::error(StatusCode::NotFound))
            }
        } else {
            Box::new(|_req: &Request| Response::error(StatusCode::NotFound))
        };

        let mut h: Handler = base_handler;
        for mw in middleware.iter() {
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

    pub fn serve(&self) -> std::io::Result<()> {
        smol::block_on(self.serve_async())
    }

    pub async fn serve_async(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("Server listening on {}", self.addr);

        loop {
            let (stream, _) = listener.accept().await?;

            let routes = Arc::clone(&self.routes);
            let middleware = Arc::clone(&self.middleware);

            smol::spawn(Self::handle_connection(stream, routes, middleware)).detach();
        }
    }
}
