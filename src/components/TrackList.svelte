<script lang="ts">
  import { playContext } from "../lib/playback";
  import type { Track } from "../lib/types";

  let { tracks, contextUri }: { tracks: Track[]; contextUri?: string } = $props();

  function fmt(ms: number): string {
    const s = Math.floor(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  }

  function playAt(index: number) {
    if (contextUri) {
      playContext({ contextUri, offsetPosition: index });
    } else {
      // No album/playlist context — play this track and the rest as a uri list.
      playContext({ uris: tracks.slice(index).map((t) => t.uri) });
    }
  }
</script>

<table class="tracklist">
  <thead>
    <tr>
      <th class="num">#</th>
      <th>Title</th>
      <th>Album</th>
      <th class="dur">Time</th>
    </tr>
  </thead>
  <tbody>
    {#each tracks as track, i (track.id + i)}
      <tr ondblclick={() => playAt(i)}>
        <td class="num">
          <button class="play-cell" onclick={() => playAt(i)} aria-label="Play">▶</button>
          <span class="idx">{i + 1}</span>
        </td>
        <td>
          <div class="title">{track.name}</div>
          <div class="artist">{track.artists?.map((a) => a.name).join(", ")}</div>
        </td>
        <td class="album">{track.album?.name ?? ""}</td>
        <td class="dur">{fmt(track.duration_ms)}</td>
      </tr>
    {/each}
  </tbody>
</table>
