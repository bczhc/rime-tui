use anyhow::anyhow;
use rime_api::KeyEvent;
use rime_tui::cli::build_cli;
use rime_tui::key_event::KeyEventResolver;
use rime_tui::rime::{Config, DeployResult, Engine};
use rime_tui::tui::TuiApp;
use rime_tui::xinput::XInput;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
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
    let deploy_result = engine.wait_for_deploy_result(Duration::from_secs_f64(0.1));
    match deploy_result {
        DeployResult::Init => {
            unreachable!();
        }
        DeployResult::Success => {
            println!("Deployment succeeded");
        }
        DeployResult::Failure => {
            println!("Deployment failed");
            return Ok(());
        }
    }
    engine.create_session()?;
    engine.select_schema(schema)?;

    let mut app = TuiApp::new()?;
    // app.redraw()?;

    let engine = RefCell::new(engine);
    let mut key_resolver = KeyEventResolver::new(|repr| {
        let mut engine = engine.borrow_mut();

        println!("Repr: {}", repr);
        if engine.simulate_key_sequence(repr).is_err() {
            eprintln!("Key simulation failed: {}", repr);
        }

        let context = engine.context();
        let menu = &context.as_ref().unwrap().menu;
        for c in &menu.candidates {
            println!("{:?}", c);
        }
        drop(context);
        let commit = engine.commit();
        let commit = commit.as_ref();
        if let Some(commit) = commit {
            println!("Commit: {}", commit.text);
        }
    });

    let input = XInput::new(None);
    loop {
        let Some((_, event)) = input.next_event() else {
            continue
        };
        key_resolver.on_key_event(&event);
    }

    engine.borrow().close()?;
    // app.stop()?;

    Ok(())
}
