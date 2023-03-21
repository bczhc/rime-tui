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
use x11::keysym::*;

use rime_tui::cli::build_cli;
use rime_tui::fd_reader::FdReader;
use rime_tui::key_event::KeyEventResolver;
use rime_tui::rime::{Config, DeployResult, Engine};
use rime_tui::tui::{Candidate, TuiApp};
use rime_tui::xinput::XInput;
use rime_tui::WithLockExt;

static STDERR_REDIRECT: Lazy<Mutex<Option<Redirect<RawFd>>>> = Lazy::new(|| Mutex::new(None));

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();
    let schema = matches.get_one::<String>("schema").unwrap();
    let user_dir = matches.get_one::<String>("user-dir").unwrap();
    let shared_dir = matches.get_one::<String>("shared-dir").unwrap();
    let exit_command = matches.get_one::<String>("exit-command").unwrap();

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

    let engine = RefCell::new(engine);
    let mut key_resolver = KeyEventResolver::new(|ke| {
        let mut engine = engine.borrow_mut();
        let mut app = app.lock().unwrap();
        let ui_data = &mut app.ui_data;

        // kAccepted: true, otherwise false
        let result = engine.process_key(ke).unwrap();
        if !result && ke.modifiers == 0 {
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

        if &app.lock().unwrap().ui_data.preedit == exit_command {
            break;
        }
    }

    engine.borrow_mut().close()?;
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
