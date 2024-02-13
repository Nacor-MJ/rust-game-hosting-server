//! I don't know what to put here <3

use web_server::{
    self,
    hostable_servers::GeneralBashServer,
};

fn main() {
    let mut web_server = web_server::WebServer::new();

    web_server.add_hostable_server(Box::new(GeneralBashServer::new("minecraft")));
    web_server.add_hostable_server(Box::new(GeneralBashServer::new("arma")));

    web_server.start("192.168.11.69", 31415);
}
