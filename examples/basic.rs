use cf::{meta::Method, middleware, request::Request, response::Response, server::Server};

fn main() -> std::io::Result<()> {
    Server::at("localhost:8080")
        .mw(middleware::cors)
        .mw(middleware::logger)
        .route(Method::GET, "/asdf/{name}", |req| {
            Response::text(format!(
                "hi {}, u like {}?",
                req.param("name").unwrap(),
                req.query_param("fav_food").unwrap_or("NOTHING")
            ))
        })
        .get("/hi", |_| "hi") // anything that implements an Into<Response> goes. you can even
        // implement it yourself
        .get("/hello", |req: &Request| {
            Response::text(format!("hiiii!!! ur ip is {}", req.peer_addr,))
        })
        .get("/user/{name}", |req: &Request| {
            Response::text(format!(
                "hiiii!!! ur ip is {} and yr name is {}",
                req.peer_addr,
                req.param("name").unwrap()
            ))
        })
        .static_route("/static", "./examples/static")
        .serve()
}
