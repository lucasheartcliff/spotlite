/// <reference types="svelte" />
/// <reference types="vite/client" />

// The Spotify Web Playback SDK attaches a global. Minimal typings for what we use.
interface Window {
  onSpotifyWebPlaybackSDKReady: () => void;
  Spotify: typeof Spotify;
}

declare namespace Spotify {
  interface PlayerInit {
    name: string;
    getOAuthToken: (cb: (token: string) => void) => void;
    volume?: number;
  }
  interface WebPlaybackError {
    message: string;
  }
  interface Player {
    connect(): Promise<boolean>;
    disconnect(): void;
    addListener(event: string, cb: (arg: any) => void): boolean;
    removeListener(event: string): boolean;
    getCurrentState(): Promise<any>;
    setName(name: string): Promise<void>;
    getVolume(): Promise<number>;
    setVolume(volume: number): Promise<void>;
    pause(): Promise<void>;
    resume(): Promise<void>;
    togglePlay(): Promise<void>;
    seek(positionMs: number): Promise<void>;
    previousTrack(): Promise<void>;
    nextTrack(): Promise<void>;
  }
  const Player: {
    new (init: PlayerInit): Player;
  };
}
