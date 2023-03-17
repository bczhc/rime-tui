use rime_tui::cli::build_cli;
use rime_tui::rime::{Config, Engine};
use rime_tui::tui::TuiApp;
use rime_tui::xinput::XInput;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();
    let schema = matches.get_one::<String>("schema").unwrap();
    let user_dir = matches.get_one::<String>("user-dir").unwrap();
    let shared_dir = matches.get_one::<String>("shared-dir").unwrap();

    let mut engine = Engine::new(&Config {
        user_data_dir: user_dir.into(),
        shared_data_dir: shared_dir.into(),
    });
    engine.wait_for_session_created(Duration::from_secs(1));
    println!("Done!!");
    engine.close()?;
    return Ok(());

    let mut app = TuiApp::new()?;
    app.redraw()?;

    let input = XInput::new(None);
    loop {
        let Some(event) = input.next_event() else {
            continue
        };
        app.redraw()?;
    }

    app.stop()?;

    Ok(())
}
