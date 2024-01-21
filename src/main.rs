use web_server::WebServer;

fn main() {
    let config_txt = include_str!("../config.json");

    WebServer::start(config_txt);
}