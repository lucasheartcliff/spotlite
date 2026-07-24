//! Spotlite Tauri host library. `run()` is the shared entrypoint used by the
//! desktop binary (`main.rs`).

mod auth;
mod commands;
mod config;
mod error;
mod spotify_api;
mod token_store;

use commands::AppState;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::new())
        .setup(|app| {
            register_media_keys(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::has_client_id,
            commands::is_authenticated,
            commands::mark_ready,
            commands::login,
            commands::logout,
            commands::get_access_token,
            commands::api_me,
            commands::api_search,
            commands::api_my_playlists,
            commands::api_playlist_tracks,
            commands::api_saved_tracks,
            commands::api_playback_state,
            commands::transfer_playback,
            commands::play,
            commands::pause,
            commands::next_track,
            commands::previous_track,
            commands::seek,
            commands::set_volume,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Spotlite");
}

/// Register OS media keys and forward them to the frontend, which drives the
/// Web Playback SDK. The frontend listens for the `media-key` event.
fn register_media_keys(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

    let handle = app.clone();
    let shortcuts = [
        (Code::MediaPlayPause, "playpause"),
        (Code::MediaTrackNext, "next"),
        (Code::MediaTrackPrevious, "previous"),
    ];

    let gs = app.global_shortcut();
    for (code, action) in shortcuts {
        let handle = handle.clone();
        let shortcut = Shortcut::new(None, code);
        // Registration can fail if another app grabbed the key; that's non-fatal.
        let _ = gs.on_shortcut(shortcut, move |_app, _sc, event| {
            if event.state() == ShortcutState::Pressed {
                let _ = handle.emit("media-key", action);
            }
        });
    }
    Ok(())
}
