<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { api } from "./lib/api";
  import { authenticated, me, toast, route } from "./lib/stores";
  import {
    initPlayback,
    togglePlay,
    nextTrack,
    previousTrack,
  } from "./lib/playback";
  import Login from "./views/Login.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import NowPlaying from "./components/NowPlaying.svelte";
  import Home from "./views/Home.svelte";
  import Search from "./views/Search.svelte";
  import Library from "./views/Library.svelte";
  import Playlist from "./views/Playlist.svelte";

  let booting = $state(true);

  onMount(async () => {
    // Tell the host the UI is interactive (startup benchmark + reveal window).
    api.markReady().catch(() => {});

    // OS media keys forwarded from the Rust host.
    listen<string>("media-key", (e) => {
      if (e.payload === "playpause") togglePlay();
      else if (e.payload === "next") nextTrack();
      else if (e.payload === "previous") previousTrack();
    });

    try {
      if (await api.isAuthenticated()) {
        await afterLogin();
      }
    } finally {
      booting = false;
    }
  });

  async function afterLogin() {
    const profile = await api.me();
    me.set(profile);
    authenticated.set(true);
    // Only Premium accounts can drive the Web Playback SDK.
    if (profile.product === "premium") {
      await initPlayback();
    }
  }

  // Re-run playback bootstrap when the Login view reports success.
  async function handleLoggedIn() {
    await afterLogin();
  }
</script>

<main class="app">
  {#if booting}
    <div class="splash">Loading Spotlite…</div>
  {:else if !$authenticated}
    <Login onLoggedIn={handleLoggedIn} />
  {:else}
    <div class="shell">
      <Sidebar />
      <section class="content">
        {#if $route.name === "home"}
          <Home />
        {:else if $route.name === "search"}
          <Search />
        {:else if $route.name === "library"}
          <Library />
        {:else if $route.name === "playlist"}
          <Playlist id={$route.id} title={$route.title} />
        {/if}
      </section>
      <NowPlaying />
    </div>
  {/if}

  {#if $toast}
    <div class="toast">{$toast}</div>
  {/if}
</main>
