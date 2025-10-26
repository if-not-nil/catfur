use catfur::{
    builders::{ResponseBuilder, ServerBuilder}, middleware, request::Request
};

fn main() -> std::io::Result<()> {
    ServerBuilder::new("localhost:8080")
        .mw(middleware::cors)
        .mw(middleware::logger)
        .get("/hello", |req: &Request| {
            ResponseBuilder::ok()
                .text(format!("hiiii!!! ur ip is {}", req.peer_addr,))
                .build()
        })
        .get("/user/(?name*)", |req: &Request| {
            ResponseBuilder::ok()
                .text(format!(
                    "hiiii!!! ur ip is {} and yr name is {}",
                    req.peer_addr,
                    req.get_param("name").unwrap()
                ))
                .build()
        })
        .static_route("/static/(?file*)", "./examples/static")
        .build()
        .serve()
}
