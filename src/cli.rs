use clap::{Arg, ArgAction, Command};

fn build_cli() -> Command {
    Command::new("rime-tui").arg(
        Arg::new("schema")
            .required(true)
            .short('s')
            .long("schema")
            .action(ArgAction::Set),
    )
}
