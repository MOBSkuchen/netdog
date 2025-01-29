use crate::logger::{LogLevel, Logger};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HttpCode {
    // 2xx Success
    OK = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,

    // 3xx Redirection
    MovedPermanently = 301,
    Found = 302,
    NotModified = 304,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // 4xx Client Errors
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    UnsupportedMediaType = 415,
    TooManyRequests = 429,

    // 5xx Server Errors
    InternalError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
}

impl HttpCode {
    pub fn to_num(&self) -> u16 {
        self.to_owned() as u16
    }

    pub fn from_num(num: u16) -> Option<Self> {
        match num {
            // 2xx Success
            200 => Some(HttpCode::OK),
            201 => Some(HttpCode::Created),
            202 => Some(HttpCode::Accepted),
            204 => Some(HttpCode::NoContent),

            // 3xx Redirection
            301 => Some(HttpCode::MovedPermanently),
            302 => Some(HttpCode::Found),
            304 => Some(HttpCode::NotModified),
            307 => Some(HttpCode::TemporaryRedirect),
            308 => Some(HttpCode::PermanentRedirect),

            // 4xx Client Errors
            400 => Some(HttpCode::BadRequest),
            401 => Some(HttpCode::Unauthorized),
            402 => Some(HttpCode::PaymentRequired),
            403 => Some(HttpCode::Forbidden),
            404 => Some(HttpCode::NotFound),
            405 => Some(HttpCode::MethodNotAllowed),
            406 => Some(HttpCode::NotAcceptable),
            409 => Some(HttpCode::Conflict),
            410 => Some(HttpCode::Gone),
            411 => Some(HttpCode::LengthRequired),
            412 => Some(HttpCode::PreconditionFailed),
            413 => Some(HttpCode::PayloadTooLarge),
            415 => Some(HttpCode::UnsupportedMediaType),
            429 => Some(HttpCode::TooManyRequests),

            // 5xx Server Errors
            500 => Some(HttpCode::InternalError),
            501 => Some(HttpCode::NotImplemented),
            502 => Some(HttpCode::BadGateway),
            503 => Some(HttpCode::ServiceUnavailable),
            504 => Some(HttpCode::GatewayTimeout),

            _ => None,
        }
    }
}

pub type NetResult<T> = Result<T, NetError>;

#[derive(Clone, Debug)]
pub struct NetError {
    pub erc: HttpCode,
    pub details: String,
}

impl NetError {
    pub fn new(erc: HttpCode, details: Option<String>) -> Self {
        let details_x = if details.is_some() {
            details.unwrap()
        } else {
            "No details provided".to_string()
        };
        Self {
            erc,
            details: details_x,
        }
    }
}

impl Display for NetError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

pub type DogResult<T> = Result<T, DogError>;

#[derive(Clone, Debug)]
pub struct DogError {
    pub name: String,
    pub details: String,
    logger: Logger,
    log_level: LogLevel,
}

impl DogError {
    pub fn __fmtx(&self) -> String {
        format!("NetPup Error -> {}: {}", self.name, self.details)
    }

    pub fn new(logger: &Logger, name: String, details: String) -> Self {
        let mut s = Self {
            name,
            details,
            logger: logger.to_owned(),
            log_level: LogLevel::ERROR,
        };
        s.print();
        s
    }

    pub fn fatal(logger: Logger, name: String, details: String) -> Self {
        let mut s = Self {
            name,
            details,
            logger,
            log_level: LogLevel::FATAL,
        };
        s.print();
        Self::__terminate();
        s
    }

    pub fn __terminate() {
        std::process::exit(1)
    }

    pub fn print(&mut self) {
        self.logger.log(self.log_level.clone(), &*self.__fmtx())
    }
}

impl Display for DogError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NetPup Error -> {}: {}", self.name, self.details)
    }
}
