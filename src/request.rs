use crate::errors::HttpCode::BadRequest;
use crate::errors::{NetError, NetResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Methods {
    GET,
    POST,
}

impl Methods {
    pub fn from_str(s: &str) -> Result<Self, ()> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Methods::GET),
            "POST" => Ok(Methods::POST),
            _ => Err(()),
        }
    }

    pub fn from_str_mult(sv: Vec<&str>) -> Result<Vec<Methods>, ()> {
        let mut ret: Vec<Methods> = vec![];
        for s in sv {
            let i = Self::from_str(s)?;
            ret.push(i);
        }
        Ok(ret)
    }
}

pub type Headers = HashMap<String, String>;

fn split_once(in_string: &str) -> Result<(&str, &str), NetError> {
    let mut splitter = in_string.splitn(2, ": ");
    let first = splitter.next();
    let second = splitter.next();
    if first.is_none() || second.is_none() {
        Err(NetError::new(BadRequest, None))
    } else {
        Ok((first.unwrap(), second.unwrap()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: Methods,
    protocol_v: String,
    pub path: String,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub host: Option<String>,
}

impl HttpRequest {
    /*
    pub fn new(method: Methods,
               protocol_v: String,
               path: String) -> Self {
        Self {method, protocol_v, path, headers: Default::default(), body: vec![], host: None}
    }
    */

    pub fn format(&self) -> String {
        format!(
            "{:?} {} ({})",
            self.method,
            self.path,
            self.host
                .clone()
                .or_else(|| { Some("None".to_string()) })
                .unwrap()
        )
    }

    pub fn mk_headers(lns: Vec<String>) -> NetResult<Headers> {
        let mut hsm = Headers::new();

        for ln in lns {
            let x = split_once(&ln)?;
            hsm.insert(x.0.to_string(), x.1.to_string());
        }

        Ok(hsm)
    }

    pub fn from_raw(mut req_lines: Vec<String>) -> NetResult<Self> {
        if (&req_lines).is_empty() {
            return Err(NetError::new(BadRequest, None));
        }
        let head_line = &(&req_lines)[0];
        let head_line_v = head_line.split(" ").collect::<Vec<_>>();
        if head_line_v.len() != 3 {
            return Err(NetError::new(BadRequest, None));
        }

        let method_r = Methods::from_str(head_line_v[0].to_uppercase().as_str());
        if method_r.is_err() {
            return Err(NetError::new(BadRequest, None));
        }
        let method = method_r.unwrap();

        let path = head_line_v[1].to_string();
        let protocol_v = head_line_v[2].to_string();

        req_lines.remove(0);

        let headers = Self::mk_headers(req_lines)?;
        let host = headers.get("Host").cloned();

        Ok(Self {
            method,
            protocol_v,
            path,
            headers,
            body: vec![],
            host,
        })
    }
}
