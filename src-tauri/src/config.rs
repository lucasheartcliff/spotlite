//! Static configuration for the Spotify integration.
//!
//! The Spotify **client id** identifies the app. For an installed desktop app
//! using Authorization Code + PKCE there is *no client secret* (it is a public
//! client), so the client id is safe to ship. It is resolved at runtime from
//! the `SPOTLITE_CLIENT_ID` environment variable, falling back to a value
//! baked in at compile time (also via `SPOTLITE_CLIENT_ID`). Register your own
//! app at https://developer.spotify.com/dashboard and add the redirect URI
//! below to it.

/// Loopback redirect. Must be registered verbatim in the Spotify dashboard.
pub const REDIRECT_PORT: u16 = 8888;
pub const REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";

pub const AUTH_URL: &str = "https://accounts.spotify.com/authorize";
pub const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
pub const API_BASE: &str = "https://api.spotify.com/v1";

/// Scopes required for browsing the library and driving the Web Playback SDK.
pub const SCOPES: &str = "streaming \
     user-read-email \
     user-read-private \
     user-read-playback-state \
     user-modify-playback-state \
     user-read-currently-playing \
     user-library-read \
     playlist-read-private \
     playlist-read-collaborative";

/// Resolve the Spotify client id. Runtime env var wins over the compile-time
/// baked value so users can supply their own without rebuilding.
pub fn client_id() -> Option<String> {
    if let Ok(v) = std::env::var("SPOTLITE_CLIENT_ID") {
        if !v.trim().is_empty() {
            return Some(v.trim().to_string());
        }
    }
    match option_env!("SPOTLITE_CLIENT_ID") {
        Some(v) if !v.trim().is_empty() => Some(v.trim().to_string()),
        _ => None,
    }
}
