//! =============================================================
//! Rust Game Hosting Server - hostable_servers/mod.rs
//!
//! STATUS: Project is in limbo and may not work on newer Rust versions.
//! Creates an interface for servers that are supposed to be hosted <3
//! =============================================================

use std::fmt;
use std::process::Command;

/// Represents a server that can be hosted
///
/// # Errors
/// Doesn't work if there isn't an update.js file in `path_home`.
/// This file should have update_{`path_home`} function which is called periodically
/// to update the state in the website
pub trait HostableServer {
    /// Returns the Path to where the server is stored at
    ///
    /// The folder needs to contain an update.js file that has a update_{path} function
    /// to update the client side
    fn get_path(&self) -> &'static str;
    /// Starts the Server
    /// # Errors
    /// Errors if the start.sh doesn't work.
    /// Could be from not having enough priviliges.
    fn start(&mut self) -> Result<(), CommandFailure>;
    /// Stops the Server gracefully
    ///
    /// The function should not return until the server is stopped due to
    /// the default [`HostableServer::restart`] implementation
    /// # Errors
    /// Errors if the stop.sh doesn't work.
    /// Could be from not having enough priviliges.
    fn stop(&mut self) -> Result<(), CommandFailure>;
    /// Restart
    /// # Errors
    /// Errors if the stop.sh or start.sh doesn't work.
    /// Could be from not having enough priviliges.
    fn restart(&mut self) -> Result<(), CommandFailure> {
        Self::stop(self)?;
        Self::start(self)
    }
    /// Updates the Hostable Server Object
    /// The update to the client will be sent later
    /// # Errors
    /// Errors if the status.sh doesn't work.
    /// Could be from not having enough priviliges.
    fn update_status(&mut self) -> Result<(), CommandFailure>;
    /// Returns a representation of self as a Json object, the object shouldn't be nested
    /// # Errors
    /// Serialization can fail if Self's implementation of Serialize decides to fail,
    /// or if Self contains a map with non-string keys.
    fn to_json(&self) -> Result<String, serde_json::Error>;
}

/// Failure of a [`HostableServer`] command
pub struct CommandFailure(String);
// Implement std::fmt::Display for CommandFailure
impl fmt::Display for CommandFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An Error With Requested Command Occurred: {}", self.0) // user-facing output
    }
}
// Implement std::fmt::Debug for CommandFailure
impl fmt::Debug for CommandFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}\n{}", file!(), line!(), self.0) // programmer-facing output
    }
}

// Generic Helper Functions <3

/// Executes the `command` and possibly parses the error into [`CommandFailure`].
///
/// `command` is formated as it would be when passed into the shell, arguments seperated by a space
fn exec_and_parse_command(command: &str) -> Result<(), CommandFailure> {
    let mut command_split = command.split(' ');
    let program: &str = command_split.next().expect("Invalid Command");
    let arguments: Vec<&str> = command_split.collect();

    let output = Command::new(program).args(arguments).output();

    match output {
        Err(e) => Err(CommandFailure(e.to_string())),
        Ok(_) => Ok(()),
    }
}

/// Returns the screen sessions
#[must_use]
pub fn get_screen_sessions() -> String {
    let screen_server_list = Command::new("screen").arg("-list").output();

    match screen_server_list {
        Ok(screen_server_list) => std::str::from_utf8(&screen_server_list.stdout)
            .unwrap_or("Unrecognizible screen -list output")
            .to_string(),
        Err(e) => {
            format!("Error with the screen -list command: \r\n{e:#?}")
        }
    }
}

/// Status of a minecraft server
#[derive(serde::Serialize)]
enum State {
    /// Turned On
    On,
    /// Turned Off
    Off,
    /// Unknown
    Unknown,
}
impl State {
    /// Returns a new instance of `State`
    pub const fn new() -> Self {
        Self::Off
    }
}

/// Number of players logged into the Server and their nametags
#[derive(serde::Serialize)]
struct Players {
    /// Num of players
    count: usize,
    /// Players nametags
    name_tags: Vec<String>,
}
impl Players {
    /// Returns a new instance of `Players`
    pub const fn new() -> Self {
        Self {
            count: 0,
            name_tags: Vec::new(),
        }
    }
}

/// Basic implementation for the [`HostableServer`] Trait.
///
/// The Server is contolled by start.sh and stop.sh scripts that are located in path.
///
/// For now the bash scripts are responsible for creating a screen session with the name {path}_server
#[derive(serde::Serialize)]
pub struct GeneralBashServer {
    /// Path to the home directory of the Server
    path: &'static str,
    /// State of the Server {On, Off, Unknown}
    state: State,
    /// Number of Players and their name tags
    players: Players,
}

impl GeneralBashServer {
    /// Returns a new Instance of `Server`
    #[must_use]
    pub const fn new(path: &'static str) -> Self {
        Self {
            path,
            state: State::new(),
            players: Players::new(),
        }
    }
}

impl HostableServer for GeneralBashServer {
    fn start(&mut self) -> Result<(), CommandFailure> {
        let state = exec_and_parse_command(&format!("sh ./{}/start.sh", self.path));

        if state.is_ok() {
            self.state = State::Unknown;
        };

        state
    }

    fn stop(&mut self) -> Result<(), CommandFailure> {
        exec_and_parse_command(&format!("sh ./{}/stop.sh", self.path))?;

        self.update_status()
    }

    fn update_status(&mut self) -> Result<(), CommandFailure> {
        let sessions = get_screen_sessions();

        if sessions.contains(&format!(".{}_server\t", self.path)) {
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
        self.path
    }
}
