use std::env;
use std::path::Path;

use owly_core::config::{Config, ConfigContext};

const DEFAULT_CONFIG_NAME: &str = "owly.yml";

pub fn load(config_path: Option<&Path>) -> Result<ConfigContext, String> {
    let path = match config_path {
        Some(path) => path.to_path_buf(),
        None => env::current_dir()
            .map_err(|error| format!("cannot determine current directory: {error}"))?
            .join(DEFAULT_CONFIG_NAME),
    };
    if !path.is_file() {
        return Err(format!("configuration file not found: {}", path.display()));
    }
    let config = Config::from_path(&path)?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    ConfigContext::new(config, base_dir.to_path_buf())
}
