use std::cmp::PartialEq;
use std::fmt;
use crate::errors::HttpError::INTERNAL_ERROR;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpError {
    OK = 200,
    BAD_REQUEST = 400,
    UNAUTHORIZED = 401,
    FORBIDDEN = 403,
    NOT_FOUND = 404,
    METHOD_NOT_ALLOWD = 405,
    INTERNAL_ERROR = 500,
}

pub type NtResult<T> = Result<T, NtError>;

#[derive(Debug, Clone)]
pub struct NtError {
    pub erc: HttpError,
    details: String
}

impl NtError {
    pub fn new(erc: HttpError, details: Option<String>) -> Self {
        let details_x = if details.is_some() {details.unwrap()} else {"No details provided".to_string()};
        Self {erc, details: details_x}
    }
}

impl fmt::Display for NtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.erc == INTERNAL_ERROR {
            write!(f, format!("Internal error in NetDog -> {}", self.details)).expect("Panicked!");
            return Ok(());
        }
        write!(f, "Other-side-error")
    }
}
