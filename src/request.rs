use std::collections::HashMap;
use crate::errors::HttpCode::BAD_REQUEST;
use crate::errors::{NtError, NtResult};

enum Methods {
    GET,
    POST
}
pub type Headers = HashMap<String, String>;

fn split_once(in_string: &str) -> Result<(&str, &str), NtError> {
    let mut splitter = in_string.splitn(2, ": ");
    if splitter.clone().count() < 2 {return Err(NtError::new(BAD_REQUEST, None))}
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    Ok((first, second))
}

pub struct HttpRequest {
    pub method: Methods,
    protocol_v: String,
    pub path: String,
    pub headers: Headers,
    pub body: Vec<u8>
}

impl HttpRequest {
    pub fn new(method: Methods,
               protocol_v: String,
               path: String) -> Self {
        Self {method, protocol_v, path, headers: Default::default(), body: vec![]}
    }

    pub fn mk_headers(lns: Vec<String>) -> NtResult<Headers> {
        let mut hsm = Headers::new();

        for ln in lns {
            let x = split_once(&ln)?;
            hsm.insert(x.0.to_string(), x.1.to_string());
        }

        Ok(hsm)
    }

    pub fn from_raw(mut req_lines: Vec<String>) -> NtResult<Self> {
        if (&req_lines).is_empty() { return Err(NtError::new(BAD_REQUEST, None)) }
        let head_line = (&req_lines)[0].clone();
        let head_line_v = head_line.split(" ").collect::<Vec<_>>();
        if head_line_v.clone().len() != 3 {return Err(NtError::new(BAD_REQUEST, None))}

        let method = match head_line_v[0].to_uppercase().as_str() {
            "GET" => Methods::GET,
            "POST" => Methods::POST,
            _ => {return Err(NtError::new(BAD_REQUEST, None))}
        };

        let path = head_line_v[1].to_string();
        let protocol_v = head_line_v[2].to_string();

        req_lines.remove(0);
        req_lines.remove(1);

        let headers = Self::mk_headers(req_lines)?;

        Ok(Self {method, protocol_v, path, headers, body: vec![]})
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body
    }
}