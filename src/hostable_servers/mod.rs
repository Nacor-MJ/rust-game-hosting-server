use std::collections::HashMap;
use std::process::Command;
use std::fmt;

pub mod minecraft;

/// Describes a generic server that can be hosted
pub trait HostableServer {
    /// Starts the Server
    fn start(&mut self) -> Result<(), CommandFailure>;
    /// Stops the Server gracefully
    fn stop(&mut self) -> Result<(), CommandFailure>;
    /// Restart
    fn restart(&mut self) -> Result<(), CommandFailure> {
        Self::stop(self)?;
        Self::start(self)
    }
    /// Updates the Hostable Server Object
    /// The update to the client will be sent later
    fn update_status(&mut self) -> Result<(), CommandFailure>;
    /// Returns a representation of self as a Json object, the object shouldn't be nested
    fn to_json(&self) -> Result<String, serde_json::Error>;
}

pub type HostableServerHashed<'a> = HashMap<&'static str, &'a mut dyn HostableServer>;

/// Failure of a `HostableServer` command
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

/// Executes the `command` and parses the error into `CommandFailure`.
/// `command` is formated as it would be to the shell, arguments seperated by a space
fn exec_parse_command(command: &str) -> Result<(), CommandFailure> {
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
pub fn get_screen_sessions() -> String {
    let screen_server_list = Command::new("screen").arg("-list").output();

    match screen_server_list {
        Ok(screen_server_list) => {
            format!(
                "{:}",
                std::str::from_utf8(&screen_server_list.stdout)
                    .unwrap_or("Unrecognizible screen -list output")
            )
        }
        Err(e) => {
            format!("Error with the screen -list command: \r\n{:#?}", e)
        }
    }
}
