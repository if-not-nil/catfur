# cf
> a lightweight http server using the smol crate for async, with an effort to have as few dependencies as possible
<img width="400" alt="image" src="https://github.com/user-attachments/assets/4d89570a-460e-4a08-8609-8cbb154cd3e4" />

currently handles 25k requests on an m1 with a simple route

## instructions
```bash
git clone https://github.com/if-not-nil/cf
cd cf
cargo run --example basic
```

the api is
- you make a server, chain methods on it, and serve it
- you make a response, chain methods on it, and return it

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
        .get("/user/{name}", |req: &Request| { // use {name} to capture a segment, ...
            Response::text(format!(
                "hiiii!!! ur ip is {} and yr name is {}",
                req.peer_addr,
                req.param("name").unwrap() // then get it with req.param
            ))
        })
        .static_route("/static", "./examples/static")
        .serve()
}

```

the middleware just takes the previous handler and returns a new one like this.

the context is a simple k-v store for strings only (until i manage to use it with any type), it's attached to every request by default
```rust
use cf::{meta::Handler, request::Request, response::Response, server::Server};

fn main() {
    Server::at("127.0.0.1:8080")
        .mw(check_auth)      // the middleware populates context with "name"
        .get("/hi", |req| {  // then the request can retrieve it
            let name: String = req.context.get("name").unwrap_or("guest".to_string());
            Response::text(format!("hello, {}", name))
        })
        .serve()
        .unwrap();
}

fn check_auth(handler: Handler) -> Handler {
    Box::new(move |req: &Request| {
        // insert whatever you wanna use for JWT
        if let Some(id) = req.query_param("token") {
            req.context.set("name", id.to_string());
        }
        handler(req)
    })
}
```

# documentation

## `cf::Server`
> you add routes and middleware to the server object and then serve it
- `at(addr)`
    ```rust
    Server::at("127.0.0.1:8080") // bind an address
        .serve()                 // serve at that address (you have to call both)
    ```
- `static_route(path, route)`: serve files from a static dir
    ```rust
    Server::at("127.0.0.1:8080")
        .static_route("/static", "./public");
    ```
- `route(method, route, handler)`: bind a route
    ```rust
    .route(cf::meta::Method::PATCH, "/patch", |req: &Request| {
        Response::text(format!("ur ip is {}", req.peer_addr))
            .status(cf::meta::StatusCode::ImATeapot)
    })
    ```
- `post(route, handler)`: shorthands for route()
    ```rust
    .get("/yo", |req| { 
        Response::text(format!("yo", name))
    })
    .post("/yo", |req| { 
        Response::text(format!("yo", name))
    })
    ```
## `cf::Response`
a handler needs to return this, then it is written to the client

```rust
Response::text(format!("ur ip is {}", req.peer_addr))
    .status(cf::meta::StatusCode::ImATeapot)
```
- status codes
    ```rust
    Response::text("yo").status(status: cf::meta::StatusCode) // you can also do 404.into() and all that
    ```
- headers
    ```rust
    Response::text("yo").header("Content-Type", "application/json")
    ```
- content type shorthands
    - `Response::text("...")` – text/plain
    - `Response::html("...")` – text/html
    - `Response::json("...")` – application/json
    - `Response::bytes(vec, content_type)` – binary
    - `Response::empty()` – HTTP 204 No Content
    - `Response::error(StatusCode)` – HTTP error response (as text/plain)
