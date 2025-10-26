use std::io::Write;

use catfur::{
    builders::{ServerBuilder},
    middleware,
    request::Request, response::Response,
};

fn main() -> std::io::Result<()> {
    ServerBuilder::new("localhost:8080")
        .mw(middleware::logger)
        .get("/sse", |_: &Request| {
            Response::new_sse(|stream| {
                for i in 0..5 {
                    let msg = format!("data: hello #{i}\n\n");
                    stream.write_all(msg.as_bytes())?;
                    stream.flush()?;
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                Ok(())
            })
        })
        .build()
        .serve()
}
