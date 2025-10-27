use std::{
    collections::HashMap, io::{stdout, Write}, path::Path, str::FromStr
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
    // 2xx success
    Ok = 200 "OK",
    Created = 201 "Created",
    Accepted = 202 "Accepted",
    NoContent = 204 "No Content",
    ResetContent = 205 "Reset Content",
    PartialContent = 206 "Partial Content",
    
    // 3xx redirection
    MovedPermanently = 301 "Moved Permanently",
    Found = 302 "Found",
    SeeOther = 303 "See Other",
    NotModified = 304 "Not Modified",
    TemporaryRedirect = 307 "Temporary Redirect",
    PermanentRedirect = 308 "Permanent Redirect",
    
    // 4xx client errors
    BadRequest = 400 "Bad Request",
    Unauthorized = 401 "Unauthorized",
    PaymentRequired = 402 "Payment Required",
    Forbidden = 403 "Forbidden",
    NotFound = 404 "Not Found",
    MethodNotAllowed = 405 "Method Not Allowed",
    NotAcceptable = 406 "Not Acceptable",
    ProxyAuthenticationRequired = 407 "Proxy Authentication Required",
    RequestTimeout = 408 "Request Timeout",
    Conflict = 409 "Conflict",
    Gone = 410 "Gone",
    LengthRequired = 411 "Length Required",
    PreconditionFailed = 412 "Precondition Failed",
    PayloadTooLarge = 413 "Payload Too Large",
    UriTooLong = 414 "URI Too Long",
    UnsupportedMediaType = 415 "Unsupported Media Type",
    RangeNotSatisfiable = 416 "Range Not Satisfiable",
    ExpectationFailed = 417 "Expectation Failed",
    ImATeapot = 418 "I'm a teapot",
    UnprocessableEntity = 422 "Unprocessable Entity",
    TooManyRequests = 429 "Too Many Requests",
    
    // 5xx server errors
    InternalServerError = 500 "Internal Server Error",
    NotImplemented = 501 "Not Implemented",
    BadGateway = 502 "Bad Gateway",
    ServiceUnavailable = 503 "Service Unavailable",
    GatewayTimeout = 504 "Gateway Timeout",
    HttpVersionNotSupported = 505 "HTTP Version Not Supported",
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

pub fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
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
