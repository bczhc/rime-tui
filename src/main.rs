use std::cell::RefCell;
use std::io;
use std::io::{BufRead, BufReader};
use std::mem::MaybeUninit;
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;

use gag::Redirect;
use libc::{c_int, pipe};
use once_cell::sync::Lazy;
use rime_api::engine::{DeployResult, Engine};
use rime_api::{KeyStatus, Traits};
use x11::keysym::*;

use rime_tui::cli::build_cli;
use rime_tui::fd_reader::FdReader;
use rime_tui::key_event::KeyEventResolver;
use rime_tui::tui::{Candidate, TuiApp};
use rime_tui::xinput::XInput;
use rime_tui::{
    load_clipboard, put_clipboard, WithLockExt, APP_NAME, DISTRIBUTION_CODE_NAME,
    DISTRIBUTION_NAME, DISTRIBUTION_VERSION,
};

static STDERR_REDIRECT: Lazy<Mutex<Option<Redirect<RawFd>>>> = Lazy::new(|| Mutex::new(None));

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();
    let schema = matches.get_one::<String>("schema");
    let user_dir = matches.get_one::<String>("user-dir").unwrap();
    let shared_dir = matches.get_one::<String>("shared-dir").unwrap();
    let exit_command = matches.get_one::<String>("exit-command").unwrap();
    let copy_command = matches.get_one::<String>("copy-command").unwrap();
    let load_command = matches.get_one::<String>("load-command").unwrap();

    let app = TuiApp::new()?;
    let app = Arc::new(Mutex::new(app));

    app.with_lock(|x| x.start().unwrap()).unwrap();

    app.with_lock(|mut x| {
        x.start()?;
        x.redraw()
    })
    .unwrap()?;

    let app_clone = Arc::clone(&app);
    spawn(move || {
        let stderr_reader = setup_stderr_redirect().unwrap();
        let app = app_clone;

        let reader = BufReader::new(stderr_reader);
        for line in reader.lines().map(Result::unwrap) {
            app.with_lock(|mut x| {
                x.ui_data.log.push(line);
                x.redraw()
            })
            .unwrap()
            .expect("Redraw error");
        }
    });

    let mut traits = Traits::new();
    traits.set_user_data_dir(user_dir);
    traits.set_shared_data_dir(shared_dir);
    traits.set_distribution_name(DISTRIBUTION_NAME);
    traits.set_distribution_code_name(DISTRIBUTION_CODE_NAME);
    traits.set_distribution_version(DISTRIBUTION_VERSION);
    traits.set_app_name(APP_NAME);

    let mut engine = Engine::new(traits);
    let deploy_result = engine.wait_for_deploy_result(Duration::from_secs_f64(0.1));
    match deploy_result {
        DeployResult::Success => {
            eprintln!("Deployment succeeded");
        }
        DeployResult::Failure => {
            eprintln!("Deployment failed");
            return Ok(());
        }
    }
    engine.create_session()?;
    let session = engine.session().unwrap();
    if let Some(schema) = schema {
        session.select_schema(schema);
    }

    let engine = RefCell::new(engine);
    let mut key_resolver = KeyEventResolver::new(|ke| {
        let engine = engine.borrow();
        let session = engine.session().unwrap();
        let mut app = app.lock().unwrap();
        let ui_data = &mut app.ui_data;

        let key_status = session.process_key(ke);
        if key_status == KeyStatus::Pass && ke.modifiers == 0 {
            // default behaviors
            #[allow(non_upper_case_globals)]
            match ke.key_code as u32 {
                k @ XK_a..=XK_z => ui_data.output.push(char::from((k - XK_a) as u8 + b'a')),
                k @ XK_0..=XK_9 => ui_data.output.push(char::from((k - XK_0) as u8 + b'0')),
                XK_BackSpace => {
                    ui_data.output.pop();
                }
                XK_Return => ui_data.output.push('\n'),
                XK_space => ui_data.output.push(' '),
                _ => {}
            }
        }

        let context = session.context();
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
        let commit = session.commit();
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

        // custom commands
        let mut app_guard = app.lock().unwrap();
        let preedit = &app_guard.ui_data.preedit;

        let engine = engine.borrow();
        let session = engine.session().unwrap();
        match preedit.as_str() {
            _ if preedit == exit_command => {
                break;
            }
            _ if preedit == copy_command => {
                put_clipboard(app_guard.ui_data.output.as_str())?;
                app_guard.ui_data.preedit.clear();
                session.simulate_key_sequence("{Escape}")?;
                app_guard.redraw()?;
            }
            _ if preedit == load_command => {
                app_guard.ui_data.output = load_clipboard()?;
                app_guard.ui_data.preedit.clear();
                session.simulate_key_sequence("{Escape}")?;
                app_guard.redraw()?;
            }
            _ => {}
        }
        drop(app_guard);
    }

    drop(engine);
    app.with_lock(|mut x| x.stop()).unwrap()?;

    if let Some(r) = STDERR_REDIRECT.lock().unwrap().take() {
        drop(r);
    }

    Ok(())
}

fn setup_stderr_redirect() -> io::Result<FdReader> {
    let fds = unsafe {
        let mut fds = MaybeUninit::<[c_int; 2]>::uninit();
        pipe(fds.assume_init_mut().as_mut_ptr());
        fds.assume_init()
    };

    let read_fd = fds[0];
    let write_fd = fds[1];

    let redirect = Redirect::stderr(write_fd).unwrap();
    STDERR_REDIRECT.lock().unwrap().replace(redirect);
    let reader = FdReader::new(read_fd);
    Ok(reader)
}
