use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("AMP_API_KEY").unwrap_or_default(),
            url: std::env::var("AMP_URL").ok(),
            default_model: std::env::var("AMP_MODEL_DEFAULT")
                .unwrap_or_else(|_| "gpt-4o".to_string()),
            route_config_path: std::env::var("AMP_MODEL_ROUTE").ok(),
            ..Default::default()
        }
    }
}
