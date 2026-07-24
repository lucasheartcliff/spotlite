<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import { showToast } from "../lib/stores";
  import TrackList from "../components/TrackList.svelte";
  import type { Track } from "../lib/types";

  let tracks = $state<Track[]>([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      const res = await api.savedTracks(50);
      tracks = (res.items ?? []).map((it: any) => it.track).filter(Boolean);
    } catch (e) {
      showToast(`Could not load library: ${e}`);
    } finally {
      loading = false;
    }
  });
</script>

<div class="view">
  <h1>Liked Songs</h1>
  {#if loading}
    <p class="subtle">Loading…</p>
  {:else if tracks.length}
    <TrackList {tracks} />
  {:else}
    <p class="subtle">No liked songs yet.</p>
  {/if}
</div>
