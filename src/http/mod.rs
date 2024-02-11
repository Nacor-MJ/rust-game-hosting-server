//! Describes all the fun stuff that has something to do with HTTP requests

use std::io;

/// Message meant to be sent over Http
#[derive(Debug)]
pub struct Message {
    /// Describes the HTTP response variant
    pub variant: Variant,
    /// Represents possible contents of an HTTP response
    pub content: Content,
}
impl Message {
    /// Returns a new Message
    #[must_use]
    pub const fn new(variant: Variant, content: Content) -> Self {
        Self { variant, content }
    }
    /// A 500 internal server error with the text content of `e` 
    #[must_use]
    pub const  fn internal_server_error(e: String) -> Self {
        Self::new(Variant::InternalServerError, Content::Text(e))
    }
}
impl Default for Message {
    fn default() -> Self {
        Self {
            variant: Variant::Ok,
            content: Content::Empty,
        }
    }
}
impl std::string::ToString for Message {
    fn to_string(&self) -> String {
        // status line
        let mut message = match self.variant {
            Variant::Ok => "HTTP/1.1 200 OK\r\n",
            Variant::ServiceUnavailable => "HTTP/1.1 503 Service Unavailable\r\n",
            Variant::NotFound => "HTTP/1.1 404 NOT FOUND\r\n",
            Variant::InternalServerError => "500 Internal Server Error\r\n",
        }
        .to_owned();

        // header
        message += match self.content {
            Content::Text(_) => "Content-Type: text/plain\r\n",
            _ => "\r\n",
        };

        // response
        return match self.content.to_string() {
            Ok(ok) => {
                message + &ok
            },
            Err(e) => {
                format!("HTTP/1.1 404 NOT FOUND \r\n Content-Type: text/plain\r\n {}", e)
            },
        };
    }
}


/// Describes the HTTP response variant
#[derive(Debug, PartialEq, Eq)]
pub enum Variant {
    /// 200 OK
    Ok,                  
    /// 503 Service Unavailable
    ServiceUnavailable,  
    /// 404 Not Found
    NotFound,            
    /// 500 Internal Server Error
    InternalServerError, 
}

/// Represents possible contents of an HTTP response
#[derive(Debug)]
pub enum Content {
    /// File to be read using [`std::fs::read_to_string()`]
    File(String),
    /// Generic text
    Text(String),
    /// Struct already parsed into json
    Struct(String),
    /// RawBytes, used to transfer file such as the favicon.ico
    RawBytes(Box<[u8]>),
    /// Notgin
    Empty,
}

impl Content {
    /// This Should Not Be Used for `HttpContent::RawBytes` with any other purpose  
    /// then displaying
    fn to_string(&self) -> io::Result<String> {
        match self {
            Self::File(filename) => std::fs::read_to_string(filename),
            Self::Text(txt) | Self::Struct(txt) => Ok(txt.clone()),
            Self::Empty => Ok(String::new()),
            Self::RawBytes(_) => Ok(String::from("RawBytes is not meant to be displayed ᓚᘏᗢ")),
        }
    }
}
