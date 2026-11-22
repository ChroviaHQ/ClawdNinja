use std::fmt::Display;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Mutex, PoisonError};

pub struct Preference<T> {
    path: PathBuf,
    value: Mutex<T>,
}

impl<T: Copy + PartialEq + FromStr + Display> Preference<T> {
    pub fn load(path: PathBuf, default: T) -> Self {
        let value = fs::read_to_string(&path)
            .ok()
            .and_then(|text| text.trim().parse().ok())
            .unwrap_or(default);
        Self { path, value: Mutex::new(value) }
    }

    pub fn get(&self) -> T {
        *self.value.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn set(&self, value: T) {
        let mut saved = self.value.lock().unwrap_or_else(PoisonError::into_inner);
        if *saved == value {
            return;
        }
        if let Err(error) = persist(&self.path, value) {
            tracing::warn!(%error, path = %self.path.display(), "cannot save preference");
        }
        *saved = value;
    }
}

fn persist(path: &Path, value: impl Display) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, value.to_string())
}
