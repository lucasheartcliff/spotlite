<script lang="ts">
  import { api } from "../lib/api";
  import { showToast } from "../lib/stores";

  let { onLoggedIn }: { onLoggedIn: () => Promise<void> } = $props();

  let busy = $state(false);
  let hasClientId = $state(true);

  api.hasClientId().then((v) => (hasClientId = v));

  async function login() {
    busy = true;
    try {
      await api.login();
      await onLoggedIn();
    } catch (e) {
      showToast(`Login failed: ${e}`);
    } finally {
      busy = false;
    }
  }
</script>

<div class="login">
  <div class="login-card">
    <h1>Spotlite</h1>
    <p class="tagline">A lightweight Spotify desktop client.</p>

    {#if !hasClientId}
      <p class="warn">
        No Spotify client id configured. Set <code>SPOTLITE_CLIENT_ID</code> and
        restart. See the README for setup.
      </p>
    {/if}

    <button class="primary" disabled={busy || !hasClientId} onclick={login}>
      {busy ? "Waiting for Spotify…" : "Log in with Spotify"}
    </button>
    <p class="fineprint">Spotify Premium is required for in-app playback.</p>
  </div>
</div>
