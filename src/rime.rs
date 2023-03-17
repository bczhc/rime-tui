use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::time::Duration;
use std::{hint, thread};

use rime_api::{
    create_session, initialize, setup, start_maintenance, Commit, Context, KeyEvent, Session,
    Traits,
};

pub struct Engine {
    pub session: Session,
}

pub struct Config {
    pub shared_data_dir: PathBuf,
    pub user_data_dir: PathBuf,
}

#[derive(Debug)]
pub enum RimeError {
    ProcessKey,
    CloseSession,
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
        initialize(&mut traits);
        start_maintenance(false);
        Self {
            session: create_session(),
        }
    }

    pub fn process_key(&mut self, event: KeyEvent) -> Result<(), RimeError> {
        // Rime engine hasn't prepared for processing keys,
        // retry and create the session
        if !self.session.find_session() {
            self.session = create_session();
        }
        self.session
            .process_key(event)
            .map_err(|_| RimeError::ProcessKey)?;
        Ok(())
    }

    pub fn context(&mut self) -> Option<Context> {
        self.session.context()
    }

    pub fn commit(&mut self) -> Option<Commit> {
        self.session.commit()
    }

    pub fn wait_for_session_created(&mut self, interval: Duration) {
        while !self.session.find_session() {
            self.session = create_session();
            thread::sleep(interval);
            hint::spin_loop();
        }
    }

    pub fn close(&mut self) -> Result<(), RimeError> {
        if self.session.find_session() {
            self.session.close().map_err(|_| RimeError::CloseSession)?;
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
