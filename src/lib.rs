use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr,
};
mod threadpool;

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub enum Method {
    POST,
    GET,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseMethodError;
impl FromStr for Method {
    type Err = ParseMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            _ => Err(ParseMethodError),
        }
    }
}

pub struct Response {
    status: u16,
    headers: Headers,
    body: Option<String>,
}

impl Response {
    pub fn new() -> Self {
        let mut s = Self {
            status: 200,
            headers: HashMap::new(),
            body: None,
        };
        s.set_header("Connection", "close");
        s
    }

    pub fn set_body_json(self: &mut Response, s: &String) {
        self.body = Some(s.into());
        self.set_header("Content-Type", "text/json");
    }
    pub fn set_body_plain(self: &mut Response, s: &String) {
        self.body = Some(s.into());
        self.set_header("Content-Type", "text/plain");
    }

    pub fn set_header(&mut self, key: &str, val: &str) {
        _ = self.headers.insert(key.to_string(), val.to_string());
    }

    fn write_to(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let status_line = match self.status {
            200 => "HTTP/1.1 200 OK",
            404 => "HTTP/1.1 404 Not Found",
            500 => "HTTP/1.1 500 Internal Server Error",
            _ => "HTTP/1.1 200 OK",
        };
        let mut res = format!("{}\r\n", status_line);
        for (k, v) in &self.headers {
            res.push_str(format!("{}: {}\r\n", k.as_str(), v.as_str()).as_str());
        }

        if let Some(body) = &self.body {
            res.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
            res.push_str(body.as_str());
        }
        stream.write_all(res.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

pub struct Request {
    pub method: Method,
    pub route: String,
    pub headers: Headers,
    pub body: String,
    pub peer_addr: SocketAddr,
}

type Handler = fn(&Request) -> Response;

impl Request {
    fn from_stream(stream: &mut TcpStream) -> Result<Self, std::io::Error> {
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
            headers,
            body,
            method,
            route,
            peer_addr: stream.peer_addr().unwrap(),
        })
    }
}

pub struct Server {
    routes: HashMap<(Method, String), Handler>,
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
            routes: HashMap::new(),
        }
    }
    pub fn add_route(&mut self, method: Method, route: &str, handler: Handler) {
        self.routes.insert((method, route.to_string()), handler);
    }

    // fn route_static(path: &str) -> Response {
    //     Response::new()
    // }
    // pub fn add_route_static(&mut self, route: &str, path: &str) {
    //     self.routes.insert((Method::GET, route.to_string()), );
    // }
    pub fn serve(&self) -> Result<(), std::io::Error> {
        // let total = Arc::new(AtomicUsize::new(0));
        let listener = TcpListener::bind(self.addr)?;
        let pool = threadpool::ThreadPool::new(4);
        for sopt in listener.incoming() {
            let mut stream = sopt?;
            let routes = self.routes.clone();

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

                if let Some(handler) = routes.get(&(request.method.clone(), request.route.clone()))
                {
                    handler(&request).write_to(&mut stream).unwrap();
                } else {
                    let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
                    let _ = stream.flush();
                }
            });
        }
        Ok(())
    }
}

type Headers = HashMap<String, String>;

// fn main() -> std::io::Result<()> {
//     let mut router = Router::new();
//     router.add_route(Method::GET, "/hello", |request| {
//         let mut res = Response::new();
//         res.body = Some(
//             json!({
//                 "message": "hi",
//                 "ip": request.peer_addr
//             })
//             .to_string(),
//         );
//         res.set_header("Content-Type", "text/json");
//         res
//     });
//
//     router.serve().unwrap();
//
//     Ok(())
// }
