use crate::rime::{get_shared_data_dir, get_user_data_dir};
use clap::{Arg, ArgAction, Command, ValueHint};

pub fn build_cli() -> Command {
    let user_data_dir = get_user_data_dir().to_string_lossy().to_string();
    let user_data_dir = Box::leak(user_data_dir.into_boxed_str()) as &'static str;
    let shared_data_dir = get_shared_data_dir().to_string_lossy().to_string();
    let shared_data_dir = Box::leak(shared_data_dir.into_boxed_str()) as &'static str;

    Command::new("rime-tui")
        .arg(
            Arg::new("schema")
                .required(true)
                .short('s')
                .long("schema")
                .action(ArgAction::Set)
                .required(false),
        )
        .arg(
            Arg::new("shared-dir")
                .long("shared-dir")
                .default_value(shared_data_dir)
                .value_hint(ValueHint::DirPath)
                .action(ArgAction::Set)
                .help("Rime shared data directory"),
        )
        .arg(
            Arg::new("user-dir")
                .long("user-dir")
                .default_value(user_data_dir)
                .value_hint(ValueHint::DirPath)
                .action(ArgAction::Set)
                .help("Rime user data directory"),
        )
        .arg(
            Arg::new("exit-command")
                .long("exit-command")
                .default_value("/exit")
                .action(ArgAction::Set)
                .help("Input command for exiting the program"),
        )
        .arg(
            Arg::new("copy-command")
                .long("copy-command")
                .default_value("/copy")
                .action(ArgAction::Set)
                .help("Input command for putting the output into X11 clipboard"),
        )
        .arg(
            Arg::new("load-command")
                .long("load-command")
                .action(ArgAction::Set)
                .default_value("/load")
                .help("Input command for loading from X11 clipboard"),
        )
}
