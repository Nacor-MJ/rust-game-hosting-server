//! I don't know what to put here <3

use web_server::{
    self,
    hostable_servers::{arma, minecraft},
};

fn main() {
    let mut web_server = web_server::WebServer::new();

    web_server.add_hostable_server(Box::new(minecraft::Server::new()));
    web_server.add_hostable_server(Box::new(arma::Server::new()));

    web_server.start("192.168.11.69", 31415);
}
