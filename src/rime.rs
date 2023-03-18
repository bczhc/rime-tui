use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::time::Duration;
use std::{hint, thread};

use cstr::cstr;
use librime_sys::{RimeSessionId, RimeSetNotificationHandler};
use rime_api::{initialize, setup, start_maintenance, Commit, Context, KeyEvent, Session, Traits};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DeployResult {
    Init,
    Success,
    Failure,
}

pub struct Engine {
    pub session: Option<Session>,
    deploy_result: Box<DeployResult>,
}

pub struct Config {
    pub shared_data_dir: PathBuf,
    pub user_data_dir: PathBuf,
}

#[derive(Debug)]
pub enum RimeError {
    ProcessKey,
    CloseSession,
    SessionNotExists,
}

impl Display for RimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:?}", self).as_str())
    }
}

impl std::error::Error for RimeError {}

impl Engine {
    pub fn new(config: &Config) -> Engine {
        let mut traits = Traits::new();
        // TODO: use OsStr
        traits.set_shared_data_dir(
            config
                .shared_data_dir
                .to_string_lossy()
                .to_string()
                .as_str(),
        );
        traits.set_user_data_dir(config.user_data_dir.to_string_lossy().to_string().as_str());
        traits.set_distribution_name(DISTRIBUTION_NAME);
        traits.set_distribution_code_name(DISTRIBUTION_CODE_NAME);
        traits.set_distribution_version(DISTRIBUTION_VERSION);
        traits.set_app_name(APP_NAME);
        setup(&mut traits);

        extern "C" fn notification_handler(
            obj: *mut c_void,
            _session_id: RimeSessionId,
            message_type: *const c_char,
            message_value: *const c_char,
        ) {
            unsafe {
                let deploy_result = &mut *(obj as *mut DeployResult);
                let message_type = CStr::from_ptr(message_type);
                let message_value = CStr::from_ptr(message_value);
                if message_type == cstr!("deploy") {
                    match message_value {
                        _ if message_value == cstr!("success") => {
                            *deploy_result = DeployResult::Success
                        }
                        _ if message_value == cstr!("failure") => {
                            *deploy_result = DeployResult::Failure
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut deploy_result = Box::new(DeployResult::Init);

        unsafe {
            RimeSetNotificationHandler(
                Some(notification_handler),
                &mut *deploy_result as *mut DeployResult as *mut c_void,
            );
        }

        initialize(&mut traits);
        start_maintenance(true);
        Self {
            session: None,
            deploy_result,
        }
    }

    pub fn process_key(&mut self, event: KeyEvent) -> Result<(), RimeError> {
        let Some(session) = self.session.as_ref() else {
            return Err(RimeError::SessionNotExists)
        };
        session
            .process_key(event)
            .map_err(|_| RimeError::ProcessKey)?;
        Ok(())
    }

    pub fn context(&mut self) -> Option<Context> {
        self.session.as_ref()?.context()
    }

    pub fn commit(&mut self) -> Option<Commit> {
        self.session.as_ref()?.commit()
    }

    /// Note when using this, function `start_maintenance` needs `full_check` to be `true`.
    pub fn wait_for_deploy_result(&mut self, interval: Duration) -> DeployResult {
        #[allow(clippy::while_immutable_condition)]
        while *self.deploy_result == DeployResult::Init {
            thread::sleep(interval);
            hint::spin_loop();
        }
        *self.deploy_result
    }

    pub fn close(&mut self) -> Result<(), RimeError> {
        if let Some(session) = self.session.as_ref() {
            if session.find_session() {
                session.close().map_err(|_| RimeError::CloseSession)?;
            }
        }
        Ok(())
    }
}

pub fn get_user_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    match home::home_dir() {
        None => PathBuf::new(),
        Some(mut home) => {
            home.push(".local/share/fcitx5/rime");
            home
        }
    }

    #[cfg(not(target_os = "linux"))]
    // TODO
    PathBuf::new()
}

pub fn get_shared_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    let dir = PathBuf::from("/usr/share/rime-data/");
    #[cfg(not(target_os = "linux"))]
    // TODO
    let dir = PathBuf::new();

    dir
}

const DISTRIBUTION_NAME: &str = "Rime";
const DISTRIBUTION_CODE_NAME: &str = "Rime";
const DISTRIBUTION_VERSION: &str = "0.0.0";
const APP_NAME: &str = "rime-tui";
