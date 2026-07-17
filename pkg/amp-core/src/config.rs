use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRoute {
    pub provider: String,
    pub endpoint: String,
    pub auth_header: Option<String>,
    pub auth_scheme: Option<String>,
    pub extra_headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    #[serde(rename = "route")]
    pub routes: HashMap<String, ModelRoute>,
}

/// JSON settings file (~/.config/amp/settings.json)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsFile {
    pub api_key: Option<String>,
    pub url: Option<String>,
    pub default_model: Option<String>,
    pub route_config_path: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub db_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub url: Option<String>,
    pub default_model: String,
    pub route_config_path: Option<String>,
    pub host: String,
    pub port: u16,
    pub db_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            url: None,
            default_model: "gpt-4o".to_string(),
            route_config_path: None,
            host: "127.0.0.1".to_string(),
            port: 8080,
            db_path: "amp-code.db".to_string(),
        }
    }
}

impl AppConfig {
    /// Load config from environment variables, falling back to settings file.
    pub fn load() -> Self {
        let env = Self::from_env();
        let file = SettingsFile::load();

        Self {
            api_key: take(&env.api_key, &file.api_key, ""),
            url: env.url.or(file.url),
            default_model: take(&env.default_model, &file.default_model, "gpt-4o"),
            route_config_path: env.route_config_path.or(file.route_config_path),
            host: take(&env.host, &file.host, "127.0.0.1"),
            port: file.port.unwrap_or(env.port),
            db_path: take(&env.db_path, &file.db_path, "amp-code.db"),
        }
    }

    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("AMP_API_KEY").unwrap_or_default(),
            url: std::env::var("AMP_URL").ok(),
            default_model: std::env::var("AMP_MODEL_DEFAULT")
                .unwrap_or_else(|_| "gpt-4o".to_string()),
            route_config_path: std::env::var("AMP_MODEL_ROUTE").ok(),
            host: std::env::var("AMP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("AMP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            db_path: std::env::var("AMP_DB_PATH").unwrap_or_else(|_| "amp-code.db".to_string()),
        }
    }
}

impl SettingsFile {
    /// Load from ~/.config/amp/settings.json (amp-compatible global config)
    pub fn load() -> Self {
        let path = dirs::config_dir()
            .map(|p| p.join("amp").join("settings.json"))
            .unwrap_or_else(|| PathBuf::from("settings.json"));

        match std::fs::File::open(&path) {
            Ok(f) => {
                match serde_json::from_reader(f) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("Failed to parse {path:?}: {e}");
                        Self::default()
                    }
                }
            }
            Err(_) => Self::default(),
        }
    }
}

/// Take first non-empty, then fallback, then default.
fn take(primary: &str, fallback: &Option<String>, default: &str) -> String {
    if !primary.is_empty() {
        primary.to_string()
    } else if let Some(val) = fallback {
        if !val.is_empty() {
            return val.to_string();
        }
        default.to_string()
    } else {
        default.to_string()
    }
}
