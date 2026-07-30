// Web Playback SDK integration. This is where audio actually plays: the SDK
// registers a Connect device inside the WebView2 renderer and decrypts audio
// via Widevine/EME. We keep a normalized snapshot in the `player` store and
// expose thin control helpers. Remote control (play a context/track) goes
// through the Web API in `api.ts`; local transport (toggle/seek/next/prev)
// uses the SDK directly for lower latency.

import { api } from "./api";
import { player, showToast } from "./stores";
import type { Track } from "./types";

const SDK_URL = "https://sdk.scdn.co/spotify-player.js";
const DEVICE_NAME = "Spotlite";

let sdkPlayer: Spotify.Player | null = null;
let deviceId: string | null = null;
let pollTimer: ReturnType<typeof setInterval> | null = null;

/** Inject the SDK script once and resolve when the global is ready. */
function loadSdk(): Promise<void> {
  return new Promise((resolve, reject) => {
    if (window.Spotify) return resolve();
    window.onSpotifyWebPlaybackSDKReady = () => resolve();
    const script = document.createElement("script");
    script.src = SDK_URL;
    script.async = true;
    script.onerror = () => reject(new Error("failed to load Web Playback SDK"));
    document.head.appendChild(script);
  });
}

/** Initialize the SDK player and make Spotlite the active playback device. */
export async function initPlayback(): Promise<void> {
  await loadSdk();

  const p = new window.Spotify.Player({
    name: DEVICE_NAME,
    // The SDK asks for a token here (and again on expiry). We mint a fresh one
    // from the Rust host, which handles refresh transparently.
    getOAuthToken: (cb) => {
      api.getAccessToken().then(cb).catch((e) => showToast(`Auth error: ${e}`));
    },
    volume: 0.8,
  });
  sdkPlayer = p;

  p.addListener("initialization_error", ({ message }: Spotify.WebPlaybackError) =>
    showToast(`Playback init failed: ${message}`),
  );
  p.addListener("authentication_error", ({ message }: Spotify.WebPlaybackError) =>
    showToast(`Playback auth failed: ${message}`),
  );
  p.addListener("account_error", () =>
    showToast("Spotify Premium is required for in-app playback."),
  );

  p.addListener("ready", async ({ device_id }: { device_id: string }) => {
    deviceId = device_id;
    player.update((s) => ({ ...s, ready: true, deviceId: device_id }));
    // Make Spotlite the active device (without auto-starting playback).
    try {
      await api.transferPlayback(device_id, false);
    } catch (e) {
      showToast(`Could not activate device: ${e}`);
    }
  });

  p.addListener("not_ready", () => {
    player.update((s) => ({ ...s, ready: false }));
  });

  p.addListener("player_state_changed", (state: any) => {
    if (!state) return;
    const current = state.track_window?.current_track;
    const track: Track | null = current
      ? {
          id: current.id,
          name: current.name,
          uri: current.uri,
          duration_ms: current.duration_ms,
          artists: current.artists,
          album: current.album,
        }
      : null;
    player.update((s) => ({
      ...s,
      paused: state.paused,
      positionMs: state.position,
      durationMs: state.duration,
      track,
    }));
  });

  const ok = await p.connect();
  if (!ok) showToast("Could not connect the playback device.");

  // Advance the position locally between SDK events so the seek bar is smooth.
  startPositionPolling();
}

function startPositionPolling() {
  if (pollTimer) clearInterval(pollTimer);
  pollTimer = setInterval(() => {
    player.update((s) => {
      if (s.paused || !s.track) return s;
      return { ...s, positionMs: Math.min(s.positionMs + 1000, s.durationMs) };
    });
  }, 1000);
}

export function getDeviceId(): string | null {
  return deviceId;
}

// --- Transport controls (SDK-local for responsiveness) ---

export async function togglePlay() {
  await sdkPlayer?.togglePlay();
}
export async function nextTrack() {
  await sdkPlayer?.nextTrack();
}
export async function previousTrack() {
  await sdkPlayer?.previousTrack();
}
export async function seek(positionMs: number) {
  await sdkPlayer?.seek(positionMs);
  player.update((s) => ({ ...s, positionMs }));
}
export async function setVolume(fraction: number) {
  await sdkPlayer?.setVolume(fraction);
}

/** Start playing a context (playlist/album) or specific track uris. */
export async function playContext(opts: {
  contextUri?: string;
  uris?: string[];
  offsetPosition?: number;
}) {
  if (!deviceId) {
    showToast("Playback device not ready yet.");
    return;
  }
  try {
    await api.play(deviceId, opts);
  } catch (e) {
    showToast(`Playback failed: ${e}`);
  }
}
