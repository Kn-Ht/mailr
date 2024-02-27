//! Logging errors, warnings, etc.
//! All of these functions output **colored** output

use std::process;
use colored::Colorize;

/// Logs "error: {cause}: {root}" to stderr
pub fn error<S: AsRef<str>>(cause: S, root: anyhow::Error) -> ! {
    eprintln!("{}: {}: \"{root}\"", "error".bright_red().bold(), cause.as_ref().red());
    process::exit(1)
}

/// Logs "warning: {msg}" to stderr
pub fn warning<S: AsRef<str>>(msg: S) {
    eprintln!("{}: {}", "warning".bright_yellow().bold(), msg.as_ref());
}

/// Logs info: {msg} to stderr
pub fn info<S: AsRef<str>>(msg: S) {
    println!("{}: {}", "info".bright_green().bold(), msg.as_ref());
}