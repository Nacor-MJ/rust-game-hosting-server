use std::{
    collections::HashMap,
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    process::Command,
};
use serde::{Serialize, Deserialize};
use hostable_servers::{minecraft::MinecraftServer, HostableServer, HostableServerHashed};
use http::{Content, Message, Variant};

pub mod hostable_servers;
pub mod http;

#[derive(Serialize, Deserialize)]
pub struct WebServer{
    ip: String,
    port: usize,
}
impl WebServer{
    pub fn new(config_json: &str) {

        let config: WebServer = serde_json::from_str(config_json).expect("Failed to Parse the Config file");

        let ip_and_port = format!(
            "{}:{}",
            config.ip,
            config.port,
        );

        println!("Http://{}/", ip_and_port);

        let mut hostable_servers: HostableServerHashed = HashMap::new();

        let mut minecraft_server = MinecraftServer::new();
        minecraft_server.start().unwrap();
        hostable_servers.insert("minecraft", &mut minecraft_server);

        let listener = TcpListener::bind(ip_and_port).expect("The server is already running");

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            handle_connection(stream, &mut hostable_servers).unwrap_or_else(|e| {
                println!("Connection Failed: {}", e);
            });
        }
    }
}


fn handle_connection(
    mut stream: TcpStream,
    hostable_servers: &mut HostableServerHashed,
) -> std::io::Result<()> {
    let mut buffer = vec![0; 1024];
    stream.read(&mut buffer)?;

    let buffer_str = match std::str::from_utf8(&buffer) {
        Ok(ok) => ok.to_string(),
        Err(e) => e.to_string(),
    };
    let mut buffer_iter = buffer_str.split(' ');

    let method = buffer_iter.next().unwrap_or_default();
    let link = buffer_iter.next().unwrap_or_default();

    println!("----------------------------------------------------------------");
    println!("Time: {}", chrono::Local::now());
    println!(
        "Peer: '{:?}', Method: '{}', Link: '{}'",
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
        let vec: Vec<u8> = http_header
            .to_owned()
            .into_iter()
            .chain(body.to_owned().into_iter())
            .collect();

        stream.write_all(&vec)?;
    } else {
        println!("ᓚᘏᗢ\r\n{:#?}\r\nᓚᘏᗢ", htttp_response);

        let response_str = htttp_response.to_string();
        let response = response_str.as_bytes();

        dbg!(&response_str);

        stream.write_all(response)?;
    };

    stream.flush()?;

    println!("----------------------------------------------------------------");

    Ok(())
}

fn parse_http_request(
    method: &str,
    link: &str,
    hostable_servers: &mut HashMap<&str, &mut dyn HostableServer>,
) -> Message {
    match method {
        "GET" => match link {
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

                let first_domain = link_split.next().unwrap();

                return if first_domain == "file" {
                    if let Some(file_path) = link_split.next() {
                        match fs::read_to_string(file_path) {
                            Ok(file_text) => Message::new(Variant::Ok, Content::File(file_text)),
                            Err(e) => Message::new(
                                Variant::InternalServerError,
                                Content::Text(e.to_string()),
                            ),
                        }
                    } else {
                        Message::new(
                            Variant::NotFound,
                            Content::Text(format!("File Not Found: {}", link)),
                        )
                    }
                } else if let Some(hostable_server) = hostable_servers.get_mut(first_domain) {
                    let second_domain = link_split.next().unwrap();

                    match second_domain {
                        "start" => match hostable_server.start() {
                            Ok(_) => Message::default(),
                            Err(e) => Message::new(
                                Variant::InternalServerError,
                                Content::Text(e.to_string()),
                            ),
                        },
                        "stop" => match hostable_server.stop() {
                            Ok(_) => Message::default(),
                            Err(e) => Message::new(
                                Variant::InternalServerError,
                                Content::Text(e.to_string()),
                            ),
                        },
                        "get_status" => {
                            match hostable_server.update_status() {
                                // succesfull update now send the message :)
                                Ok(_) => {
                                    let json = hostable_server.to_json();
                                    match json {
                                        Ok(ok) => {
                                            Message::new(
                                                Variant::Ok, 
                                                Content::Struct(ok)
                                            )
                                        },
                                        Err(e) => Message::internal_server_error(e.to_string()),
                                    }
                                }
                                Err(e) => Message::internal_server_error(e.to_string()),
                            }
                        }
                        _ => Message::new(
                            Variant::ServiceUnavailable,
                            Content::Text(format!("Unkown link: {link}",)),
                        ),
                    }
                } else {
                    Message::new(
                        Variant::ServiceUnavailable,
                        Content::Text(format!("Unkown link: {link}",)),
                    )
                };
            }
        },
        "POST" => match link {
            "/Shutdown" => shutdown_server(),
            "/Ping" => Message::new(Variant::Ok, Content::Text("Ping succesfull".to_owned())),
            e => {
                println!("Link not accesible: {e}");
                Message::new(Variant::ServiceUnavailable, Content::Text(String::from(e)))
            }
        },
        e => {
            println!("Method not available: {e}");
            Message::new(Variant::ServiceUnavailable, Content::Text(String::from(e)))
        }
    }
}
fn shutdown_server() -> Message {
    let output = Command::new("shutdown").arg("now").output();

    println!("Shutdown:\r\n{:#?}", output);

    let output = match output {
        Ok(ok) => ok,
        Err(e) => {
            return Message::new(
                Variant::ServiceUnavailable,
                Content::Text(format!("{:?}", e)),
            );
        }
    };

    let status = output.clone().status.success();
    if status {
        Message::new(
            Variant::Ok,
            Content::Text("Shutting the server down".to_owned()),
        )
    } else {
        Message::new(
            Variant::ServiceUnavailable,
            Content::Text(format!("{:?}", output)),
        )
    }
}