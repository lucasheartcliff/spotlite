<script lang="ts">
  import { api } from "../lib/api";
  import { showToast } from "../lib/stores";
  import TrackList from "../components/TrackList.svelte";
  import type { Track } from "../lib/types";

  let query = $state("");
  let tracks = $state<Track[]>([]);
  let searching = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function onInput() {
    clearTimeout(timer);
    timer = setTimeout(runSearch, 300); // debounce
  }

  async function runSearch() {
    const q = query.trim();
    if (!q) {
      tracks = [];
      return;
    }
    searching = true;
    try {
      const res = await api.search(q, "track", 30);
      tracks = res.tracks?.items ?? [];
    } catch (e) {
      showToast(`Search failed: ${e}`);
    } finally {
      searching = false;
    }
  }
</script>

<div class="view">
  <input
    class="search-input"
    type="text"
    placeholder="What do you want to listen to?"
    bind:value={query}
    oninput={onInput}
  />

  {#if searching}
    <p class="subtle">Searching…</p>
  {:else if tracks.length}
    <TrackList {tracks} />
  {:else if query}
    <p class="subtle">No results.</p>
  {/if}
</div>
