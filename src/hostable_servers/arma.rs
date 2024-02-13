//! Implements [`crate::hostable_servers::HostableServer`] for arma3

use crate::{hostable_servers::get_screen_sessions, HostableServer};

use super::{exec_and_parse_command, CommandFailure, GeneralBashServer, Players, State};

/// Arma Server
#[derive(serde::Serialize)]
pub struct Server {
    /// State of the Server {On/Off/Unknown}
    state: State,

    /// State of the Server {On/Off/Unknown}
    players: Players,
}

impl Server {
    /// Returns a new Instance of `Server`
    #[must_use]
    pub const fn idk() -> GeneralBashServer {
        GeneralBashServer::new("arma")
    }
}

impl HostableServer for Server {
    fn start(&mut self) -> Result<(), super::CommandFailure> {
        let state = exec_and_parse_command("sh ./arma/start.sh");

        if state.is_ok() {
            self.state = State::On;
        };

        state
    }

    fn stop(&mut self) -> Result<(), super::CommandFailure> {
        let state = exec_and_parse_command("sh ./arma/stop.sh");

        if state.is_ok() {
            self.state = State::Off;
        };

        state
    }

    fn update_status(&mut self) -> Result<(), CommandFailure> {
        let sessions = get_screen_sessions();

        if sessions.contains(".arma_server\t") {
            self.state = State::On;
        } else {
            self.state = State::Off;
        }

        Ok(())
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    fn get_path(&self) -> &'static str {
        "arma"
    }
}
