//! Tauri command surface exposed to the webview. All Spotify access is funneled
//! through here so token refresh stays centralized and the access token is
//! minted on demand rather than persisted in JS.

use serde_json::{json, Value};
use tauri::State;
use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::token_store::Tokens;
use crate::{auth, config, spotify_api, token_store};

/// Shared application state, managed by Tauri.
pub struct AppState {
    pub http: reqwest::Client,
    pub client_id: Option<String>,
    pub tokens: Mutex<Option<Tokens>>,
    /// Process start instant — used by the startup benchmark.
    pub started_at: std::time::Instant,
}

impl AppState {
    pub fn new() -> Self {
        // Load any persisted tokens on startup.
        let tokens = token_store::load().unwrap_or(None);
        AppState {
            http: reqwest::Client::new(),
            client_id: config::client_id(),
            tokens: Mutex::new(tokens),
            started_at: std::time::Instant::now(),
        }
    }

    /// Return a valid (non-expired) access token, refreshing if necessary.
    async fn valid_token(&self) -> AppResult<String> {
        let client_id = self.client_id.clone().ok_or(AppError::MissingClientId)?;
        let mut guard = self.tokens.lock().await;
        let current = guard.clone().ok_or(AppError::NotAuthenticated)?;
        if current.is_expired() {
            let refreshed = auth::refresh(&self.http, &client_id, &current.refresh_token).await?;
            token_store::save(&refreshed)?;
            let access = refreshed.access_token.clone();
            *guard = Some(refreshed);
            Ok(access)
        } else {
            Ok(current.access_token)
        }
    }
}

#[tauri::command]
pub fn has_client_id(state: State<'_, AppState>) -> bool {
    state.client_id.is_some()
}

/// Called once by the frontend when the UI has mounted and is interactive.
/// Emits a machine-parseable line the startup benchmark scrapes from stdout,
/// giving a precise "launch → interactive" figure for Spotlite.
#[tauri::command]
pub fn mark_ready(app: tauri::AppHandle, state: State<'_, AppState>) {
    use tauri::Manager;
    let ms = state.started_at.elapsed().as_millis();
    println!("SPOTLITE_READY_MS={ms}");
    // Reveal the window only once the UI is ready, avoiding a white flash.
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[tauri::command]
pub async fn is_authenticated(state: State<'_, AppState>) -> AppResult<bool> {
    Ok(state.tokens.lock().await.is_some())
}

/// Run the full PKCE login: open the browser, capture the redirect, exchange
/// the code, persist tokens, and return the user profile.
#[tauri::command]
pub async fn login(app: tauri::AppHandle, state: State<'_, AppState>) -> AppResult<Value> {
    use tauri_plugin_opener::OpenerExt;

    let client_id = state.client_id.clone().ok_or(AppError::MissingClientId)?;
    let pkce = auth::Pkce::new();
    let url = auth::authorize_url(&client_id, &pkce);
    let expected_state = pkce.state.clone();

    // Open the authorize URL in the system browser.
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Auth(e.to_string()))?;

    // The loopback server is blocking — run it off the async runtime.
    let code = tokio::task::spawn_blocking(move || auth::wait_for_code(&expected_state))
        .await
        .map_err(|e| AppError::Auth(e.to_string()))??;

    let tokens = auth::exchange_code(&state.http, &client_id, &code, &pkce.verifier).await?;
    token_store::save(&tokens)?;
    let access = tokens.access_token.clone();
    *state.tokens.lock().await = Some(tokens);

    spotify_api::me(&state.http, &access).await
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> AppResult<()> {
    token_store::clear()?;
    *state.tokens.lock().await = None;
    Ok(())
}

/// Used by the Web Playback SDK's `getOAuthToken` callback.
#[tauri::command]
pub async fn get_access_token(state: State<'_, AppState>) -> AppResult<String> {
    state.valid_token().await
}

#[tauri::command]
pub async fn api_me(state: State<'_, AppState>) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::me(&state.http, &token).await
}

#[tauri::command]
pub async fn api_search(
    state: State<'_, AppState>,
    query: String,
    types: Option<String>,
    limit: Option<u32>,
) -> AppResult<Value> {
    let token = state.valid_token().await?;
    let types = types.unwrap_or_else(|| "track,album,artist,playlist".to_string());
    spotify_api::search(&state.http, &token, &query, &types, limit.unwrap_or(20)).await
}

#[tauri::command]
pub async fn api_my_playlists(state: State<'_, AppState>, limit: Option<u32>) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::my_playlists(&state.http, &token, limit.unwrap_or(50)).await
}

#[tauri::command]
pub async fn api_playlist_tracks(
    state: State<'_, AppState>,
    playlist_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::playlist_tracks(
        &state.http,
        &token,
        &playlist_id,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    )
    .await
}

#[tauri::command]
pub async fn api_saved_tracks(state: State<'_, AppState>, limit: Option<u32>) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::saved_tracks(&state.http, &token, limit.unwrap_or(50)).await
}

#[tauri::command]
pub async fn api_playback_state(state: State<'_, AppState>) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::playback_state(&state.http, &token).await
}

#[tauri::command]
pub async fn transfer_playback(
    state: State<'_, AppState>,
    device_id: String,
    play: Option<bool>,
) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::transfer_playback(&state.http, &token, &device_id, play.unwrap_or(false)).await
}

#[tauri::command]
pub async fn play(
    state: State<'_, AppState>,
    device_id: String,
    context_uri: Option<String>,
    uris: Option<Vec<String>>,
    offset_position: Option<u32>,
) -> AppResult<Value> {
    let token = state.valid_token().await?;
    let mut body = json!({});
    if let Some(ctx) = context_uri {
        body["context_uri"] = json!(ctx);
    }
    if let Some(uris) = uris {
        body["uris"] = json!(uris);
    }
    if let Some(pos) = offset_position {
        body["offset"] = json!({ "position": pos });
    }
    spotify_api::play(&state.http, &token, &device_id, body).await
}

#[tauri::command]
pub async fn pause(state: State<'_, AppState>, device_id: String) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::pause(&state.http, &token, &device_id).await
}

#[tauri::command]
pub async fn next_track(state: State<'_, AppState>, device_id: String) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::next(&state.http, &token, &device_id).await
}

#[tauri::command]
pub async fn previous_track(state: State<'_, AppState>, device_id: String) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::previous(&state.http, &token, &device_id).await
}

#[tauri::command]
pub async fn seek(
    state: State<'_, AppState>,
    device_id: String,
    position_ms: u64,
) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::seek(&state.http, &token, &device_id, position_ms).await
}

#[tauri::command]
pub async fn set_volume(
    state: State<'_, AppState>,
    device_id: String,
    percent: u8,
) -> AppResult<Value> {
    let token = state.valid_token().await?;
    spotify_api::set_volume(&state.http, &token, &device_id, percent).await
}
