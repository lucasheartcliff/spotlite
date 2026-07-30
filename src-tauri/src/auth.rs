//! OAuth 2.0 Authorization Code flow with PKCE.
//!
//! Flow:
//! 1. Generate a random `code_verifier` and its S256 `code_challenge`.
//! 2. Open the Spotify authorize URL in the user's system browser.
//! 3. Spotify redirects to our loopback server (`REDIRECT_URI`) with a `code`.
//! 4. Exchange the `code` (+ verifier) for access/refresh tokens.
//! 5. Later, refresh the access token as it nears expiry.

use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::config;
use crate::error::{AppError, AppResult};
use crate::token_store::{now_secs, Tokens};

/// PKCE pair held between building the authorize URL and the code exchange.
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn random_token(len: usize) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

impl Pkce {
    pub fn new() -> Self {
        let verifier = random_token(96);
        let digest = Sha256::digest(verifier.as_bytes());
        let challenge = b64url(&digest);
        let state = random_token(24);
        Pkce {
            verifier,
            challenge,
            state,
        }
    }
}

/// Build the Spotify authorize URL for the given PKCE challenge.
pub fn authorize_url(client_id: &str, pkce: &Pkce) -> String {
    let mut url = url::Url::parse(config::AUTH_URL).expect("valid auth url");
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", config::REDIRECT_URI)
        .append_pair("scope", config::SCOPES)
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", &pkce.challenge)
        .append_pair("state", &pkce.state);
    url.to_string()
}

/// Block on the loopback server until Spotify redirects back with a `code`.
/// Returns the authorization code. Validates the `state` parameter.
pub fn wait_for_code(expected_state: &str) -> AppResult<String> {
    let server = tiny_http::Server::http(("127.0.0.1", config::REDIRECT_PORT))
        .map_err(|e| AppError::Auth(format!("cannot bind loopback server: {e}")))?;

    for request in server.incoming_requests() {
        let url = format!("http://127.0.0.1{}", request.url());
        let parsed = url::Url::parse(&url).map_err(|e| AppError::Auth(e.to_string()))?;
        if parsed.path() != "/callback" {
            let _ = request
                .respond(tiny_http::Response::from_string("Not found").with_status_code(404));
            continue;
        }

        let mut code = None;
        let mut state = None;
        let mut error = None;
        for (k, v) in parsed.query_pairs() {
            match k.as_ref() {
                "code" => code = Some(v.into_owned()),
                "state" => state = Some(v.into_owned()),
                "error" => error = Some(v.into_owned()),
                _ => {}
            }
        }

        let (body, result) = if let Some(err) = error {
            (
                "Spotlite login failed. You can close this tab.".to_string(),
                Err(AppError::Auth(format!("authorization denied: {err}"))),
            )
        } else if state.as_deref() != Some(expected_state) {
            (
                "Spotlite login failed (state mismatch). You can close this tab.".to_string(),
                Err(AppError::Auth("state mismatch (possible CSRF)".into())),
            )
        } else if let Some(code) = code {
            (
                "Spotlite is now connected. You can close this tab and return to the app."
                    .to_string(),
                Ok(code),
            )
        } else {
            (
                "Spotlite login failed (no code). You can close this tab.".to_string(),
                Err(AppError::Auth("no authorization code in redirect".into())),
            )
        };

        let html = format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>Spotlite</title></head>\
             <body style=\"font-family:sans-serif;background:#121212;color:#fff;display:flex;\
             align-items:center;justify-content:center;height:100vh;margin:0\">\
             <h2>{body}</h2></body></html>"
        );
        let response = tiny_http::Response::from_string(html).with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                .unwrap(),
        );
        let _ = request.respond(response);
        return result;
    }

    Err(AppError::Auth("loopback server closed unexpectedly".into()))
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    /// Absent on refresh responses that reuse the existing refresh token.
    refresh_token: Option<String>,
    expires_in: u64,
}

/// Exchange an authorization code for a fresh token bundle.
pub async fn exchange_code(
    http: &reqwest::Client,
    client_id: &str,
    code: &str,
    verifier: &str,
) -> AppResult<Tokens> {
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", config::REDIRECT_URI),
        ("client_id", client_id),
        ("code_verifier", verifier),
    ];
    post_token(http, &params, None).await
}

/// Refresh an access token using the stored refresh token.
pub async fn refresh(
    http: &reqwest::Client,
    client_id: &str,
    refresh_token: &str,
) -> AppResult<Tokens> {
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
    ];
    post_token(http, &params, Some(refresh_token)).await
}

async fn post_token(
    http: &reqwest::Client,
    params: &[(&str, &str)],
    fallback_refresh: Option<&str>,
) -> AppResult<Tokens> {
    let resp = http.post(config::TOKEN_URL).form(params).send().await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(AppError::Auth(format!(
            "token endpoint {}: {}",
            status, text
        )));
    }
    let parsed: TokenResponse =
        serde_json::from_str(&text).map_err(|e| AppError::Auth(e.to_string()))?;

    let refresh_token = parsed
        .refresh_token
        .or_else(|| fallback_refresh.map(|s| s.to_string()))
        .ok_or_else(|| AppError::Auth("no refresh token returned".into()))?;

    Ok(Tokens {
        access_token: parsed.access_token,
        refresh_token,
        expires_at: now_secs() + parsed.expires_in,
    })
}
