use std::cmp::PartialEq;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpCode {
    OK = 200,
    BAD_REQUEST = 400,
    UNAUTHORIZED = 401,
    FORBIDDEN = 403,
    NOT_FOUND = 404,
    METHOD_NOT_ALLOWD = 405,
    INTERNAL_ERROR = 500,
}

impl HttpCode {
    pub fn to_num(&self) -> u16 {
        self.to_owned() as u16
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

    pub fn to_erf(&self) -> (HttpCode, String) {
        (self.erc.clone(), self.details.clone())
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
    pub details: String
}

impl DogError {
    fn __fmtx(&self) -> String {
        format!("NetDog Error -> {}: {}", self.name, self.details)
    }
    pub fn new(name: String, details: String) -> Self {
        let s = Self {name, details};
        println!("{}", s.__fmtx());
        // s.__terminate();
        s
    }
    
    pub fn __terminate(&self) {
        std::process::exit(1)
    }
    
    pub fn print(&self) {
        println!("{}", self.__fmtx())
    }
}

impl Display for DogError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NetDog Error -> {}: {}", self.name, self.details)
    }
}