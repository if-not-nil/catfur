use catfur::{
    builders::{ResponseBuilder, ServerBuilder},
    request::Request,
};

fn main() -> std::io::Result<()> {
    ServerBuilder::new("localhost:8080")
        .get(
            "/hello",
            Box::new(|req: &Request| {
                ResponseBuilder::ok()
                    .body(format!("hiiii!!! ur ip is {}", req.peer_addr).into())
                    .build()
            }),
        )
        .static_route("/static/*", "./static")
        .build()
        .serve()
}
