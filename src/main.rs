use std::collections::HashMap;

use web_server::{
    self,
    hostable_servers::{minecraft, HostableServerHashed},
};

fn main() {
    let mut hostable_servers: HostableServerHashed =
        web_server::hostable_server_hashed!(("minecraft", minecraft::Server::new));

    web_server::start("192.168.11.69", 31415, &mut hostable_servers);
}
