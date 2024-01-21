use web_server::WebServer;

fn main() {
    let config_txt = include_str!("../config.json");

    WebServer::new(config_txt);
}





