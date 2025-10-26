use std::{collections::HashMap, str::FromStr};

use crate::{request::Request, response::Response};

pub type Handler = Box<dyn Fn(&Request) -> Response + Send + Sync>;

pub type Headers = HashMap<String, String>;

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub enum Method {
    GET,
    POST,
    HEAD,
    OPTIONS,
    PUT,
    PATCH,
    DELETE,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseMethodError;

impl FromStr for Method {
    type Err = ParseMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "HEAD" => Ok(Method::HEAD),
            "OPTIONS" => Ok(Method::OPTIONS),
            "PUT" => Ok(Method::PUT),
            "PATCH" => Ok(Method::PATCH),
            "DELETE" => Ok(Method::DELETE),
            _ => Err(ParseMethodError),
        }
    }
}
