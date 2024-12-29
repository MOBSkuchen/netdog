use std::cmp::PartialEq;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpCode {
    OK = 200,
    BAD_REQUEST = 400,
    UNAUTHORIZED = 401,
    FORBIDDEN = 403,
    NOT_FOUND = 404,
    METHOD_NOT_ALLOWD = 405,
    INTERNAL_ERROR = 500,
    NETDOG_ERROR
}

pub type NtResult<T> = Result<T, NtError>;

#[derive(Clone)]
pub struct NtError {
    pub erc: HttpCode,
    details: String
}

impl NtError {
    pub fn new(erc: HttpCode, details: Option<String>) -> Self {
        let details_x = if details.is_some() {details.unwrap()} else {"No details provided".to_string()};
        Self {erc, details: details_x}
    }

    pub fn to_erf(&self) -> (HttpCode, String) {
        let mut erc = (self.erc.clone(), self.details.clone());
        if erc.0 == HttpCode::NETDOG_ERROR {erc.0 = HttpCode::INTERNAL_ERROR}
        erc
    }
}

impl fmt::Display for NtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.erc == HttpCode::NETDOG_ERROR {
            write!(f, "Internal error in NetDog -> {}", self.details).expect("Panicked!");
            return Ok(());
        }
        write!(f, "Other-side-error")
    }
}
