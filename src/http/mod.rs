/// Message meant to be sent over Http
#[derive(Debug)]
pub struct Message {
    pub variant: Variant,
    pub content: Content,
}
impl Message {
    #[must_use]
    pub const fn new(variant: Variant, content: Content) -> Self {
        Self { variant, content }
    }
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
        message += self.content.to_string().as_str();

        // finished
        message
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Variant {
    Ok,                  // "HTTP/1.1 200 OK\r\n\r\n"
    ServiceUnavailable,  // "HTTP/1.1 503 Service Unavailable\r\n\r\n"
    NotFound,            // "HTTP/1.1 404 Not Found\r\n\r\n"
    InternalServerError, // "HTTP/1.1 500 Internal Server Error\r\n\r\n"
}

/// Represents possible contents of an HTTP response
#[derive(Debug)]
pub enum Content {
    File(String),
    Text(String),
    Struct(String),
    RawBytes(Box<[u8]>),
    Empty,
}

impl std::string::ToString for Content {
    /// This Should Not Be Used for `HttpContent::RawBytes` with any other purpose  
    /// then displaying
    fn to_string(&self) -> String {
        match self {
            Self::File(filename) => match std::fs::read_to_string(filename) {
                Ok(ok) => ok,
                Err(e) => Message::new(Variant::NotFound, Self::Text(e.to_string())).to_string(),
            },
            Self::Text(txt) | Self::Struct(txt) => txt.clone(),
            Self::Empty => String::new(),
            Self::RawBytes(_) => String::from("RawBytes is not meant to be displayed ᓚᘏᗢ"),
        }
    }
}
