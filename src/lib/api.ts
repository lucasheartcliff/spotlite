// Typed wrapper around the Tauri command surface. Tauri converts camelCase JS
// argument keys to the snake_case Rust parameter names automatically.

import { invoke } from "@tauri-apps/api/core";
import type { Me } from "./types";

export interface PlayOptions {
  contextUri?: string;
  uris?: string[];
  offsetPosition?: number;
}

export const api = {
  hasClientId: () => invoke<boolean>("has_client_id"),
  isAuthenticated: () => invoke<boolean>("is_authenticated"),
  markReady: () => invoke<void>("mark_ready"),

  login: () => invoke<Me>("login"),
  logout: () => invoke<void>("logout"),
  getAccessToken: () => invoke<string>("get_access_token"),

  me: () => invoke<Me>("api_me"),
  search: (query: string, types?: string, limit?: number) =>
    invoke<any>("api_search", { query, types, limit }),
  myPlaylists: (limit?: number) => invoke<any>("api_my_playlists", { limit }),
  playlistTracks: (playlistId: string, limit?: number, offset?: number) =>
    invoke<any>("api_playlist_tracks", { playlistId, limit, offset }),
  savedTracks: (limit?: number) => invoke<any>("api_saved_tracks", { limit }),
  playbackState: () => invoke<any>("api_playback_state"),

  transferPlayback: (deviceId: string, play?: boolean) =>
    invoke<any>("transfer_playback", { deviceId, play }),
  play: (deviceId: string, opts: PlayOptions = {}) =>
    invoke<any>("play", { deviceId, ...opts }),
  pause: (deviceId: string) => invoke<any>("pause", { deviceId }),
  next: (deviceId: string) => invoke<any>("next_track", { deviceId }),
  previous: (deviceId: string) => invoke<any>("previous_track", { deviceId }),
  seek: (deviceId: string, positionMs: number) =>
    invoke<any>("seek", { deviceId, positionMs }),
  setVolume: (deviceId: string, percent: number) =>
    invoke<any>("set_volume", { deviceId, percent }),
};
