//!
//! Abstracts away the implementation details of a web server.
//! Servers that implement the [`HostableServer`] trait can be run on it

//! =============================================================
//! Rust Game Hosting Server - lib.rs
//!
//! STATUS: Project is in limbo and may not work on newer Rust versions.
//! Abstracts away the implementation details of a web server.
//! Servers that implement the [`HostableServer`] trait can be run on it
//! =============================================================

use hostable_servers::HostableServer;
use http::{Content, Message, Variant};
use std::{
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

/// Simple Web interface for the [`HostableServer`] trait
pub struct WebServer {
    /// hostable_servers
    hostable_servers: Vec<Box<dyn HostableServer>>,
}

impl WebServer {
    /// Returns a new instance of `WebServer`
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hostable_servers: Vec::new(),
        }
    }

    /// Adds a `HostableServer` to the `WebServer` Interface
    ///
    /// # Example
    /// Creates and Runs a Server with the `arma` and `minecraft` Interfaces:
    /// ```no_run
    /// use web_server::{ self, hostable_servers::GeneralBashServer};
    ///
    /// let mut web_server = web_server::WebServer::new();
    ///
    /// web_server.add_hostable_server(Box::new(GeneralBashServer::new("minecraft")));
    /// web_server.add_hostable_server(Box::new(GeneralBashServer::new("arma")));
    ///
    /// web_server.start("192.168.11.69", 31415);
    /// ```
    pub fn add_hostable_server(&mut self, server: Box<dyn HostableServer>) {
        self.hostable_servers.push(server);
    }

    /// Returns a tcp listener with the set IP Adress and port
    ///
    /// # Pancis
    /// Panics if the `port` is used or blocker
    fn get_tcp_listener(ip: &'static str, port: usize) -> TcpListener {
        // Ip adress setup
        let ip_and_port = format!("{ip}:{port}");

        println!("Http://{ip_and_port}/");

        TcpListener::bind(ip_and_port).expect("Problems with the IP and port")
    }

    /// Starts the web server and starts listening for connections
    ///
    /// `hostable_servers` is used to provide all available server.
    /// Use the [`hostable_server_hashed`] macro to create the hasmap
    /// # Panics
    /// Panics if the `port` is used or blocker
    /// # Example
    /// Creates and Runs a Server with the `arma` and `minecraft` Interfaces:
    /// ```no_run
    /// use web_server::{ self, hostable_servers::GeneralBashServer};
    ///
    /// let mut web_server = web_server::WebServer::new();
    ///
    /// web_server.add_hostable_server(Box::new(GeneralBashServer::new("minecraft")));
    /// web_server.add_hostable_server(Box::new(GeneralBashServer::new("arma")));
    ///
    /// web_server.start("192.168.11.69", 31415);
    /// ```
    pub fn start(&mut self, ip: &'static str, port: usize) -> ! {
        // timeout setup
        let (tx, timer_thread) = Self::create_timeout_thread(Duration::from_secs(60 * 30));

        // TCP setup

        let listener = Self::get_tcp_listener(ip, port);

        // accepting connections
        for stream in listener.incoming() {
            // listener.incoming() never returns None, read the docs for more info
            let stream = stream.expect("Never happens");
            self.handle_connection(stream, &tx).unwrap_or_else(|e| {
                println!("Connection Failed: {e}");
            });
        }

        // IDK if I need this, but I always wanted to use drop somewhere
        // and when I put this here I know timer_thread will live long enough
        drop(timer_thread);

        loop {
            // Seriously though the listener.incoming() method never returns None
            println!("NEVER SHOULD HAVE COME HERE");
        }
    }

    /// Handles the TCP connection
    ///
    /// Prints updates to stdout or stderr during the whole operation
    fn handle_connection(&mut self, mut stream: TcpStream, tx: &Sender<()>) -> std::io::Result<()> {
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

        let htttp_response: Message = self.parse_http_request(method, link);

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

    /// Parses the http to the best of it's abilities
    ///
    /// # Errors
    /// Can only process GET and POST methods
    fn parse_http_request(&mut self, method: &str, link: &str) -> Message {
        match method {
            "GET" => self.parse_get(link),
            "POST" => self.parse_post(link),
            e => {
                println!("Method not available: {e}");
                Message::new(
                    Variant::NotFound,
                    Content::Text(format!("Unkown method: {e}")),
                )
            }
        }
    }

    /// Parses a post method
    fn parse_post(&mut self, link: &str) -> Message {
        match link {
            "/Shutdown" => Self::shutdown(),
            "/Ping" => Message::new(Variant::Ok, Content::Text("Ping succesfull".to_owned())),
            link => {
                let mut link_split = link.split('/');
                let _ = link_split.next(); // the link starts with '/'

                let first_domain = link_split.next().unwrap_or("Unavailabe");

                self.hostable_servers
                    // Checks if any of the paths match the domain
                    .iter_mut()
                    .find(|s| s.get_path() == first_domain)
                    .map_or_else(
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
                    )
            }
        }
    }

    /// Parses a GET request
    ///
    /// # Errors
    /// Returns a 404 if it can't find the file
    fn parse_get(&mut self, link: &str) -> Message {
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
                let servers: Vec<&str> =
                    self.hostable_servers.iter().map(|s| s.get_path()).collect();
                Message {
                    variant: Variant::Ok,
                    content: Content::Struct(serde_json::to_string(&servers).unwrap_or_default()),
                }
            }
            link => {
                let mut link_split = link.split('/');
                let _ = link_split.next(); // the link starts with '/'

                let first_domain = link_split.next().unwrap_or("Unavailable");

                if first_domain == "file" {
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
                } else if let Some(hostable_server) = self
                    .hostable_servers
                    .iter_mut()
                    .find(|s| s.get_path() == first_domain)
                {
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
                        "update.js" => Message::new(
                            Variant::Ok,
                            Content::File(format!("{first_domain}/update.js")),
                        ),
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
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    /// Attempts to command the whole PC to shutdown.
    ///
    /// The PC shoudl Shutdown in one minute
    fn shutdown() -> Message {
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
    #[cfg(not(target_os = "linux"))]
    /// Shutdown is only implemented for linux
    fn shutdown() -> Message {
        Message::new(
            Variant::ServiceUnavailable,
            Content::Text("Shutting downt isn't supported in this platform yet".to_string()),
        )
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
                        Self::shutdown();
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
}
