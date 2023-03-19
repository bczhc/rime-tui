use std::cell::RefCell;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::mem::MaybeUninit;
use std::os::fd::RawFd;
use std::sync::Mutex;
use std::thread::spawn;
use std::time::Duration;

use gag::Redirect;
use libc::{c_int, pipe};
use once_cell::sync::Lazy;

use rime_tui::cli::build_cli;
use rime_tui::fd_reader::FdReader;
use rime_tui::key_event::KeyEventResolver;
use rime_tui::rime::{Config, DeployResult, Engine};
use rime_tui::tui::{Candidate, TuiApp};
use rime_tui::xinput::XInput;

static STDERR_REDIRECT: Lazy<Mutex<Option<Redirect<RawFd>>>> = Lazy::new(|| Mutex::new(None));

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();
    let schema = matches.get_one::<String>("schema").unwrap();
    let user_dir = matches.get_one::<String>("user-dir").unwrap();
    let shared_dir = matches.get_one::<String>("shared-dir").unwrap();
    let exit_command = matches.get_one::<String>("exit-command").unwrap();

    setup_stderr_redirect()?;

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

    if let Some(r) = STDERR_REDIRECT.lock().unwrap().take() {
        drop(r);
    }

    Ok(())
}

fn setup_stderr_redirect() -> io::Result<()> {
    let fds = unsafe {
        let mut fds = MaybeUninit::<[c_int; 2]>::uninit();
        pipe(fds.assume_init_mut().as_mut_ptr());
        fds.assume_init()
    };

    let read_fd = fds[0];
    let write_fd = fds[1];

    spawn(move || {
        let redirect = Redirect::stderr(write_fd).unwrap();
        STDERR_REDIRECT.lock().unwrap().replace(redirect);
        let reader = FdReader::new(read_fd);
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            println!("{:?}", line);
        }
    });
    Ok(())
}
