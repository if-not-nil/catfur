# catfur
an implementation for a http server in rust. works on top of regular sockets and regex

the api is really simple and straightforward

you can use two ways of making servers

## the builder pattern
```rust
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

```

## procedural way
```rust
use catfur::{Method, Response, Server};

fn main() -> std::io::Result<()> {
    let mut router = Server::new("localhost:8080");
    println!("serving on localhost:8080");

    router.add_route(Method::GET, "/hello", |request| {
        let mut res = Response::new();
        let body = (format!("hiiii!!! ur ip is {}", request.peer_addr)).to_string();
        res.set_body_plain(&body);
        res
    });

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


