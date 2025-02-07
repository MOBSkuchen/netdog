use crate::errors::{DogError, HttpCode, NetError};
use crate::logger::Logger;
use crate::request::Headers;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::TcpStream;

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum ContentType {
    HTML,
    JSON,
    XML,
    PLAIN,
    CSS,
    JAVASCRIPT,
    JPEG,
    PNG,
    GIF,
    BMP,
    SVG,
    WEBP,
    MP3,
    MP4,
    WAV,
    OGG,
    AVI,
    PDF,
    ZIP,
    TAR,
    GZIP,
    BZIP2,
    WEBM,
    ICO,
    NONE,
    UNKNOWN,
}

impl ContentType {
    pub fn to_string(&self) -> String {
        match self {
            ContentType::HTML => "text/html",
            ContentType::JSON => "application/json",
            ContentType::XML => "application/xml",
            ContentType::PLAIN => "text/plain",
            ContentType::CSS => "text/css",
            ContentType::JAVASCRIPT => "application/javascript",
            ContentType::JPEG => "image/jpeg",
            ContentType::PNG => "image/png",
            ContentType::GIF => "image/gif",
            ContentType::BMP => "image/bmp",
            ContentType::SVG => "image/svg+xml",
            ContentType::WEBP => "image/webp",
            ContentType::MP3 => "audio/mpeg",
            ContentType::MP4 => "video/mp4",
            ContentType::WAV => "audio/wav",
            ContentType::OGG => "audio/ogg",
            ContentType::AVI => "video/x-msvideo",
            ContentType::PDF => "application/pdf",
            ContentType::ZIP => "application/zip",
            ContentType::TAR => "application/x-tar",
            ContentType::GZIP => "application/gzip",
            ContentType::BZIP2 => "application/x-bzip2",
            ContentType::WEBM => "video/webm",
            ContentType::ICO => "image/x-icon",
            ContentType::NONE => "",
            ContentType::UNKNOWN => "application/octet-stream",
        }
        .to_string()
    }

    pub fn from_ext(ext: &str) -> ContentType {
        match ext.to_lowercase().as_str() {
            "html" | "htm" => ContentType::HTML,
            "json" => ContentType::JSON,
            "xml" => ContentType::XML,
            "txt" => ContentType::PLAIN,
            "css" => ContentType::CSS,
            "js" => ContentType::JAVASCRIPT,
            "jpeg" | "jpg" => ContentType::JPEG,
            "png" => ContentType::PNG,
            "gif" => ContentType::GIF,
            "bmp" => ContentType::BMP,
            "svg" => ContentType::SVG,
            "webp" => ContentType::WEBP,
            "mp3" => ContentType::MP3,
            "mp4" => ContentType::MP4,
            "wav" => ContentType::WAV,
            "ogg" => ContentType::OGG,
            "avi" => ContentType::AVI,
            "pdf" => ContentType::PDF,
            "zip" => ContentType::ZIP,
            "tar" => ContentType::TAR,
            "gz" => ContentType::GZIP,
            "bz2" => ContentType::BZIP2,
            "webm" => ContentType::WEBM,
            "ico" => ContentType::ICO,
            "" => ContentType::NONE,
            _ => { ContentType::UNKNOWN }
        }
    }

    pub fn from_file_name(file_name: &str) -> ContentType {
        if !file_name.contains(".") {
            return ContentType::UNKNOWN;
        }
        let mut splitter = file_name.splitn(2, ".");
        splitter.next().unwrap();
        Self::from_ext(splitter.next().unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpResponse {
    protocol_v: String,
    response: (HttpCode, String),
    headers: Headers,
    content: (Vec<u8>, String),
    has_content: bool,
    pub reroute: bool,
}

impl HttpResponse {
    pub fn new(
        response: (HttpCode, String),
        headers: Headers,
        content: (Vec<u8>, String),
        reroute: bool,
    ) -> Self {
        let mut header_c = headers.clone();
        if content.1 != "" {
            header_c.insert("Content-Length".to_string(), content.0.len().to_string());
            header_c.insert("Content-Type".to_string(), content.1.clone());
        }
        Self {
            protocol_v: "HTTP/1.1".to_string(),
            response,
            headers: header_c,
            content: (&content).to_owned(),
            has_content: content.1 != "",
            reroute,
        }
    }

    pub fn to_net_error(&self) -> NetError {
        NetError::new(self.response.0.clone(), Some(self.response.1.clone()))
    }

    pub fn make(&self) -> Vec<u8> {
        let content_vecu8 = &self.content;
        let mut r = format!(
            "{} {:?} {}",
            self.protocol_v,
            self.response.0.to_num(),
            self.response.1
        );
        r += "\r\n";
        for header in &self.headers {
            r += format!("\n{}: {}", header.0, header.1).as_str()
        }
        if self.has_content {
            r += "\n\n";
        }
        [r.into_bytes(), content_vecu8.0.clone()].concat()
    }

    fn __send(&self, mut stream: &TcpStream) -> Result<(), ()> {
        if stream.write(self.make().as_ref()).is_err() {
            return Err(());
        }
        if stream.flush().is_err() {
            return Err(());
        }
        Ok(())
    }

    pub fn send(&self, logger: &Logger, stream: &TcpStream) {
        if self.__send(&stream).is_err() {
            DogError::new(
                &logger,
                "con-sendfail-sr".to_string(),
                "Error while sending response to client".to_string(),
            )
            .print();
        }
    }
}
