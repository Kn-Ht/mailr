use crate::{crypto, info, warning};
use inquire::validator::Validation;
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::{Credentials, Mechanism},
    Address,
};
use serde::{Deserialize, Serialize};
use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Relay {
    None,
    Outlook,
    GMail,
    Custom,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelaySettings {
    pub addr: String,
    pub port: u16,
    pub tls: bool,
    pub authentication: Vec<Mechanism>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
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
                Self::None => "none",
                Self::Outlook => "outlook",
                Self::GMail => "Gmail",
                Self::Custom => "custom",
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
    pub fn settings(&self) -> RelaySettings {
        match self {
            Self::Outlook => RelaySettings {
                addr: "smtp.office365.com".to_string(),
                port: 587,
                tls: true,
                authentication: vec![Mechanism::Login],
            },
            Self::GMail => RelaySettings {
                addr: "smtp.gmail.com".to_string(),
                port: 587,
                tls: true,
                authentication: vec![Mechanism::Login],
            },
            _ => RelaySettings {
                addr: "your-address-here".to_string(),
                port: 1234,
                tls: true,
                authentication: vec![],
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    username: String,
    password: Vec<u8>,
    nonce: Vec<u8>,

    #[serde(skip)]
    password_str: Option<String>,
    #[serde(skip)]
    store_loc: Vec<SaveLocation>,
    #[serde(rename(serialize = "relay", deserialize = "relay"))]
    pub relay_settings: RelaySettings,
}

impl Config {
    const fn local_file_loc() -> &'static str {
        "./.mailr.toml"
    }
    fn global_file_loc() -> anyhow::Result<PathBuf> {
        if cfg!(target_os = "windows") {
            Ok(PathBuf::from(
                env::var_os("APPDATA").ok_or(anyhow::anyhow!("No %APPDATA% folder found."))?,
            )
            .join(".mailr.toml"))
        } else {
            Ok(PathBuf::new())
        }
    }

    pub fn credentials(&self) -> Credentials {
        Credentials::new(
            self.username.clone(),
            self.password_str.as_ref().unwrap().clone(),
        )
    }

    pub fn from_file() -> anyhow::Result<Self> {
        let local_path = Path::new(Self::local_file_loc());
        let contents: String;

        if local_path.is_file() {
            contents = fs::read_to_string(&local_path)?;
            let mut des: Self = toml::from_str(&contents)?;
            des.password_str = Some(crypto::decrypt(&des.password, des.nonce.as_slice().into())?);
            return Ok(des);
        }

        let global_path = Self::global_file_loc()?;

        if global_path.is_file() {
            contents = fs::read_to_string(&global_path)?;
            let mut des: Self = toml::from_str(&contents)?;
            des.password_str = Some(crypto::decrypt(&des.password, des.nonce.as_slice().into())?);
            return Ok(des);
        }

        Err(
            anyhow::anyhow!(
                "Failed to read configuration file, searched for '{}' (local) and '{}' (global), but they don't exist. Please configure the program first with {} --configure",
                local_path.display(), global_path.display(), env!("CARGO_PKG_NAME")
            )
        )
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let store_local = self.store_loc.contains(&SaveLocation::Local);
        let store_global = self.store_loc.contains(&SaveLocation::Global);

        if store_local {
            let path_local = Path::new(Self::local_file_loc());

            if path_local.is_file() {
                eprintln!(
                    "[WARNING] overwriting existing config '{}'",
                    path_local.display()
                );
            }

            fs::write(&path_local, toml::to_string_pretty(self)?)?;

            info!("saved file (local) to '{}'", path_local.display());
        }
        if store_global {
            let path_global = Self::global_file_loc()?;

            if path_global.is_file() {
                warning!(
                    "overwriting existing file '{}'",
                    path_global.display()
                );
            }

            fs::write(&path_global, toml::to_string_pretty(self)?)?;

            println!("[INFO] saved file (global) to '{}'", path_global.display());
        }
        Ok(())
    }
    /// Ask the user for config values
    pub fn ask() -> anyhow::Result<Self> {
        let validate_email = |email: &str| {
            Ok(if let Err(e) = email.parse::<Address>() {
                Validation::Invalid(e.into())
            } else {
                Validation::Valid
            })
        };

        let email = inquire::Text::new("email:")
            .with_validator(validate_email)
            .prompt()?;
        println!("");

        let password_plain = inquire::prompt_secret("password (will not be shown):")?;
        let mut password = Vec::with_capacity(password_plain.len());

        // encrypt password
        let nonce = crypto::encrypt(&password_plain, &mut password)?;

        let relay = inquire::Select::new(
            "which relay to use:",
            vec![Relay::Outlook, Relay::GMail, Relay::Custom],
        )
        .prompt()?;

        let relay_settings = if relay == Relay::Custom {
            // Read custom settings
            let addr = inquire::prompt_text("(custom) server address:")?;

            let validate_port = |port: &u16| {
                if (1..65535).contains(port) {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid("Valid port range is 1 to 65535".into()))
                }
            };

            let port = inquire::CustomType::<u16>::new("(custom) server port:")
                .with_validator(validate_port)
                .prompt()?;

            let tls = inquire::prompt_confirmation("(custom) use TLS? (y/n)")?;

            let authentication = inquire::MultiSelect::new(
                "authentication mechanisms:",
                vec![Mechanism::Plain, Mechanism::Login, Mechanism::Xoauth2],
            )
            .prompt()?;

            RelaySettings {
                addr,
                port,
                tls,
                authentication,
            }
        } else {
            relay.settings()
        };

        let store_loc = inquire::MultiSelect::new(
            "location to store email & password:",
            vec![SaveLocation::Local, SaveLocation::Global],
        )
        .prompt()?;

        Ok(Self {
            username: email,
            password,
            password_str: None,
            nonce: nonce.to_vec(),
            store_loc,
            relay_settings,
        })
    }
}
