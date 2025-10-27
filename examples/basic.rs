use cf::{middleware, request::Request, response::Response, server::Server};

fn main() -> std::io::Result<()> {
    Server::at("localhost:8080")
        .mw(middleware::cors)
        .mw(middleware::logger)
        .get("/hi", |_req| Response::text("hi"))
        .get("/hello", |req: &Request| {
            Response::text(format!("hiiii!!! ur ip is {}", req.peer_addr,))
        })
        .get("/user/(?name*)", |req: &Request| {
            Response::text(format!(
                "hiiii!!! ur ip is {} and yr name is {}",
                req.peer_addr,
                req.param("name").unwrap()
            ))
        })
        .static_route("/static", "./examples/static")
        .serve()
}
