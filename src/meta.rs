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

#[derive(Debug, Clone, Copy)]
pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
    NotImplemented = 501,
}

impl StatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusCode::Ok => "200 OK",
            StatusCode::BadRequest => "400 Bad Request",
            StatusCode::NotFound => "404 Not Found",
            StatusCode::MethodNotAllowed => "405 Method Not Allowed",
            StatusCode::InternalServerError => "500 Internal Server Error",
            StatusCode::NotImplemented => "501 Not Implemented",
        }
    }
}
impl From<u16> for StatusCode {
    fn from(code: u16) -> Self {
        match code {
            200 => StatusCode::Ok,
            400 => StatusCode::BadRequest,
            404 => StatusCode::NotFound,
            405 => StatusCode::MethodNotAllowed,
            500 => StatusCode::InternalServerError,
            501 => StatusCode::NotImplemented,
            _ => StatusCode::InternalServerError,
        }
    }
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
