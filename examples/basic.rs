use catfur::{meta::Method, request::Request, response::Response, server::Server};

fn main() -> std::io::Result<()> {
    let mut router = Server::new("localhost:8080");
    println!("serving on localhost:8080");

    router.add_route(Method::GET, "/hello", |req: &Request| {
        let mut res = Response::new();
        let body = (format!("hiiii!!! ur ip is {}", req.peer_addr)).to_string();
        res.set_body_plain(body);
        res
    });

    router.add_route_static("/static/*", "./static");

    router.serve().unwrap();
    Ok(())
}
