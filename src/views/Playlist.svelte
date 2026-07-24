<script lang="ts">
  import { api } from "../lib/api";
  import { showToast } from "../lib/stores";
  import { playContext } from "../lib/playback";
  import TrackList from "../components/TrackList.svelte";
  import type { Track } from "../lib/types";

  let { id, title }: { id: string; title?: string } = $props();

  let tracks = $state<Track[]>([]);
  let loading = $state(true);
  const contextUri = $derived(`spotify:playlist:${id}`);

  // Re-fetch whenever the selected playlist id changes.
  $effect(() => {
    load(id);
  });

  async function load(playlistId: string) {
    loading = true;
    try {
      const res = await api.playlistTracks(playlistId, 100, 0);
      tracks = (res.items ?? []).map((it: any) => it.track).filter(Boolean);
    } catch (e) {
      showToast(`Could not load playlist: ${e}`);
    } finally {
      loading = false;
    }
  }
</script>

<div class="view">
  <div class="playlist-header">
    <h1>{title ?? "Playlist"}</h1>
    <button class="primary" onclick={() => playContext({ contextUri })}>Play</button>
  </div>

  {#if loading}
    <p class="subtle">Loading…</p>
  {:else}
    <TrackList {tracks} {contextUri} />
  {/if}
</div>
