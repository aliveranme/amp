use amp_core::ModelRoute;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub fn build_headers(route: &ModelRoute, api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // Auth header
    if let Some(header_name) = &route.auth_header {
        let value = match &route.auth_scheme {
            Some(scheme) => format!("{scheme} {api_key}"),
            None => api_key.to_string(),
        };
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(header_name.as_bytes()),
            HeaderValue::from_str(&value),
        ) {
            headers.insert(name, val);
        }
    }

    // Extra headers
    if let Some(extra) = &route.extra_headers {
        for (k, v) in extra {
            let resolved = v.replace("${API_KEY}", api_key);
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(&resolved),
            ) {
                headers.insert(name, val);
            }
        }
    }

    headers
}
