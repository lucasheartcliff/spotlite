// Minimal shapes for the Spotify Web API responses we actually touch. These are
// intentionally partial — the API returns far more than the UI renders.

export interface Image {
  url: string;
  width?: number;
  height?: number;
}

export interface Artist {
  id: string;
  name: string;
  uri: string;
}

export interface Album {
  id: string;
  name: string;
  uri: string;
  images: Image[];
}

export interface Track {
  id: string;
  name: string;
  uri: string;
  duration_ms: number;
  artists: Artist[];
  album: Album;
}

export interface Playlist {
  id: string;
  name: string;
  uri: string;
  description?: string;
  images: Image[];
  owner?: { display_name?: string };
  tracks?: { total: number };
}

export interface Me {
  id: string;
  display_name: string;
  product: string; // "premium" required for the Web Playback SDK
  images: Image[];
}

/** Player state as surfaced to the UI (normalized from the SDK). */
export interface PlayerState {
  ready: boolean;
  deviceId: string | null;
  paused: boolean;
  positionMs: number;
  durationMs: number;
  track: Track | null;
}
