<script lang="ts">
  import { player } from "../lib/stores";
  import {
    togglePlay,
    nextTrack,
    previousTrack,
    seek,
    setVolume,
  } from "../lib/playback";

  let volume = $state(0.8);

  function fmt(ms: number): string {
    const s = Math.floor(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  }

  function onSeek(e: Event) {
    const value = Number((e.target as HTMLInputElement).value);
    seek(value);
  }

  function onVolume(e: Event) {
    volume = Number((e.target as HTMLInputElement).value);
    setVolume(volume);
  }

  const cover = $derived($player.track?.album?.images?.[0]?.url ?? "");
  const artists = $derived(
    $player.track?.artists?.map((a) => a.name).join(", ") ?? "",
  );
</script>

<footer class="nowplaying">
  <div class="np-track">
    {#if $player.track}
      {#if cover}<img src={cover} alt="" class="np-cover" />{/if}
      <div class="np-meta">
        <div class="np-title">{$player.track.name}</div>
        <div class="np-artist">{artists}</div>
      </div>
    {:else}
      <div class="np-meta"><div class="np-artist">Nothing playing</div></div>
    {/if}
  </div>

  <div class="np-controls">
    <div class="np-buttons">
      <button onclick={previousTrack} aria-label="Previous">⏮</button>
      <button class="np-play" onclick={togglePlay} aria-label="Play/Pause">
        {$player.paused ? "▶" : "⏸"}
      </button>
      <button onclick={nextTrack} aria-label="Next">⏭</button>
    </div>
    <div class="np-seek">
      <span>{fmt($player.positionMs)}</span>
      <input
        type="range"
        min="0"
        max={$player.durationMs || 0}
        value={$player.positionMs}
        oninput={onSeek}
        disabled={!$player.track}
      />
      <span>{fmt($player.durationMs)}</span>
    </div>
  </div>

  <div class="np-volume">
    <span aria-hidden="true">🔊</span>
    <input type="range" min="0" max="1" step="0.01" value={volume} oninput={onVolume} />
  </div>
</footer>
