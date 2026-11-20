use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};
use tempfile::NamedTempFile;

use crate::{is_proxy_variable, variables};

const CONFIG_DIR_VARIABLE: &str = "CLAUDE_CONFIG_DIR";
const UTF8_BOM: &[u8] = &[0xef, 0xbb, 0xbf];

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot locate the Claude config directory")]
    ConfigDirUnknown,
    #[error("{CONFIG_DIR_VARIABLE} must be an absolute path")]
    ConfigDirRelative,
    #[error("{0}")]
    Malformed(&'static str),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn settings_path() -> Result<PathBuf, SettingsError> {
    let root = std::env::var_os(CONFIG_DIR_VARIABLE)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".claude")))
        .ok_or(SettingsError::ConfigDirUnknown)?;
    if !root.is_absolute() {
        return Err(SettingsError::ConfigDirRelative);
    }
    Ok(root.join("settings.json"))
}

pub fn is_configured(path: &Path) -> Result<bool, SettingsError> {
    let data = read(path)?;
    Ok(data["env"].as_object().is_some_and(|env| {
        env.iter().any(|(key, value)| value.as_str().is_some_and(|v| is_proxy_variable(key, v)))
    }))
}

pub fn write(path: &Path, url: &str) -> Result<(), SettingsError> {
    let mut data = read(path)?;
    let env = env_mut(&mut data);
    for (key, value) in variables(url) {
        env.insert(key.to_owned(), json!(value));
    }
    replace(path, &data)
}

pub fn remove(path: &Path) -> Result<(), SettingsError> {
    let mut data = read(path)?;
    let env = env_mut(&mut data);
    let before = env.len();
    env.retain(|key, value| !value.as_str().is_some_and(|v| is_proxy_variable(key, v)));
    if env.len() == before {
        return Ok(());
    }
    if env.is_empty()
        && let Some(root) = data.as_object_mut()
    {
        root.remove("env");
    }
    replace(path, &data)
}

fn read(path: &Path) -> Result<Value, SettingsError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(json!({})),
        Err(error) => return Err(error.into()),
    };
    let data: Value = serde_json::from_slice(bytes.strip_prefix(UTF8_BOM).unwrap_or(&bytes))?;
    if !data.is_object() {
        return Err(SettingsError::Malformed("settings.json root must be an object"));
    }
    if data.get("env").is_some_and(|env| !env.is_object()) {
        return Err(SettingsError::Malformed("settings.json env must be an object"));
    }
    Ok(data)
}

fn env_mut(data: &mut Value) -> &mut Map<String, Value> {
    if !data["env"].is_object() {
        data["env"] = Value::Object(Map::new());
    }
    data["env"].as_object_mut().expect("env is an object")
}

fn replace(path: &Path, data: &Value) -> Result<(), SettingsError> {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_owned());
    let dir = path.parent().ok_or(SettingsError::ConfigDirUnknown)?;
    fs::create_dir_all(dir)?;
    let mut temp = NamedTempFile::new_in(dir)?;
    serde_json::to_writer_pretty(&mut temp, data)?;
    temp.write_all(b"\n")?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|error| error.error)?;
    Ok(())
}
