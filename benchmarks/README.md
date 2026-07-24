# Spotlite benchmarks

The **final validation** for Spotlite: a Windows PowerShell harness that measures
Spotlite against the official Spotify desktop client on the same machine, across
four axes:

| Metric | What it captures |
|--------|------------------|
| **Memory** | Working set + private bytes, summed over the whole process tree, idle and during playback |
| **CPU** | % of total machine CPU, idle and during playback |
| **Startup** | Launch → main window visible |
| **Size** | Installed directory size + installer download size |

## Why a process *tree*

Neither app is a single process. Spotify spawns several `Spotify.exe`
renderer/helper processes; Spotlite spawns the host `Spotlite.exe` plus one or
more `msedgewebview2.exe` children. Comparing only the main process would be
misleading, so the harness walks the real parent/child tree from the launched
PID (`Get-DescendantPids` in `lib/Common.ps1`) and sums across it.

## Fairness rules baked into the harness

- **Whole-tree** memory/CPU, re-resolved every sample so new helpers are caught.
- **Matched state**: same Premium account, default window size, same track, both
  measured *idle* (logged in, paused) and *during playback*.
- **Repeated runs** (`runs` in config, default 5) with **median + sample std dev**
  reported, not single-shot numbers.
- **Same startup signal** for both apps (launch → first visible main window).

## Prerequisites

- Windows 10/11 with both apps installed and signed into the **same Spotify
  Premium** account.
- Spotlite built and installed (`npm run tauri build`, then run the installer),
  so it appears as its own process tree.
- PowerShell 5.1+ (ships with Windows) or PowerShell 7.

## Running

```powershell
cd benchmarks
copy config.example.json config.json
# edit config.json: set each app's exe, installDir, processNames, installer path

# full suite (size + startup automated; memory/CPU interactive)
.\run.ps1 -Fresh

# or individual phases
.\run.ps1 -Only size,startup
.\measure-usage.ps1 -AppName Spotlite -Phase idle
```

The memory/CPU phase is interactive on purpose: in-app playback needs a manual
Premium sign-in and pressing Play (DRM can't be driven headlessly). Both apps
remember the login between launches, so you sign in once per app.

Raw samples land in `results/*.csv`; `report.ps1` aggregates them into
`REPORT.md`.

## Notes

- CPU% is normalized by logical core count: 100% = one full core-equivalent of
  the whole machine.
- Numbers depend on machine, network, and the specific track/playlist. Treat the
  report as a same-machine comparison, not absolute figures.
- For a precise Spotlite-only startup number you can run a **debug** build and
  read the `SPOTLITE_READY_MS=<ms>` line it prints (emitted by the `mark_ready`
  command when the UI mounts).
