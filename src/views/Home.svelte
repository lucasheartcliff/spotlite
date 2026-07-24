<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import { route } from "../lib/stores";
  import type { Playlist } from "../lib/types";

  let playlists = $state<Playlist[]>([]);

  onMount(async () => {
    try {
      const res = await api.myPlaylists(24);
      playlists = res.items ?? [];
    } catch {
      // handled by empty state
    }
  });
</script>

<div class="view">
  <h1>Good to see you</h1>
  <p class="subtle">Jump back into your playlists.</p>

  <div class="grid">
    {#each playlists as pl (pl.id)}
      <button
        class="card"
        onclick={() => route.set({ name: "playlist", id: pl.id, title: pl.name })}
      >
        {#if pl.images?.[0]?.url}
          <img src={pl.images[0].url} alt="" />
        {:else}
          <div class="art-placeholder"></div>
        {/if}
        <div class="card-title">{pl.name}</div>
        <div class="card-sub">{pl.tracks?.total ?? 0} tracks</div>
      </button>
    {/each}
  </div>
</div>
