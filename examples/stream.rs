use cf::{response::Response, server::Server};
use smol::{Timer, io::AsyncWriteExt};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    Server::new("0.0.0.0:8080")
        .get("/", |_req| {
            Response::stream(|mut stream: smol::net::TcpStream| {
                Box::pin(async move {
                    loop {
                        let chunk = b"hi\r\n";
                        // chunk header (length in hex)
                        let header = format!("{:X}\r\n", chunk.len());
                        stream.write_all(header.as_bytes()).await.unwrap();
                        stream.write_all(chunk).await.unwrap();
                        stream.write_all(b"\r\n").await.unwrap(); // chunk terminator
                        stream.flush().await.unwrap();

                        Timer::after(Duration::from_secs(1)).await; 
                    }
                })
            })
        })
        .serve()
}
