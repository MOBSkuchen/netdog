use crate::errors::{HttpCode, NtError};
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
        let mut r = format!("{} {:?} {}", self.protocol_v, self.response.0, self.response.1);
        for header in &self.headers {
            r += format!("\n{}: {}", header.0, header.1).as_str()
        }
        if self.has_content {
            r += "\n\n";
        }
        [r.into_bytes(), content_vecu8.0.clone()].concat()
    }

    pub fn from_error(nd_error: NtError) -> Self {
        HttpResponse::new(nd_error.to_erf(), Headers::new(), (vec![], ContentType::NONE))
    }
}