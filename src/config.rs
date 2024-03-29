use crate::{crypto::Cipher, info, warning};
use anyhow::Ok;
use inquire::{list_option::ListOption, validator::Validation};
use lettre::{
    transport::smtp::authentication::{Credentials, Mechanism},
    Address,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    process,
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
pub struct Login {
    username: String,
    password: Vec<u8>,
    nonce: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    login: Login,
    #[serde(rename(serialize = "relay", deserialize = "relay"))]
    pub relay_settings: RelaySettings,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigManager {
    #[serde(flatten)]
    pub config: Config,
    #[serde(skip)]
    password_str: Option<String>,
    #[serde(skip)]
    store_loc: Vec<SaveLocation>,
}

impl ConfigManager {
    pub fn email_validator(
        email: &str,
    ) -> Result<Validation, Box<dyn Error + Send + Sync + 'static>> {
        type Res = Result<Validation, Box<dyn Error + Send + Sync + 'static>>;
        Res::Ok(if let Err(e) = email.parse::<Address>() {
            Validation::Invalid(e.into())
        } else {
            Validation::Valid
        })
    }

    const fn local_file_loc() -> &'static str {
        "./.mailr.toml"
    }
    fn global_file_loc() -> anyhow::Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            Ok(PathBuf::from(
                env::var_os("APPDATA").ok_or(anyhow::anyhow!("No %APPDATA% folder found."))?,
            )
            .join(".mailr.toml"))
        }

        #[cfg(target_os = "macos")]
        {
            fs::create_dir_all("/Library/Application Support/mailr")?;
            return Ok(PathBuf::from(
                "/Library/Application Support/mailr/.mailr.toml",
            ));
        }

        // NOTE: if compiling fails here, you have to implement a function that returns the global config file path for your OS.
    }

    /// Clone username & password into `Credentials`
    pub fn credentials(&self) -> Credentials {
        Credentials::new(
            self.config.login.username.clone(),
            self.password_str.as_ref().unwrap().clone(),
        )
    }

    /// Try to read the config from a file.  
    /// Strategy: First check local, then check global.
    pub fn from_file() -> anyhow::Result<Self> {
        let local_path = Path::new(Self::local_file_loc());
        let contents: String;

        let cipher = Cipher::new();

        if local_path.is_file() {
            contents = fs::read_to_string(&local_path)?;
            let mut des: Self = toml::from_str(&contents).map_err(|e| {
                anyhow::anyhow!(
                    "failed to read local config '{}': {e}",
                    local_path.display()
                )
            })?;
            des.password_str = Some(cipher.decrypt(
                &des.config.login.password,
                des.config.login.nonce.as_slice().into(),
            )?);
            return Ok(des);
        }

        let global_path = Self::global_file_loc()?;

        if global_path.is_file() {
            contents = fs::read_to_string(&global_path)?;
            let mut des: Self = toml::from_str(&contents).map_err(|e| {
                anyhow::anyhow!(
                    "failed to read global config '{}': {e}",
                    global_path.display()
                )
            })?;
            des.password_str = Some(cipher.decrypt(
                &des.config.login.password,
                des.config.login.nonce.as_slice().into(),
            )?);
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
        &self.config.login.username
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let store_local = self.store_loc.contains(&SaveLocation::Local);
        let store_global = self.store_loc.contains(&SaveLocation::Global);

        if store_local {
            let path_local = Path::new(Self::local_file_loc());

            if path_local.is_file() {
                warning(format!(
                    "local config already exists at '{}'",
                    path_local.display()
                ));
                if let Err(_) | Result::Ok(false) =
                    inquire::prompt_confirmation("overwrite existing config? (y/n)")
                {
                    info("aborting...");
                    process::exit(0);
                }
            }

            fs::write(&path_local, toml::to_string_pretty(self)?)?;

            info(format!("saved file (local) to '{}'", path_local.display()));
        }
        if store_global {
            let path_global = Self::global_file_loc()?;

            if path_global.is_file() {
                warning(format!(
                    "global config already exists at '{}'",
                    path_global.display()
                ));
                if let Err(_) | Result::Ok(false) =
                    inquire::prompt_confirmation("overwrite existing config? (y/n)")
                {
                    info("aborting...");
                    process::exit(0);
                }
            }

            fs::write(
                &path_global,
                toml::to_string_pretty(self).map_err(|e| {
                    anyhow::anyhow!(
                        "failed to read global config '{}': {e}",
                        path_global.display()
                    )
                })?,
            )?;

            info(format!(
                "saved config (global) to '{}'",
                path_global.display()
            ));
        }
        Ok(())
    }
    /// Ask the user for config values
    pub fn ask() -> anyhow::Result<Self> {
        let cipher = Cipher::new();

        // Ask user for email:
        let email = inquire::Text::new("email:")
            .with_validator(Self::email_validator)
            .prompt()?;
        println!("");

        // Ask user for their password.
        let password_plain = inquire::prompt_secret("password (will not be shown):")?;
        let mut password = Vec::with_capacity(password_plain.len());

        // encrypt password
        let nonce = cipher.encrypt(&password_plain, &mut password)?;

        // Drop the password_plain early
        drop(password_plain);

        let relay = inquire::Select::new(
            "which relay to use:",
            vec![Relay::Outlook, Relay::GMail, Relay::Custom],
        )
        .prompt()?;

        let relay_settings = if relay == Relay::Custom {
            // Read custom settings
            let addr = inquire::prompt_text("(custom) server address:")?;

            let validate_port = |port: &u16| {
                type Res = Result<Validation, Box<dyn Error + Send + Sync + 'static>>;
                Res::Ok(if (1..65535).contains(port) {
                    Validation::Valid
                } else {
                    Validation::Invalid("Valid port range is 1 to 65535".into())
                })
            };

            let port = inquire::CustomType::<u16>::new("(custom) server port:")
                .with_validator(validate_port)
                .prompt()?;

            let tls = inquire::prompt_confirmation("(custom) use TLS? (y/n)")?;

            let mut authentication = inquire::MultiSelect::new(
                "authentication mechanisms:",
                vec![Mechanism::Plain, Mechanism::Login, Mechanism::Xoauth2],
            )
            .prompt()?;

            if authentication.is_empty() {
                authentication.push(Mechanism::Plain);
            }

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
        .with_validator(|locs: &[ListOption<&SaveLocation>]| {
            type ValidResult = Result<Validation, Box<dyn Error + Send + Sync>>;
            ValidResult::Ok(if !locs.is_empty() {
                Validation::Valid
            } else {
                Validation::Invalid("Please select at least one save location.".into())
            })
        })
        .prompt()?;

        Ok(Self {
            config: Config {
                login: Login {
                    username: email,
                    password,
                    nonce: nonce.to_vec(),
                },
                relay_settings,
            },
            password_str: None,
            store_loc,
        })
    }
}
