use cf::{meta::Method, request::Request, response::Response, server::Server};

fn main() -> std::io::Result<()> {
    let mut router = Server::new("localhost:8080");
    router.add_route(Method::GET, "/hello", |req: &Request| {
        Response::new_text(format!("hiiii!!! ur ip is {}", req.peer_addr))
    });

    router.add_route_static("/static/*", "./static");

    router.serve().unwrap();

    Ok(())
}
