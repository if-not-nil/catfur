use std::{
    collections::HashMap,
    io::{Write, stdout},
    str::FromStr,
};

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

impl Method {
    pub fn to_string(&self) -> String {
        String::from(match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::PUT => "PUT",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
        })
    }
}

macro_rules! status_codes {
    ($($name:ident = $code:literal $reason:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum StatusCode {
            $($name = $code),*
        }

        impl StatusCode {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$name => concat!($code, " ", $reason)),*
                }
            }
        }

        impl From<u16> for StatusCode {
            fn from(code: u16) -> Self {
                match code {
                    $($code => Self::$name),*,
                    _ => Self::InternalServerError,
                }
            }
        }
    };
}

status_codes! {
    Ok = 200 "OK",
    NoContent = 204 "No Content",
    BadRequest = 400 "Bad Request",
    NotFound = 404 "Not Found",
    MethodNotAllowed = 405 "Method Not Allowed",
    InternalServerError = 500 "Internal Server Error",
    NotImplemented = 501 "Not Implemented",
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

pub fn print_banner(host: String, thread_num: usize) {
    let esc_banner = "\x1b[90m\x1b[1m";
    let esc_reset = "\x1b[97m\x1b[0m";
    fn make_line(input: String) -> String {
        "\x1b[97m\x1b[0m".to_owned() + &input + "\x1b[90m\x1b[1m"
    }
    let banner = format!(
        "{}
  |\\'/-..--.   {}
 / _ _   ,  ;  {}
`~=`Y'~_<._./  {}
 <`-....__.'   {}
            {}\n",
        esc_banner,
        make_line(format!(
            "cf \x1b[33m\x1b[1mv{version}",
            version = env!("CARGO_PKG_VERSION")
        )),
        make_line(format!("serving at")),
        make_line(format!("{host}")),
        make_line(format!("on {thread_num} threads")),
        esc_reset
    );
    _ = stdout().write_all(&banner.into_bytes());
    _ = stdout().flush();
}
