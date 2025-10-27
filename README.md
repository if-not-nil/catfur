# cf
<img width="281" height="102" alt="image" src="https://github.com/user-attachments/assets/b2babb54-024f-4b7b-8331-14e8fd7fd5a8" />

an implementation for a http server in rust. works on top of regular sockets and regex

## instructions
```bash
git clone (https://github.com/if-not-nil/catfur
cd catfur
cargo run --example builders
```

the api is really simple and straightforward

you can use two ways of making servers

**the builder pattern**
```rust
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

```

**procedural way**
```rust
use catfur::{meta::Method, request::Request, response::Response, server::Server};

fn main() -> std::io::Result<()> {
    let mut router = Server::new("localhost:8080");
    router.add_route(Method::GET, "/hello", |req: &Request| {
        Response::new_text(format!("hiiii!!! ur ip is {}", req.peer_addr))
    });

    router.add_route_static("/static/*", "./static");

    router.serve().unwrap();

    Ok(())
}

```
both examples are in the `examples/` directory

it can do 20k requests per second on an M1 with a simple route, too
```bash
 wrk -t4 -c100 -d5s --header "Connection: close" http://localhost:8080/hello
Running 5s test @ http://localhost:8080/hello
  4 threads and 100 connections
^C  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.18ms    2.40ms  40.33ms   98.80%
    Req/Sec     6.24k   660.51     6.89k    88.89%
  44730 requests in 1.80s, 4.82MB read
Requests/sec:  24835.55
Transfer/sec:      2.68MB
```

sse is not ready yet, you shouldnt use it
