use cf::{response::Response, server::Server};
use smol::Timer;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    Server::new("0.0.0.0:8080")
        .get("/", |_req| {
            Response::sse(|mut sink| async move {
                let mut counter = 0;
                loop {
                    sink.send(&format!("count: {}", counter)).await?;
                    counter += 1;
                    Timer::after(Duration::from_secs(1)).await;
                }
            })
        })
        .serve()
}
