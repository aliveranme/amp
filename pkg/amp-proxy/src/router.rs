use std::collections::HashMap;
use std::path::Path;

use amp_core::ModelRoute;

#[derive(Debug)]
pub struct Router {
    routes: HashMap<String, ModelRoute>,
    fallback: ModelRoute,
}

impl Router {
    pub fn from_config(path: impl AsRef<Path>) -> Result<Self, crate::ProxyError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| crate::ProxyError::Config(format!("Cannot read route config: {e}")))?;
        let config: amp_core::RouteConfig = toml::from_str(&content)
            .map_err(|e| crate::ProxyError::Config(format!("Invalid route config: {e}")))?;
        Self::from_hashmap(config.routes)
    }

    pub fn from_hashmap(mut routes: HashMap<String, ModelRoute>) -> Result<Self, crate::ProxyError> {
        let fallback = routes.remove("*").ok_or_else(|| {
            crate::ProxyError::Config("Route config must have a '*' fallback route".to_string())
        })?;
        Ok(Self { routes, fallback })
    }

    pub fn route(&self, model: &str) -> &ModelRoute {
        self.routes.get(model).unwrap_or(&self.fallback)
    }
}
