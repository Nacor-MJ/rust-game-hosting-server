//! Implements [`crate::hostable_servers::HostableServer`] for minecraft

use crate::hostable_servers::{
    exec_and_parse_command, get_screen_sessions, CommandFailure, HostableServer,
};
use serde::Serialize;

use super::{Players, State};

/// Minecraft Server with the State and number of Players
#[derive(Serialize)]
pub struct Server {
    /// State of the Server {On/Off/Unknown}
    state: State,
    /// Number of players and their nametags
    players: Players,
}
impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    /// Creates a new turned off minecraft server
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: State::new(),
            players: Players::new(),
        }
    }
    /// Sets `self` to default
    fn set_default(&mut self) {
        self.state = State::Off;
        self.players = Players {
            count: 0,
            name_tags: Vec::new(),
        };
    }
    /// Updates self
    /// 
    /// # Errors
    /// Returns a [`CommandFailure`] if the program doesn't have the right privilages 
    fn update_players(&mut self) -> Result<(), CommandFailure> {
        exec_and_parse_command("sh ./minecraft/status.sh")?;

        let output = std::fs::read_to_string("Minecraft/screenlog.0").unwrap_or_else(|e| {
            eprintln!("\x1b[31mCouldn't read the Minecraft log file: {e}\x1b[39m");
            String::new()
        });

        let last_line = output.lines().last().unwrap_or("Couldn't parse the log");

        let index_of_players = last_line.find(" of a max of ");

        if index_of_players.is_none() {
            self.state = State::Unknown;
            return Ok(());
        }
        #[allow(clippy::unwrap_used)] // This is fine because of the if statement above
        let index_of_players = index_of_players.unwrap();

        let slice = &last_line[index_of_players - 2..index_of_players];
        println!("{slice}");
        match slice.to_string().trim().parse::<usize>() {
            Ok(ok) => self.players.count = ok,
            Err(e) => println!("Error with parsing the number of players: \r\n{e}"),
        }

        // todo!("Make it so that I can play on different minecraft versions<3");

        self.state = State::On;

        Ok(())
    }
}

impl HostableServer for Server {
    fn get_path(&self) -> &'static str {
        "minecraft"
    }
    fn start(&mut self) -> Result<(), CommandFailure> {
        let state = exec_and_parse_command("sh ./minecraft/start.sh");

        if state.is_ok() {
            self.state = State::Unknown;
        };

        state
    }

    fn stop(&mut self) -> Result<(), CommandFailure> {
        let state = exec_and_parse_command("sh ./minecraft/stop.sh");

        if state.is_ok() {
            self.state = State::Unknown;
        };

        state
    }

    /// To be honest this is shit, please update it :D
    /// check if `last_line` actually contains the right info
    /// parse the `last_line`
    fn update_status(&mut self) -> Result<(), CommandFailure> {
        let sessions = get_screen_sessions();

        if sessions.contains(".minecraft_server\t") {
            self.update_players()?;
        } else {
            self.set_default();
        }

        Ok(())
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}




