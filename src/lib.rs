use hostable_servers::{minecraft::ServerInfo, HostableServer, HostableServerHashed};
use http::{Content, Message, Variant};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    process::Command,
};

pub mod hostable_servers;
pub mod http;


#[derive(Serialize, Deserialize)]
pub struct WebServer {
    ip: String,
    port: usize,
}
impl WebServer {
    /// Starts the web server and  starts listening for connections
    /// # Panics
    /// Panics if the `config_json` is not parsable into the `WebServer` struct
    pub fn start(config_json: &str) {
        let config: Self =
            serde_json::from_str(config_json).expect("Failed to Parse the Config file");

        let ip_and_port = format!("{}:{}", config.ip, config.port,);

        println!("Http://{ip_and_port}/");

        let mut hostable_servers: HostableServerHashed = HashMap::new();

        let mut minecraft_server = ServerInfo::new();

        hostable_servers.insert("minecraft", &mut minecraft_server);

        let listener = TcpListener::bind(ip_and_port).expect("The server is already running");

        for stream in listener.incoming() {
            let stream = stream.expect("Never happens");
            handle_connection(stream, &mut hostable_servers).unwrap_or_else(|e| {
                println!("Connection Failed: {e}");
            });
        }
    }
}


fn handle_connection(
    mut stream: TcpStream,
    hostable_servers: &mut HostableServerHashed,
) -> std::io::Result<()> {
    let mut buffer = vec![0; 1024];

    if let Err(e) = stream.read(&mut buffer) {
        println!("Error with reading the stream: {e}");
        return Err(e);
    };

    let buffer_str = match std::str::from_utf8(&buffer) {
        Ok(ok) => ok.to_string(),
        Err(e) => e.to_string(),
    };
    let mut buffer_iter = buffer_str.split(' ');

    let method = buffer_iter.next().unwrap_or_default();
    let link = buffer_iter.next().unwrap_or_default();

    println!("\x1b[35m========================================================\x1b[39m");
    println!("\x1b[36mTime: {}\x1b[39m", chrono::Local::now());
    println!(
        "\x1b[36mPeer: '{:?}', Method: '{}', Link: '{}'\x1b[39m",
        stream.peer_addr(),
        method,
        link,
    );

    let htttp_response: Message = parse_http_request(method, link, hostable_servers);

    // I know this sucks but I can't convince the borrow checker to work with me :(
    if let Content::RawBytes(bytes) = htttp_response.content {
        println!("ᓚᘏᗢ\r\nfavicon.ico \r\nᓚᘏᗢ");
        let http_header = b"HTTP/1.1 200 OK\r\n\r\n";
        let body: &[u8] = &bytes;
        // magic
        let vec: Vec<u8> = http_header.iter().copied().chain(body.to_owned()).collect();

        stream.write_all(&vec)?;
    } else {
        if htttp_response.variant == Variant::Ok {
            println!("\x1b[32mᓚᘏᗢ\r\n{htttp_response:#?}\r\nᓚᘏᗢ\x1b[39m");
        } else {
            eprintln!("\x1b[31mServer Error: \r\n{htttp_response:#?}\x1b[39m");
        }

        let response_str = htttp_response.to_string();
        let response = response_str.as_bytes();

        dbg!(&response_str);

        stream.write_all(response)?;
    };

    stream.flush()?;

    println!("\x1b[35m========================================================\x1b[39m");

    Ok(())
}

fn parse_http_request(
    method: &str,
    link: &str,
    hostable_servers: &mut HashMap<&str, &mut dyn HostableServer>,
) -> Message {
    match method {
        "GET" => parse_get(link, hostable_servers),
        "POST" => parse_post(link, hostable_servers),
        e => {
            println!("Method not available: {e}");
            Message::new(
                Variant::NotFound,
                Content::Text(format!("Unkown POST link: {e}")),
            )
        }
    }
}

