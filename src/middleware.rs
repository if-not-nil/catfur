use std::time::SystemTime;

use crate::{meta::Handler, request::Request};

pub type Middleware = Box<dyn Fn(Handler) -> Handler + Send + Sync>;

pub fn logger(handler: Handler) -> Handler {
    Box::new(move |req: &Request| {
        let start = SystemTime::now();
        let res = handler(req);
        let elapsed = start.elapsed().unwrap();
        println!(
            "{} request to {} took {:?}",
            req.method,
            req.route,
            elapsed
        );
        if res.status as u16 > 299 {
            println!("^^^ {}:\n{:?}", res.status.as_str(), res.body);
        }

        res
    })
}

pub fn cors(handler: Handler) -> Handler {
    Box::new(move |req: &Request| {
        handler(req)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Expose-Headers", "Content-Type")
    })
}
