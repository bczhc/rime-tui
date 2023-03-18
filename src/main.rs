use std::cell::RefCell;
use std::time::Duration;

use rime_tui::cli::build_cli;
use rime_tui::key_event::KeyEventResolver;
use rime_tui::rime::{Config, DeployResult, Engine};
use rime_tui::tui::{Candidate, TuiApp};
use rime_tui::xinput::XInput;

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();
    let schema = matches.get_one::<String>("schema").unwrap();
    let user_dir = matches.get_one::<String>("user-dir").unwrap();
    let shared_dir = matches.get_one::<String>("shared-dir").unwrap();
    let exit_command = matches.get_one::<String>("exit-command").unwrap();

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
            eprintln!("Deployment succeeded");
        }
        DeployResult::Failure => {
            eprintln!("Deployment failed");
            return Ok(());
        }
    }
    engine.create_session()?;
    engine.select_schema(schema)?;

    let mut app = TuiApp::new()?;
    app.start()?;

    app.redraw()?;

    let engine = RefCell::new(engine);
    let app = RefCell::new(app);
    let mut key_resolver = KeyEventResolver::new(|repr| {
        let mut engine = engine.borrow_mut();
        let mut app = app.borrow_mut();
        let ui_data = &mut app.ui_data;

        let status = engine.status().unwrap();

        if repr == "{BackSpace}" && !status.is_composing && ui_data.preedit.is_empty() {
            ui_data.output.pop();
            app.redraw().unwrap();
            return;
        }
        if repr == "{Return}" && !status.is_composing {
            ui_data.output.push('\n');
            app.redraw().unwrap();
            return;
        }
        if repr == "{space}" && !status.is_composing && ui_data.preedit.is_empty() {
            ui_data.output.push(' ');
            app.redraw().unwrap();
            return;
        }
        drop(status);

        if engine.simulate_key_sequence(repr).is_err() {
            eprintln!("Key simulation failed: {}", repr);
        }

        let context = engine.context();
        let menu = &context.as_ref().unwrap().menu;
        let preedit = context.as_ref().unwrap().composition.preedit.unwrap_or("");

        ui_data.preedit = String::from(preedit);
        ui_data.candidates = menu
            .candidates
            .iter()
            .enumerate()
            .map(|(i, x)| Candidate {
                text: x.text.into(),
                comment: x.comment.unwrap_or("").into(),
                highlighted: i == menu.highlighted_candidate_index as usize,
            })
            .collect();
        drop(context);
        let commit = engine.commit();
        // TODO: if taking the ownership of `commit` in the `match` below,
        //  `c.text` will be freed and thus its data is invalid
        let commit = match &commit {
            None => "",
            Some(c) => c.text,
        };
        ui_data.output.push_str(commit);

        app.redraw().unwrap();
    });

    let input = XInput::new(None);
    loop {
        let Some((_, event)) = input.next_event() else {
            continue
        };

        key_resolver.on_key_event(&event);

        if &app.borrow().ui_data.preedit == exit_command {
            break;
        }
    }

    engine.borrow_mut().close()?;
    app.borrow_mut().stop()?;

    Ok(())
}
