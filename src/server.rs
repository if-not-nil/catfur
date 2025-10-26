use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    path::Path,
    sync::Arc,
};

use regex::Regex;

use crate::{
    meta::{Handler, Method},
    request::Request,
    response::*,
    threadpool,
};

pub struct Server {
    routes: Arc<HashMap<(Method, String), Handler>>,
    middleware: Vec<Box<dyn Fn(Handler) -> Handler>>,
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
    (Regex::new(&pattern).unwrap(), param_names)
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
            middleware: Vec::new(),
        }
    }
    pub fn add_middleware<F>(&mut self, mw: F)
    where
        F: Fn(Handler) -> Handler + Send + Sync + 'static,
    {
        self.middleware.push(Box::new(mw))
    }

    pub fn add_route<F>(&mut self, method: Method, route: &str, handler: F)
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        let mut h: Handler = Box::new(handler);
        for mw in &self.middleware {
            h = mw(h);
        }
        Arc::get_mut(&mut self.routes)
            .unwrap()
            .insert((method, route.to_string()), h);
    }

    pub fn add_route_static(&mut self, route: &str, path: &str) {
        let path = path.to_string();
        if !Path::new(&path).exists() {
            panic!("path specified does not exist: {}", path);
        }
        Arc::get_mut(&mut self.routes).unwrap().insert(
            (Method::GET, route.into()),
            Box::new(move |req: &Request| {
                println!("{:?}", req);
                let slice = req
                    .path_params
                    .get("file")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let mut fpath = Path::new(&path).join(slice);

                if fpath.is_dir() {
                    fpath = fpath.join("index.html");
                }

                let mut file = match File::open(&fpath) {
                    Ok(f) => f,
                    Err(_) => return Response::new_404(),
                };

                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                Response::new_html(contents)
            }),
        );
    }

    pub fn serve(&self) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(self.addr)?;
        let pool = threadpool::ThreadPool::new(8);

        for stream in listener.incoming() {
            let mut stream = stream?;
            let routes = Arc::clone(&self.routes);

            pool.execute(move || {
                let mut request = match Request::from_stream(&mut stream) {
                    Ok(req) => req,
                    Err(err) => {
                        eprintln!("failed to parse request: {}", err);
                        let _ = Response::new_404().write_to(&mut stream); // TODO: replace with
                        // 400
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

                if let Some((path, _)) = matched_route {
                    // call the handler
                    if let Some(handler) = routes.get(&(request.method.clone(), path)) {
                        if let Err(err) = handler(&request).write_to(&mut stream) {
                            eprintln!("failed to write response: {}", err);
                        }
                    } else {
                        // should never happen but fallback
                        let _ = Response::new_404().write_to(&mut stream);
                    }
                } else {
                    // actual 404
                    let _ = Response::new_404().write_to(&mut stream);
                }
            });
        }

        Ok(())
    }
}
