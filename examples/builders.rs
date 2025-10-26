use std::time::SystemTime;

use catfur::{
    builders::{ResponseBuilder, ServerBuilder},
    meta::{Handler},
    request::Request,
};

fn main() -> std::io::Result<()> {
    ServerBuilder::new("localhost:8080")
        .mw(logger)
        .get("/hello", |req: &Request| {
            ResponseBuilder::ok()
                .body(format!("hiiii!!! ur ip is {}", req.peer_addr).into())
                .build()
        })
        .static_route(r"/static/(?file*)", "./examples/static")
        .build()
        .serve()
}

fn logger(handler: Handler) -> Handler {
    Box::new(move |req: &Request| {
        let start = SystemTime::now();
        let res = handler(req);
        let elapsed = start.elapsed().unwrap();
        println!("request took {:?}", elapsed);

        res
    })
}
