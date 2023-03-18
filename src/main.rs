use anyhow::anyhow;
use rime_api::KeyEvent;
use rime_tui::cli::build_cli;
use rime_tui::key_event::KeyEventResolver;
use rime_tui::rime::{Config, DeployResult, Engine};
use rime_tui::tui::TuiApp;
use rime_tui::xinput::XInput;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // let matches = build_cli().get_matches();
    // let schema = matches.get_one::<String>("schema").unwrap();
    // let user_dir = matches.get_one::<String>("user-dir").unwrap();
    // let shared_dir = matches.get_one::<String>("shared-dir").unwrap();
    //
    // let mut engine = Engine::new(&Config {
    //     user_data_dir: user_dir.into(),
    //     shared_data_dir: shared_dir.into(),
    // });
    // let deploy_result = engine.wait_for_deploy_result(Duration::from_secs_f64(0.1));
    // match deploy_result {
    //     DeployResult::Init => {
    //         unreachable!();
    //     }
    //     DeployResult::Success => {
    //         println!("Deployment succeeded");
    //     }
    //     DeployResult::Failure => {
    //         println!("Deployment failed");
    //         return Ok(());
    //     }
    // }
    // engine.close()?;
    // return Ok(());

    // let mut app = TuiApp::new()?;
    // app.redraw()?;

    let mut key_resolver = KeyEventResolver::new(|repr| {
        println!("Repr: {}", repr);
    });

    let input = XInput::new(None);
    loop {
        let Some((_, event)) = input.next_event() else {
            continue
        };
        // app.redraw()?;
        key_resolver.on_key_event(&event);
    }

    // app.stop()?;

    Ok(())
}
