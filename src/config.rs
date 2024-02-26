use std::fmt;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Relay {
    Outlook,
    GMail,
    Custom
}

pub struct RelaySettings {
    tls: bool,
}

#[derive(Clone, Copy)]
pub enum SaveLocation {
    Global,
    Local,
}

impl fmt::Display for Relay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Outlook => "outlook",
                Self::GMail => "Gmail",
                Self::Custom => "custom"
            }
        )
    }
}

impl fmt::Display for SaveLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Global => "global",
                Self::Local => "local (will override global)",
            }
        )
    }
}

impl Relay {
    // TODO: tls(), etc.
}

pub struct Config {
    username: String,
    password: String,
    store_loc: SaveLocation,
    relay: Relay,
}

impl Config {
    /// Try to find config file
    pub fn read() -> anyhow::Result<Self> {
        Ok(Self {
            username: "".to_string(),
            password: "".to_string(),
            relay: Relay::Outlook,
            store_loc: SaveLocation::Local
        })
    }
    pub fn input() -> anyhow::Result<Self> {
        let email = inquire::Text::new("email:").prompt()?;
        println!("");
        let password = inquire::prompt_secret("password (will not be shown):")?;

        let relay = inquire::Select::new(
            "which relay to use:",
            vec![Relay::Outlook, Relay::GMail, Relay::Custom]
        ).prompt()?;

        let relay_settings: RelaySettings;

        if relay == Relay::Custom {
            // Read custom settings
            todo!()
        }
    
        let save_loc = inquire::MultiSelect::new(
            "location to store email & password:",
            vec![SaveLocation::Local, SaveLocation::Global],
        )
        .prompt()?;


    
        Ok(())
    }
}

pub fn expect_defined<T>(arg: &Option<T>, command: &mut clap::Command, msg: &str) -> bool {
    if arg.is_none() {
        let _ = command
            .error(clap::error::ErrorKind::MissingRequiredArgument, msg)
            .print();
        return true;
    }
    false
}