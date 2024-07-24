use crate::parser::Command;

pub struct RedisApp {}

impl RedisApp {
    pub fn new() -> Self {
        RedisApp {}
    }

    pub fn execute_command(self, cmd: Command) -> Result<String, Box<dyn std::error::Error>> {
        match cmd {
            Command::Ping => Ok(Self::ping_command()),
            Command::Echo(arg) => Ok(Self::echo_command(arg)),
            _ => Ok(String::from("INVALID")),
        }
    }

    fn ping_command() -> String {
        String::from("PONG")
    }

    fn echo_command(arg: String) -> String {
        arg
    }
}
