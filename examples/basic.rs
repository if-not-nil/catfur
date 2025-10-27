use cf::{meta::Method, request::Request, response::Response, server::Server};

fn main() -> std::io::Result<()> {
    let mut server = Server::new("localhost:8080");
    server.add_route(Method::GET, "/hello", |req: &Request| {
        Response::text(format!("hiiii!!! ur ip is {}", req.peer_addr))
    });

    server.add_route_static("/static/*", "./examples/static");

    server.serve().unwrap();

    Ok(())
}
