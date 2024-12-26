use std::fmt;
use std::ptr::write;

enum HttpError {
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
    my_fault: bool,
    details: String
}

impl NtError {
    pub fn new(my_fault: bool, details: Option<String>) -> Self {
        let details_x = if details.is_some() {details.unwrap()} else {"No details provided".to_string()};
        Self {my_fault, details: details_x}
    }
}

impl fmt::Display for NtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.my_fault {
            write!(f, format!("Internal error in NetDog -> {}", self.details)).expect("Panicked!");
            return Ok(());
        }
        write!(f, "Other-side-error")
    }
}
