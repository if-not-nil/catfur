# cf
<img width="400" alt="image" src="https://github.com/user-attachments/assets/4d89570a-460e-4a08-8609-8cbb154cd3e4" />

## instructions
```bash
git clone https://github.com/if-not-nil/cf
cd catfur
cargo run --example builders
```

an implementation for a http server in async rust

the api is really simple and straightforward

```rust
use cf::{middleware, request::Request, response::Response, server::Server};

fn main() -> std::io::Result<()> {
    Server::at("localhost:8080")
        .mw(middleware::cors)
        .mw(middleware::logger)
        .get("/hi", |_req| Response::text("hi"))
        .get("/hello", |req: &Request| {
            Response::text(format!("hiiii!!! ur ip is {}", req.peer_addr,))
        })
        .get("/user/{name}", |req: &Request| {
            Response::text(format!(
                "hiiii!!! ur ip is {} and yr name is {}",
                req.peer_addr,
                req.param("name").unwrap()
            ))
        })
        .static_route("/static", "./examples/static")
        .serve()
}

```
this and future examples are in the `examples/` directory, the are the only documentation except the source code itself

it can go really fast on an M1
```bash
Running 5s test @ http://localhost:8080/hello
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     3.01ms    7.66ms  78.94ms   97.15%
    Req/Sec     6.52k     1.72k   23.55k    89.50%
  130042 requests in 5.07s, 24.18MB read
  Socket errors: connect 0, read 130037, write 0, timeout 0
Requests/sec:  25649.06
Transfer/sec:      4.77MB

```
