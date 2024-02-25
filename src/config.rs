#[derive(Clone, Copy)]
pub enum Relay {
    Outlook,
    GMail
}

pub struct Config {
    username: String,
    password: String, 
    relay: Relay
}

impl Config {
    pub fn read() -> Result<Self> {

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