fn parse_post(link: &str, hostable_servers: &mut HashMap<&str, &mut dyn HostableServer>) -> Message {
    match link {
        "/Shutdown" => shutdown_server(),
        "/Ping" => Message::new(Variant::Ok, Content::Text("Ping succesfull".to_owned())),
        link => {
            let mut link_split = link.split('/');
            let _ = link_split.next(); // the link starts with '/'

            let first_domain = link_split.next().unwrap_or("Unavailabe");

            return hostable_servers.get_mut(first_domain).map_or_else(
                || {
                    Message::new(
                        Variant::NotFound,
                        Content::Text(format!("Unkown GET link: {link}",)),
                    )
                },
                |hostable_server| {
                    let second_domain = link_split.next().unwrap_or("Unavaiable");

                    match second_domain {
                        "start" => match hostable_server.start() {
                            Ok(()) => Message::default(),
                            Err(e) => Message::new(
                                Variant::InternalServerError,
                                Content::Text(e.to_string()),
                            ),
                        },
                        "stop" => match hostable_server.stop() {
                            Ok(()) => Message::default(),
                            Err(e) => Message::new(
                                Variant::InternalServerError,
                                Content::Text(e.to_string()),
                            ),
                        },
                        e => {
                            println!("Link not accesible: {e}");
                            Message::new(
                                Variant::NotFound,
                                Content::Text(format!("Unkown GET link: {link}",)),
                            )
                        }
                    }
                },
            );
        }
    }
}

fn parse_get(link: &str, hostable_servers: &mut HashMap<&str, &mut dyn HostableServer>) -> Message {
    match link {
        "/" => Message {
            variant: Variant::Ok,
            content: Content::File(String::from("hello.html")),
        },
        "/favicon.ico" => Message {
            variant: Variant::Ok,
            content: Content::RawBytes(Box::new(include_bytes!("../favicon.ico").to_owned())),
        },
        link => {
            let mut link_split = link.split('/');
            let _ = link_split.next(); // the link starts with '/'

            let first_domain = link_split.next().unwrap_or("Unavailable");

            return if first_domain == "file" {
                link_split.next().map_or_else(
                    || {
                        Message::new(
                            Variant::NotFound,
                            Content::Text(format!("File Not Found: {link}")),
                        )
                    },
                    |file_path| match fs::read_to_string(file_path) {
                        Ok(file_text) => Message::new(Variant::Ok, Content::File(file_text)),
                        Err(e) => Message::new(
                            Variant::InternalServerError,
                            Content::Text(e.to_string()),
                        ),
                    },
                )
            } else if let Some(hostable_server) = hostable_servers.get_mut(first_domain) {
                let second_domain = link_split.next().unwrap_or("Unavailable");

                match second_domain {
                    "get_status" => {
                        match hostable_server.update_status() {
                            // succesfull update now send the message :)
                            Ok(()) => {
                                let json = hostable_server.to_json();
                                match json {
                                    Ok(ok) => Message::new(Variant::Ok, Content::Struct(ok)),
                                    Err(e) => Message::internal_server_error(e.to_string()),
                                }
                            }
                            Err(e) => Message::internal_server_error(e.to_string()),
                        }
                    }
                    e => {
                        println!("Link not accesible: {e}");
                        Message::new(
                            Variant::NotFound,
                            Content::Text(format!("Unkown GET link: {link}",)),
                        )
                    }
                }
            } else {
                println!("Link not accesible: {first_domain}");

                Message::new(
                    Variant::NotFound,
                    Content::Text(format!("Unkown GET link: {link}",)),
                )
            };
        }
    }
}
fn shutdown_server() -> Message {
    let output = Command::new("shutdown").arg("now").output();

    println!("Shutdown:\r\n{output:#?}");

    let output = match output {
        Ok(ok) => ok,
        Err(e) => {
            return Message::new(Variant::ServiceUnavailable, Content::Text(format!("{e:?}")));
        }
    };

    let status = output.status.success();
    if status {
        Message::new(
            Variant::Ok,
            Content::Text("Shutting the server down".to_owned()),
        )
    } else {
        Message::new(
            Variant::ServiceUnavailable,
            Content::Text(format!("{output:?}")),
        )
    }
}
