#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Ping,
    Unknown,
}

impl Command {
    pub async fn parse_commands(message: &str) -> Vec<Self> {
        message
            .split("\n")
            .filter(|command| *command != "")
            .map(|command| match command {
                "ping" => Command::Ping,
                _ => Command::Unknown,
            })
            .collect()
    }
}
