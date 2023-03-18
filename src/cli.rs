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
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("shared-dir")
                .long("shared-dir")
                .default_value(shared_data_dir)
                .value_hint(ValueHint::DirPath)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("user-dir")
                .long("user-dir")
                .default_value(user_data_dir)
                .value_hint(ValueHint::DirPath)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("exit-command")
                .long("exit-command")
                .default_value("/exit")
                .action(ArgAction::Set),
        )
}
