//! Logging errors, warnings, etc.

#[macro_export]
macro_rules! error {
    ($fmt:literal) => {
        eprintln!("{}: {}: \"{}\"", "error".bright_red().bold(), $cause.red(), $root);
        std::process::exit(1);
    };
}

#[macro_export]
macro_rules! warning {
    ($msg:literal) => {
        eprintln!("{}: {}", "warning".bright_yellow().bold(), $msg);
    };
}

#[macro_export]
macro_rules! info {
    ($msg:literal) => {
        println!(concat!("{}: ", $msg), "info".bright_green().bold());
    };
    ($msg:literal, $($arg:expr),*) => {
        
    };
}