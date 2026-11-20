mod settings;

pub use settings::{SettingsError, is_configured, remove, settings_path, write};

use std::num::NonZeroU16;

const PROXY_KEYS: [&str; 6] =
    ["HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"];
const FIXED_VARIABLES: [(&str, &str); 3] =
    [("NO_PROXY", LOCAL_BYPASS), ("no_proxy", LOCAL_BYPASS), ("NODE_USE_ENV_PROXY", "1")];
const LOCAL_BYPASS: &str = "localhost,127.0.0.1,::1";
const LOCAL_PREFIX: &str = "http://127.0.0.1:";

pub fn proxy_url(port: NonZeroU16) -> String {
    format!("{LOCAL_PREFIX}{port}")
}

pub fn variables(url: &str) -> impl Iterator<Item = (&'static str, &str)> {
    PROXY_KEYS.into_iter().map(move |key| (key, url)).chain(FIXED_VARIABLES)
}

fn is_proxy_variable(key: &str, value: &str) -> bool {
    if let Some((_, fixed)) = FIXED_VARIABLES.iter().find(|(name, _)| *name == key) {
        return value == *fixed;
    }
    PROXY_KEYS.contains(&key)
        && value
            .strip_prefix(LOCAL_PREFIX)
            .and_then(|port| port.parse::<NonZeroU16>().ok())
            .is_some()
}
