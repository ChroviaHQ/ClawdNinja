use clawdninja_core::SettingsError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub detail: String,
}

impl CommandError {
    pub fn new(code: &'static str, detail: impl ToString) -> Self {
        Self { code, detail: detail.to_string() }
    }

    pub fn code(code: &'static str) -> Self {
        Self::new(code, "")
    }
}

impl From<SettingsError> for CommandError {
    fn from(error: SettingsError) -> Self {
        let code = match error {
            SettingsError::ConfigDirUnknown => "config_dir_unknown",
            SettingsError::ConfigDirRelative => "config_dir_relative",
            SettingsError::Malformed(_) | SettingsError::Json(_) => "settings_invalid",
            SettingsError::Io(_) => "settings_io_failed",
        };
        Self::new(code, error)
    }
}
