//! Error type shared across the Rust host. Implements `serde::Serialize` so it
//! can be returned directly from Tauri commands as a JS-side `Error`.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("no Spotify client id configured; set SPOTLITE_CLIENT_ID")]
    MissingClientId,

    #[error("not authenticated; log in first")]
    NotAuthenticated,

    #[error("authentication flow failed: {0}")]
    Auth(String),

    #[error("Spotify API error ({status}): {body}")]
    Api { status: u16, body: String },

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("token store error: {0}")]
    Store(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
