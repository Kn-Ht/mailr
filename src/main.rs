#![cfg_attr(not(debug_assertions), allow(dead_code))]

use clap::{CommandFactory, Parser};

use crate::config::Config;
use crate::log::{error, info, warning};
mod config;
mod crypto;
mod mail;
mod log;


// SendEmail trait
use mail::SendMail;

#[derive(Parser, Debug)]
#[clap(override_usage(concat!(env!("CARGO_PKG_NAME"), " [--configure] --to <EMAIL> --subject <SUBJECT> --msg <MESSAGE BODY>")))]
pub struct Args {
    #[arg(long, action, help("set the global/local user email & password"))]
    configure: bool,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    pub to: String,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    pub subject: String,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    pub msg: String,
}

fn main() {
    let args = Args::parse();
    let mut _command = Args::command();

    if args.configure {
        // The user wants to configure their login data
        let config = match Config::ask() {
            Ok(c) => c,
            Err(e) => {
                error("failed to create config", e);
            }
        };

        if let Err(err) = config.save() {
            error("failed to save the config", err);
        }
        return;
    }

    // Is the user login saved?
    let config = Config::from_file().unwrap_or_else(|err| {
        error("can't read config", err);
    });

    // Was the email sent successfully?
    match config.send(&args) {
        Ok(_) => {
            info("Successfully sent Mail!");
        }
        Err(e) => {
            error("failed to send mail", e);
        }
    }
}
