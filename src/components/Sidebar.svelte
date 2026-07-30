<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import { route, me, authenticated } from "../lib/stores";
  import type { Playlist } from "../lib/types";

  let playlists = $state<Playlist[]>([]);

  onMount(async () => {
    try {
      const res = await api.myPlaylists(50);
      playlists = res.items ?? [];
    } catch {
      // Non-fatal: sidebar just stays empty.
    }
  });

  async function logout() {
    await api.logout();
    authenticated.set(false);
    me.set(null);
  }
</script>

<nav class="sidebar">
  <div class="brand">Spotlite</div>

  <ul class="nav">
    <li>
      <button class:active={$route.name === "home"} onclick={() => route.set({ name: "home" })}>
        Home
      </button>
    </li>
    <li>
      <button class:active={$route.name === "search"} onclick={() => route.set({ name: "search" })}>
        Search
      </button>
    </li>
    <li>
      <button class:active={$route.name === "library"} onclick={() => route.set({ name: "library" })}>
        Your Library
      </button>
    </li>
  </ul>

  <div class="playlists">
    <div class="section-label">Playlists</div>
    <ul>
      {#each playlists as pl (pl.id)}
        <li>
          <button
            class:active={$route.name === "playlist" && $route.id === pl.id}
            onclick={() => route.set({ name: "playlist", id: pl.id, title: pl.name })}
          >
            {pl.name}
          </button>
        </li>
      {/each}
    </ul>
  </div>

  {#if $me}
    <div class="account">
      <span class="account-name">{$me.display_name}</span>
      <button class="link" onclick={logout}>Log out</button>
    </div>
  {/if}
</nav>
