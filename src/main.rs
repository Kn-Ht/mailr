use clap::{CommandFactory, Parser};

use crate::config::Config;
mod config;

#[derive(Parser, Debug)]
#[clap(override_usage(concat!(env!("CARGO_PKG_NAME"), " [--configure] --to <EMAIL> --subject <SUBJECT> --msg <MESSAGE BODY>")))]
struct Args {
    #[arg(long, action, help("set the global/local user email & password"))]
    configure: bool,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    to: String,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    subject: String,
    #[arg(short, long, required_unless_present("configure"), default_value = "")]
    msg: String,
}

fn main() {
    let args = Args::parse();
    let mut command = Args::command();

    if args.configure {
        let config = Config::input();
        return;
    }

    // Is the user login saved?

    println!("{args:?}");
}
