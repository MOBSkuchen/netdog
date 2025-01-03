use std::cmp::PartialEq;
use std::fmt;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::logger::{LogLevel, Logger};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HttpCode {
    OK = 200,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalError = 500,
}

impl HttpCode {
    pub fn to_num(&self) -> u16 {
        self.to_owned() as u16
    }
    
    pub fn from_num(num: u16) -> Option<Self> {
        match num {
            200 => Some(HttpCode::OK),
            400 => Some(HttpCode::BadRequest),
            401 => Some(HttpCode::Unauthorized),
            403 => Some(HttpCode::Forbidden),
            404 => Some(HttpCode::NotFound),
            405 => Some(HttpCode::MethodNotAllowed),
            500 => Some(HttpCode::InternalError),
            _ => None
        }
    }
}

pub type NetResult<T> = Result<T, NetError>;

#[derive(Clone)]
#[derive(Debug)]
pub struct NetError {
    pub erc: HttpCode,
    pub details: String
}

impl NetError {
    pub fn new(erc: HttpCode, details: Option<String>) -> Self {
        let details_x = if details.is_some() {details.unwrap()} else {"No details provided".to_string()};
        Self {erc, details: details_x}
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
    log_level: LogLevel
}

impl DogError {
    pub fn __fmtx(&self) -> String {
        format!("NetDog Error -> {}: {}", self.name, self.details)
    }
    
    pub fn new(logger: Logger, name: String, details: String) -> Self {
        let mut s = Self {name, details, logger, log_level: LogLevel::ERROR};
        s.print();
        s
    }
    
    pub fn fatal(logger: Logger, name: String, details: String) -> Self {
        let mut s = Self {name, details, logger, log_level: LogLevel::FATAL};
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
        write!(f, "NetDog Error -> {}: {}", self.name, self.details)
    }
}