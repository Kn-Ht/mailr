#![cfg_attr(not(debug_assertions), allow(dead_code))]

// TODO: fix hint text after interactive mode end

use std::{env, io::{self, Write}, process, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use clap::Parser;
use colored::Colorize;
use config::ConfigManager;
use log::hint;

use crate::log::{error, info, warning};
mod config;
mod crypto;
mod log;
mod mail;

// SendEmail trait
use mail::SendMail;

#[derive(Parser, Debug)]
#[clap(override_usage(concat!(env!("CARGO_PKG_NAME"), " [--configure] --to <EMAIL> --subject <SUBJECT> --msg <MESSAGE BODY>")))]
pub struct Args {
    #[arg(
        short,
        long,
        action,
        help("set the global/local user email & password")
    )]
    configure: bool,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    pub to: String,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    pub subject: String,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    pub msg: String,
}

fn ask_send_email(cf: &ConfigManager) -> anyhow::Result<()> {
    let email = inquire::Text::new("recipient email:")
        .with_validator(ConfigManager::email_validator)
        .prompt()?;

    println!("");

    let subject = inquire::Text::new("subject:")
        .prompt()?;

    println!("");

    println!("{}", "message body (press CTRL-Z to stop typing):".bright_green());

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();

    // The msg body
    let mut body = String::with_capacity(512);

    // Handle control-c
    ctrlc::set_handler(move || {
        if done_clone.load(Ordering::SeqCst) {
            panic!("CONTROL-C INTERRUPT HIT");
        } else {
            done_clone.store(true, Ordering::SeqCst);
        }
        
    })?;

    print!("{} ", ">".green());
    let _ = stdout.flush();
    for line in stdin.lines() {
        let line = line?;
        print!("{} ", ">".green());
        let _ = stdout.flush();

        if done.load(Ordering::SeqCst) {
            break;
        }

        body.push_str(&line);
        body.push('\n');
    }
    println!("\n");

    let args = Args {
        configure: false,
        to: email,
        subject,
        msg: body
    };

    cf.send(&args)
}

fn main() {
    let argv = env::args().collect::<Vec<String>>();
    let argc = argv.len();

    let no_command_entered = argc <= 1;

    if no_command_entered {
        info(format!(
            "You have entered no commands. To see a list of commands run this program with {}.\n",
            "--help".blue()
        ));

        let config = ConfigManager::from_file();

        match config {
            Err(_) => {
                warning("Failed to read config for login information.");
                if let Err(_) | Ok(false) = inquire::prompt_confirmation(
                    "Do you want to set and save your login information? (y/n)",
                ) {
                    info("aborting...");
                    process::exit(0);
                }
                println!("\n");
            }
            Ok(config) => {
                info("Existing configuration found.");
                if let Err(_) | Ok(false) =
                    inquire::prompt_confirmation("Do you want to send an Email? (y/n)")
                {
                    info("aborting...");
                    process::exit(0);
                }

                if let Err(e) = ask_send_email(&config) {
                    error("failed to gather input for sending email", e);
                }
                return;
            }

        }

        // The user wants to configure their login data
        let config =
            ConfigManager::ask().unwrap_or_else(|err| error("failed to create config", err));

        if let Err(err) = config.save() {
            error("failed to save the config", err);
        }

        info("config saved successfully!\n");
        info("To send an Email, run this program again with the arguments specified in the help menu");
        hint(format!("Access the help menu by passing  {}  to the program on startup, or by simply restarting.", "--help".green()));

        return;
    }

    // The user has entered arguments
    let args = Args::parse_from(&argv);

    if args.configure {
        // The user wants to configure their login data
        let config =
            ConfigManager::ask().unwrap_or_else(|err| error("failed to create config", err));

        if let Err(err) = config.save() {
            error("failed to save the config", err);
        }
        return;
    }

    // Is the user login saved?
    let config = ConfigManager::from_file().unwrap_or_else(|err| {
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
