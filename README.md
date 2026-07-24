# Spotlite

A **lightweight Spotify desktop client** built on **Rust + Tauri 2** with the
OS-native **WebView2** on Windows. Instead of bundling a full Chromium runtime
like the official CEF-based app, Spotlite reuses the system webview, keeping the
binary and memory footprint small. Audio plays through Spotify's official
**Web Playback SDK** (Widevine/EME); browsing and control go through the
**Spotify Web API**, proxied by the Rust host.

> **Requires Spotify Premium.** The Web Playback SDK only streams for Premium
> accounts.

## How it works

```
Tauri host (Rust)                     WebView2 (Chromium) frontend (Svelte)
─────────────────                     ──────────────────────────────────────
• OAuth PKCE + loopback redirect  ─▶   • Login, browse, search, library UI
• Tokens in Windows Credential Mgr     • Web Playback SDK  ── plays audio (EME)
• Web API proxy commands          ◀─   • Calls host commands for all API access
• Global media-key shortcuts       ─▶  • media-key events drive transport
```

- **Audio** is decoded/played by the Web Playback SDK inside WebView2. The Rust
  side never touches raw audio.
- **Tokens** live in the Windows Credential Manager (via `keyring`), refreshed
  centrally in Rust; the access token is minted on demand for the SDK.

Source map:
- `src-tauri/src/` — Rust host: `auth.rs` (PKCE), `token_store.rs` (keyring),
  `spotify_api.rs` (Web API), `commands.rs` (Tauri command surface), `lib.rs`.
- `src/` — Svelte frontend: `lib/playback.ts` (SDK), `lib/api.ts`, `lib/stores.ts`,
  `views/`, `components/`.
- `benchmarks/` — the efficiency comparison harness (see below).

## Prerequisites

- **Windows 10/11** (the target platform — WebView2 has the best Widevine
  support). Rust 1.85+, Node 18+, and the
  [Tauri prerequisites](https://tauri.app/start/prerequisites/).
- A Spotify **Premium** account.

## Setup

1. **Register a Spotify app** at
   <https://developer.spotify.com/dashboard>. Add this exact redirect URI:

   ```
   http://127.0.0.1:8888/callback
   ```

   Copy the **Client ID** (public — no secret is used with PKCE).

2. **Provide the client id** via environment variable (or bake it at build time
   with the same name):

   ```powershell
   $env:SPOTLITE_CLIENT_ID = "your_client_id"
   ```

3. **Install and run**:

   ```bash
   npm install
   npm run tauri dev      # develop
   npm run tauri build    # produce an MSI/NSIS installer
   ```

## De-risk playback first (Widevine)

Playback depends on the webview negotiating the Widevine CDM. Before relying on
it, open the bundled smoke test — in dev, navigate the webview to
`http://localhost:1420/drm-test.html`. A green ✔ means the Web Playback SDK
route works in your WebView2; a red ✘ means it can't play protected audio there.

## Benchmarks

`benchmarks/` contains a PowerShell harness that compares Spotlite against the
official Spotify client for **memory, CPU, startup time, and disk/installer
size**, summing over each app's whole process tree and reporting median ± std
dev across repeated runs. See [`benchmarks/README.md`](benchmarks/README.md).

```powershell
cd benchmarks
copy config.example.json config.json   # then edit paths
.\run.ps1 -Fresh                        # writes REPORT.md
```

## Legal

Spotlite uses Spotify's official Web API and Web Playback SDK and requires a
Premium account. It is an independent project and is not affiliated with or
endorsed by Spotify. Respect the
[Spotify Developer Terms](https://developer.spotify.com/terms).

## License

MIT
