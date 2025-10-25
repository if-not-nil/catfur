use catfur::{Method, Response, Server};

fn main() -> std::io::Result<()> {
    let mut router = Server::new("localhost:8080");
    println!("serving on localhost:8080");

    router.add_route(Method::GET, "/hello", |request| {
        let mut res = Response::new();
        let body = (format!("hiiii!!! ur ip is {}", request.peer_addr)).to_string();
        res.set_body_plain(&body);
        res
    });

    router.serve().unwrap();
    Ok(())
}
