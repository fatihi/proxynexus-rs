use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyNexusError {
    #[error("Database error: {0}")]
    Database(#[from] gluesql::core::error::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[cfg(target_arch = "wasm32")]
    #[error("Network error: {0}")]
    Network(#[from] gloo_net::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ProxyNexusError>;
