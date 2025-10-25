use catfur::builders::{ResponseBuilder, ServerBuilder};

fn main() -> std::io::Result<()> {
    ServerBuilder::new("localhost:8080")
        .get(
            "/hello",
            Box::new(|req: &catfur::Request| {
                ResponseBuilder::ok()
                    .body(format!("hiiii!!! ur ip is {}", req.peer_addr).into())
                    .build()
            }),
        )
        .static_route("/static/*", "./static")
        .build()
        .serve()
}
