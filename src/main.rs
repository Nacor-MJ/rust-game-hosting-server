//! =============================================================
//! Rust Game Hosting Server - main.rs
//! -------------------------------------------------------------
//! STATUS: Project is in limbo and may not work on newer Rust versions.
//! This file starts the web server and adds hostable game servers.
//! =============================================================

use web_server::{self, hostable_servers::GeneralBashServer};

fn main() {
    let mut web_server = web_server::WebServer::new();

    web_server.add_hostable_server(Box::new(GeneralBashServer::new("minecraft")));
    web_server.add_hostable_server(Box::new(GeneralBashServer::new("arma")));

    web_server.start("192.168.11.69", 31415);
}
