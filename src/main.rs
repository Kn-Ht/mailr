use clap::{CommandFactory, Parser};
mod config;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, action)]
    configure: bool,
    #[arg(short, long)]
    to: Option<String>,
    #[arg(short, long)]
    subject: Option<String>,
    #[arg(short, long)]
    msg: Option<String>,
}

fn main() {
    let args = Args::parse();
    let mut command = Args::command();

    if args.configure {
        config::configure();
        return;
    }

    // Has the user passed the data?
    config::expect_defined(&args.msg, &mut command, "No message body provided. Please provide it with --msg");
    config::expect_defined(&args.subject, &mut command, "No email subject provided. Please provide it with --subject");
    config::expect_defined(&args.to, &mut command, "No recipient provided. Please provide it with --to");

    // Is the user login saved?
    

    println!("{args:?}");
}
