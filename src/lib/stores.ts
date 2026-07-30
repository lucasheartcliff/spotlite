import { writable } from "svelte/store";
import type { Me, PlayerState } from "./types";

export type Route =
  | { name: "home" }
  | { name: "search" }
  | { name: "library" }
  | { name: "playlist"; id: string; title?: string };

export const route = writable<Route>({ name: "home" });

export const me = writable<Me | null>(null);

export const authenticated = writable<boolean>(false);

export const player = writable<PlayerState>({
  ready: false,
  deviceId: null,
  paused: true,
  positionMs: 0,
  durationMs: 0,
  track: null,
});

/** Transient error banner text. */
export const toast = writable<string | null>(null);

export function showToast(message: string, ms = 4000) {
  toast.set(message);
  setTimeout(() => toast.set(null), ms);
}
