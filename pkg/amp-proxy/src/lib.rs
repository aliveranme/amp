pub mod injector;
pub mod router;
pub mod streamer;
pub mod transformer;

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Config error: {0}")]
    Config(String),
    #[error("Upstream error: {0}")]
    Upstream(String),
}
