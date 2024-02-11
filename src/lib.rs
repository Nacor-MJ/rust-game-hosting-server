//! Simple Rust web server.
//!
//! Abstracts away the implementation details of a web server.
//! Servers that implement the [`HostableServer`] trait can be run on it

#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::all)]

use hostable_servers::{HostableServer, HostableServerHashed};
use http::{Content, Message, Variant};
use std::{
    collections::HashMap,
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    process::Command,
    str::from_utf8,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

pub mod hostable_servers;
pub mod http;

/// Starts the web server and starts listening for connections
///
/// `hostable_servers` is used to provide all available server
/// # Panics
/// Panics if the `port` and `ip` adress provided are already in use or are otherwise blocked
pub fn start(ip: &str, port: usize, hostable_servers: &mut HostableServerHashed) {
    // Ip adress setup
    let ip_and_port = format!("{ip}:{port}");

    println!("Http://{ip_and_port}/");

    // TCP setup
    let listener = TcpListener::bind(ip_and_port).expect("Problems with the IP and port");

    // timeout setup
    let (tx, timer_thread) = create_timeout_thread(Duration::from_secs(60 * 30));

    // accepting connections
    for stream in listener.incoming() {
        let stream = stream.expect("Never happens");
        handle_connection(stream, hostable_servers, &tx).unwrap_or_else(|e| {
            println!("Connection Failed: {e}");
        });
    }

    // IDK if I need this, but I always wanted to use drop somewhere
    // and when I put this here I know timer_thread will live long enough
    drop(timer_thread);
}

/// Creates a thread that monitors the activity of the server
///
/// Everytime the server gets a request it should send `()` to the Sender to let it know it is still alive.
/// After `allowed_idle_time` the server will attempt to shutdown by calling the `shutdown()` function
fn create_timeout_thread(allowed_idle_time: Duration) -> (Sender<()>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let timer_thread: JoinHandle<()> = thread::spawn(move || {
        loop {
            match rx.recv_timeout(allowed_idle_time) {
                Ok(()) => {}
                Err(_) => {
                    shutdown_server();
                }
            }
        }

        #[allow(unreachable_code)]
        // Just a fail safe
        {
            unreachable!("This would shut down the timer thread")
        }
    });
    (tx, timer_thread)
}

fn handle_connection(
    mut stream: TcpStream,
    hostable_servers: &mut HostableServerHashed,
    tx: &Sender<()>,
) -> std::io::Result<()> {
    let _ = tx.send(());

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
        let response_str = htttp_response.to_string();

        // This is neccesary because the File is loaded in the .to_string() method on the http_response
        if &response_str[0..12] == "HTTP/1.1 200" {
            println!("\x1b[32mᓚᘏᗢ\r\n{response_str}\r\nᓚᘏᗢ\x1b[39m");
        } else if &response_str[0..12] == "HTTP/1.1 404" {
            eprintln!("\x1b[31mServer Error: \r\n{response_str}\x1b[39m");
        } else {
            println!("\x1b[34mUnsure:\r\n{response_str}\r\nᓚᘏᗢ\x1b[39m");
        }

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
    hostable_servers: &mut HashMap<&str, Box<dyn HostableServer>>,
) -> Message {
    match method {
        "GET" => parse_get(link, hostable_servers),
        "POST" => parse_post(link, hostable_servers),
        e => {
            println!("Method not available: {e}");
            Message::new(
                Variant::NotFound,
                Content::Text(format!("Unkown method: {e}")),
            )
        }
    }
}

fn parse_post(
    link: &str,
    hostable_servers: &mut HashMap<&str, Box<dyn HostableServer>>,
) -> Message {
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
                        Content::Text(format!("Unkown POST link: {link}",)),
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
                                Content::Text(format!("Unkown POST link: {link}",)),
                            )
                        }
                    }
                },
            );
        }
    }
}

fn parse_get(link: &str, hostable_servers: &mut HashMap<&str, Box<dyn HostableServer>>) -> Message {
    match link {
        "/" => Message {
            variant: Variant::Ok,
            content: Content::File(String::from("hello.html")),
        },
        "/favicon.ico" => Message {
            variant: Variant::Ok,
            content: Content::RawBytes(Box::new(include_bytes!("../favicon.ico").to_owned())),
        },
        "/available-servers" => {
            let servers: Vec<&str> = hostable_servers.iter().map(|s| s.0.to_owned()).collect();
            Message {
                variant: Variant::Ok,
                content: Content::Struct(serde_json::to_string(&servers).unwrap_or_default()),
            }
        }
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
                        Err(e) => {
                            Message::new(Variant::InternalServerError, Content::Text(e.to_string()))
                        }
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
                    "update.js" => {
                        Message::new(
                            Variant::Ok,
                            Content::File(format!("{first_domain}/update.js"))
                        )
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
    let output = Command::new("shutdown").output();

    println!("Shutdown:\r\n{output:#?}");

    let output = match output {
        Ok(ok) => ok,
        Err(e) => {
            return Message::new(Variant::ServiceUnavailable, Content::Text(format!("{e:?}")));
        }
    };

    if output.status.success() {
        let _ = Command::new("curl")
            .args(["-d", "\"Shutting the Home Server down :(\"", "ntfy.sh/mood"])
            .output();
        Message::new(
            Variant::Ok,
            Content::Text("Shutting the server down".to_owned()),
        )
    } else {
        let _ = Command::new("curl")
            .args([
                "-d",
                &format!(
                    "Failed to shut the Home Server down: \r\n{:?}",
                    from_utf8(&output.stderr)
                ),
                "ntfy.sh/mood",
            ])
            .output();
        Message::new(
            Variant::ServiceUnavailable,
            Content::Text(format!("{:?}", from_utf8(&output.stderr))),
        )
    }
}


/// Fills the `HostableServerHashed` with tuples in the format (`path_home`, `&dyn HostableServer`)
///
/// # Examples
///
/// ```no_run
/// use hostable_server::{minecraft, arma}
///
/// let mut hostable_servers: HostableServerHashed = hostable_server_hashed!(
///     ("minecraft", minecraft::Server::new),
///     ("arma", arma::Server::new)
/// );
/// web_server::start("127.0.0.1", 80, &mut hostable_servers);
/// ```
#[macro_export]
macro_rules! hostable_server_hashed {
    ( $( ($name:literal, $obj:expr) ),* ) => {
        {
            let mut hostable_servers: HostableServerHashed = HashMap::new();
            $(
                let minecraft_server_info = $obj();

                hostable_servers.insert($name, Box::new(minecraft_server_info));
            )*
            hostable_servers
        }
    };
}
