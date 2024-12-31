use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use crate::errors::{DogError, DogResult, HttpCode, NetError};
use crate::request::Headers;

#[derive(Clone, Eq, PartialEq)]
pub enum ContentType {
    HTML,
    NONE
}

pub struct HttpResponse {
    protocol_v: String,
    response: (HttpCode, String),
    headers: Headers,
    content: (Vec<u8>, ContentType),
    has_content: bool
}

fn ct_to_str(ct: ContentType) -> String {
    match ct {
        ContentType::HTML => {"text/html".to_string()}
        ContentType::NONE => {"".to_string()}
    }
}

impl HttpResponse {
    pub fn new(response: (HttpCode, String),
               headers: Headers,
               content: (Vec<u8>, ContentType)) -> Self {
        let mut header_c = headers.clone();
        if content.1 != ContentType::NONE {
            header_c.insert("Content-Length".to_string(), content.0.len().to_string());
            header_c.insert("Content-Type".to_string(), ct_to_str(content.1.clone()));
        }
        Self {protocol_v: "HTTP/1.1".to_string(), response, headers: header_c, content: (&content).to_owned(), has_content: content.1 != ContentType::NONE}
    }

    pub fn make(&self) -> Vec<u8> {
        let content_vecu8 = &self.content;
        let mut r = format!("{} {:?} {}", self.protocol_v, self.response.0.to_num(), self.response.1);
        for header in &self.headers {
            r += format!("\n{}: {}", header.0, header.1).as_str()
        }
        if self.has_content {
            r += "\n\n";
        }
        [r.into_bytes(), content_vecu8.0.clone()].concat()
    }
    
    fn __send(&self, mut stream: &TcpStream) -> Result<(), ()> {
        if stream.write(self.make().as_ref()).is_err() {return Err(())}
        if stream.flush().is_err() {return Err(())}
        Ok(())
    }
    
    pub fn send(&self, stream: &TcpStream) {
        if self.__send(&stream).is_err() {
            DogError::new("con-sendfail-sr".to_string(), "Error while sending response to client".to_string()).print();
        }
    }
}