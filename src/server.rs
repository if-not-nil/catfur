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
    addr: SocketAddr,
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
        }
    }
    pub fn add_route(&mut self, method: Method, route: &str, handler: Handler) {
        Arc::get_mut(&mut self.routes)
            .unwrap()
            .insert((method, route.to_string()), handler);
    }

    pub fn add_route_static(&mut self, route: &str, path: &str) {
        let path = path.to_string();
        Arc::get_mut(&mut self.routes).unwrap().insert(
            (Method::GET, route.into()),
            Box::new(move |req: &Request| {
                let slice = &req.route[(path.len())..(req.route.len())];
                let mut fpath = Path::new(&path).join(slice);
                if fpath.is_dir() {
                    fpath = fpath.join("index.html");
                };

                println!("{:?}", fpath.join(slice));
                let mut file = if let Ok(file) = File::open(fpath) {
                    file
                } else {
                    return Response::new_404();
                };

                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                let mut res = Response::new();
                res.set_body_html(&mut contents);
                res
            }),
        );
    }

    pub fn serve(&self) -> Result<(), std::io::Error> {
        // let total = Arc::new(AtomicUsize::new(0));
        let listener = TcpListener::bind(self.addr)?;
        let pool = threadpool::ThreadPool::new(8);
        for sopt in listener.incoming() {
            let mut stream = sopt?;
            let routes = Arc::clone(&self.routes);

            pool.execute(move || {
                let request = match Request::from_stream(&mut stream) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("failed to parse request: {}", e);
                        _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n");
                        _ = stream.flush();
                        return;
                    }
                };
                let route = routes.keys().find(|(method, path)| {
                    if *method != request.method {
                        return false;
                    }
                    let pattern = format!("^{}$", regex::escape(path).replace(r"\*", ".*"));
                    let re = Regex::new(&pattern).unwrap();
                    re.is_match(&request.route)
                });

                if let Some((method, path)) = route {
                    let handler = routes.get(&(method.clone(), path.clone())).unwrap();
                    handler(&request).write_to(&mut stream).unwrap();
                } else {
                    Response::new_404().write_to(&mut stream).unwrap();
                }
            });
        }
        Ok(())
    }
}
