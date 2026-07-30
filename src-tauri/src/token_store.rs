//! Secure token persistence. On Windows this stores the token bundle in the
//! **Windows Credential Manager** via the `keyring` crate; the token never
//! touches plaintext on disk.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const SERVICE: &str = "app.spotlite.desktop";
const ACCOUNT: &str = "spotify-tokens";

/// The full OAuth token bundle we persist between sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix seconds at which `access_token` expires.
    pub expires_at: u64,
}

impl Tokens {
    /// True when the access token is expired or within 60s of expiring.
    pub fn is_expired(&self) -> bool {
        now_secs() + 60 >= self.expires_at
    }
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn entry() -> AppResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, ACCOUNT).map_err(|e| AppError::Store(e.to_string()))
}

/// Load the persisted token bundle, if any.
pub fn load() -> AppResult<Option<Tokens>> {
    let entry = entry()?;
    match entry.get_password() {
        Ok(json) => {
            let tokens = serde_json::from_str(&json).map_err(|e| AppError::Store(e.to_string()))?;
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Store(e.to_string())),
    }
}

/// Persist the token bundle.
pub fn save(tokens: &Tokens) -> AppResult<()> {
    let json = serde_json::to_string(tokens).map_err(|e| AppError::Store(e.to_string()))?;
    entry()?
        .set_password(&json)
        .map_err(|e| AppError::Store(e.to_string()))
}

/// Remove any stored credentials (logout).
pub fn clear() -> AppResult<()> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Store(e.to_string())),
    }
}
