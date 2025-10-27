use cf::{meta::Method, request::Request, response::Response, server::Server, sse::SseSendResult};

fn main() {
    let mut server = Server::new("127.0.0.1:8080");
    server.add_route(Method::GET, "/", |_: &Request| {
        Response::sse(|mut sse| {
            let mut id = 0;
            loop {
                let res = sse
                    .id(&id.to_string())
                    .event("message")
                    .data("hello sse")
                    .send();
                match res {
                    SseSendResult::Ok => {}
                    SseSendResult::Disconnected => {
                        eprintln!("client disconnected gracefully");
                        return;
                    }
                    SseSendResult::Err(error) => {
                        eprintln!("unexpected item in the bagging area: {}", error);
                        break;
                    }
                }

                id += 1;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        })
    });
    let _ = server.serve();
}
