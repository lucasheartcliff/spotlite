//! Thin wrapper over the Spotify Web API. Methods take an access token and
//! return raw `serde_json::Value` — the frontend owns presentation, so we
//! avoid modeling every field here. Playback *audio* is handled by the Web
//! Playback SDK in the webview; these endpoints cover metadata and remote
//! control (search, library, transfer/play/pause/seek/volume).

use serde_json::Value;

use crate::config::API_BASE;
use crate::error::{AppError, AppResult};

/// Reqwest wrappers. `Method` is re-exported so commands can stay terse.
pub use reqwest::Method;

async fn request(
    http: &reqwest::Client,
    token: &str,
    method: Method,
    path_and_query: &str,
    body: Option<Value>,
) -> AppResult<Value> {
    let url = format!("{API_BASE}{path_and_query}");
    let mut req = http.request(method, &url).bearer_auth(token);
    if let Some(body) = body {
        req = req.json(&body);
    }
    let resp = req.send().await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        return Err(AppError::Api {
            status: status.as_u16(),
            body: text,
        });
    }

    // Several control endpoints (play/pause/next/...) return 204 No Content.
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&text).map_err(|e| AppError::Api {
        status: status.as_u16(),
        body: e.to_string(),
    })
}

pub async fn get(http: &reqwest::Client, token: &str, path_and_query: &str) -> AppResult<Value> {
    request(http, token, Method::GET, path_and_query, None).await
}

pub async fn me(http: &reqwest::Client, token: &str) -> AppResult<Value> {
    get(http, token, "/me").await
}

pub async fn search(
    http: &reqwest::Client,
    token: &str,
    query: &str,
    types: &str,
    limit: u32,
) -> AppResult<Value> {
    let q = urlencode(query);
    let path = format!("/search?q={q}&type={types}&limit={limit}");
    get(http, token, &path).await
}

pub async fn my_playlists(http: &reqwest::Client, token: &str, limit: u32) -> AppResult<Value> {
    get(http, token, &format!("/me/playlists?limit={limit}")).await
}

pub async fn playlist_tracks(
    http: &reqwest::Client,
    token: &str,
    playlist_id: &str,
    limit: u32,
    offset: u32,
) -> AppResult<Value> {
    let path = format!("/playlists/{playlist_id}/tracks?limit={limit}&offset={offset}");
    get(http, token, &path).await
}

pub async fn saved_tracks(http: &reqwest::Client, token: &str, limit: u32) -> AppResult<Value> {
    get(http, token, &format!("/me/tracks?limit={limit}")).await
}

pub async fn playback_state(http: &reqwest::Client, token: &str) -> AppResult<Value> {
    get(http, token, "/me/player").await
}

/// Move active playback to a device (our Web Playback SDK device id).
pub async fn transfer_playback(
    http: &reqwest::Client,
    token: &str,
    device_id: &str,
    play: bool,
) -> AppResult<Value> {
    let body = serde_json::json!({ "device_ids": [device_id], "play": play });
    request(http, token, Method::PUT, "/me/player", Some(body)).await
}

/// Start/resume playback. `context_uri` (album/playlist) or `uris` (tracks).
pub async fn play(
    http: &reqwest::Client,
    token: &str,
    device_id: &str,
    body: Value,
) -> AppResult<Value> {
    let path = format!("/me/player/play?device_id={device_id}");
    request(http, token, Method::PUT, &path, Some(body)).await
}

pub async fn pause(http: &reqwest::Client, token: &str, device_id: &str) -> AppResult<Value> {
    let path = format!("/me/player/pause?device_id={device_id}");
    request(http, token, Method::PUT, &path, None).await
}

pub async fn next(http: &reqwest::Client, token: &str, device_id: &str) -> AppResult<Value> {
    let path = format!("/me/player/next?device_id={device_id}");
    request(http, token, Method::POST, &path, None).await
}

pub async fn previous(http: &reqwest::Client, token: &str, device_id: &str) -> AppResult<Value> {
    let path = format!("/me/player/previous?device_id={device_id}");
    request(http, token, Method::POST, &path, None).await
}

pub async fn seek(
    http: &reqwest::Client,
    token: &str,
    device_id: &str,
    position_ms: u64,
) -> AppResult<Value> {
    let path = format!("/me/player/seek?device_id={device_id}&position_ms={position_ms}");
    request(http, token, Method::PUT, &path, None).await
}

pub async fn set_volume(
    http: &reqwest::Client,
    token: &str,
    device_id: &str,
    percent: u8,
) -> AppResult<Value> {
    let percent = percent.min(100);
    let path = format!("/me/player/volume?device_id={device_id}&volume_percent={percent}");
    request(http, token, Method::PUT, &path, None).await
}

fn urlencode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
