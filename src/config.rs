#[derive(Clone, Copy)]
pub enum Relay {
    Outlook,
    GMail
}

impl Relay {
    // TODO: tls(), etc.
}

pub struct Config {
    username: String,
    password: String, 
    relay: Relay
}

impl Config {
    /// Try to find config file
    pub fn read() -> anyhow::Result<Self> {
        Ok(
            Self {
                username: "".to_string(),
                password: "".to_string(),
                relay: Relay::Outlook
            }
        )
    }
}

pub fn expect_defined<T>(arg: &Option<T>, command: &mut clap::Command, msg: &str) {
    if arg.is_none() {
        command
            .error(
                clap::error::ErrorKind::MissingRequiredArgument,
                msg,
            )
            .exit();
        }
}

pub fn configure() {

}