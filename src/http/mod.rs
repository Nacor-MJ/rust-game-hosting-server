/// Message meant to be sent over Http
#[derive(Debug)]
pub struct Message {
    pub variant: Variant,
    pub content: Content,
}
impl Message {
    pub fn new(variant: Variant, content: Content) -> Self {
        Self { variant, content }
    }
    pub fn internal_server_error(e: String) -> Self {
        Message::new(Variant::InternalServerError, Content::Text(e))
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
        let mut response = match self.variant {
            Variant::Ok => "HTTP/1.1 200 OK\r\n",
            Variant::ServiceUnavailable => "HTTP/1.1 503 Service Unavailable\r\n",
            Variant::NotFound => "HTTP/1.1 404 NOT FOUND\r\n",
            Variant::InternalServerError => "500 Internal Server Error\r\n",
        }
        .to_owned();

        // header
        response += match self.content {
            Content::Struct(_) => "\r\n", // Content-Type: application/json
            _ => "\r\n",
        };

        // response
        response += self.content.to_string().as_str();

        // finished
        response
    }
}

#[derive(Debug)]
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
            Content::File(filename) => {
                let file_content = match std::fs::read_to_string(filename) {
                    Ok(ok) => ok,
                    Err(e) => {
                        return Message::new(Variant::NotFound, Content::Text(e.to_string()))
                            .to_string()
                    }
                };
                file_content
            }
            Content::Text(txt) | Content::Struct(txt) => txt.clone(),
            Content::Empty => String::new(),
            Content::RawBytes(_) => String::from("RawBytes is not meant to be displayed ᓚᘏᗢ"),
        }
    }
}
