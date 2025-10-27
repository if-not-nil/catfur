use cf::{builders::ServerBuilder, meta::Handler, request::Request, response::Response};

fn check_auth(handler: Handler) -> Handler {
    Box::new(move |req: &Request| {
        // insert whatever you wanna use for JWT
        if let Some(id) = req.query_param("token") {
            req.context.set("name", id.to_string());
        }
        handler(req)
    })
}

fn main() {
    ServerBuilder::new("127.0.0.1:8080")
        .mw(check_auth)
        .get("/hi", |req| {
            // middleware has checked auto and stored the user in context
            let name: String = req.context.get("name").unwrap_or("guest".to_string());
            Response::text(format!("hello, {}", name))
        })
        .build()
        .serve()
        .unwrap();
}
