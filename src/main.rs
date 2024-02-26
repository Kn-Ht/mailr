#![cfg_attr(not(debug_assertions), allow(dead_code))]

use std::process;
use clap::{CommandFactory, Parser};

use crate::config::Config;
mod config;
mod crypto;
mod mail;

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
    let mut command = Args::command();

    if args.configure {
        let config = match Config::ask() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[ERROR :: failed to read user input]: {e}");
                process::exit(1);
            }
        };

        if let Err(err) = config.save() {
            eprintln!("[ERROR :: failed to save the config]: {err}");
            process::exit(1);
        }
        return;
    }

    // Is the user login saved?
    let config = match Config::from_file() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR :: can't read config]: {e}");
            process::exit(1);
        }
    };

    match config.send(&args) {
        Ok(_) => {
            println!("Successfully sent Mail!");
        }
        Err(e) => {
            eprintln!("[ERROR :: failed to send mail]: {e}");
            process::exit(1);
        }
    }
}
