# Spotlite benchmark orchestrator.
#
# Runs the full comparison suite (size, startup, memory, CPU) for every app in
# config.json and regenerates REPORT.md. Some phases are interactive because
# in-app playback requires a manual Premium sign-in and pressing Play (DRM can't
# be automated headlessly). Both apps remember the login across launches, so you
# only sign in once per app.
#
# Usage:
#   .\run.ps1                 # full suite
#   .\run.ps1 -Fresh          # wipe previous results first
#   .\run.ps1 -Only size,startup
param(
    [switch]$Fresh,
    [string[]]$Only,
    [string]$ConfigPath
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "lib\Common.ps1")

$config = if ($ConfigPath) { Get-Content $ConfigPath -Raw | ConvertFrom-Json } else { Import-BenchConfig }
$resultsDir = Ensure-ResultsDir

function Want([string]$phase) { return (-not $Only) -or ($Only -contains $phase) }

if ($Fresh) {
    Get-ChildItem -Path $resultsDir -Filter *.csv -ErrorAction SilentlyContinue | Remove-Item -Force
    Write-Host "Cleared previous results."
}

Write-Host "== Spotlite benchmark suite ==" -ForegroundColor Cyan
Write-Host ("Cores: {0}  Runs: {1}  Sample window: {2}s" -f `
    [Environment]::ProcessorCount, $config.runs, $config.sampleSeconds)

# --- Size (no launch) ---
if (Want "size") {
    Write-Host "`n-- Disk / installer size --" -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "measure-size.ps1") -ConfigPath $ConfigPath
}

# --- Startup (automated launch/kill loop) ---
if (Want "startup") {
    Write-Host "`n-- Startup time --" -ForegroundColor Yellow
    foreach ($app in $config.apps) {
        & (Join-Path $PSScriptRoot "measure-startup.ps1") -AppName $app.name -ConfigPath $ConfigPath
    }
}

# --- Memory + CPU (interactive: needs login + Play) ---
if (Want "usage") {
    Write-Host "`n-- Memory + CPU (idle and playback) --" -ForegroundColor Yellow
    foreach ($app in $config.apps) {
        Write-Host "`n### $($app.name)" -ForegroundColor Green
        for ($run = 1; $run -le $config.runs; $run++) {
            Write-Host "Run $run of $($config.runs)"

            Get-Process -Name $app.processNames -ErrorAction SilentlyContinue |
                Stop-Process -Force -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 2
            Start-Process -FilePath $app.exe | Out-Null

            Read-Host "  Sign in if needed, make sure playback is PAUSED and the window is at its default size, then press Enter for the IDLE sample"
            Start-Sleep -Seconds $config.settleSeconds
            & (Join-Path $PSScriptRoot "measure-usage.ps1") -AppName $app.name -Phase idle -Run $run -ConfigPath $ConfigPath

            Read-Host "  Now press PLAY on a track and let it play, then press Enter for the PLAYBACK sample"
            Start-Sleep -Seconds $config.settleSeconds
            & (Join-Path $PSScriptRoot "measure-usage.ps1") -AppName $app.name -Phase playback -Run $run -ConfigPath $ConfigPath

            Get-Process -Name $app.processNames -ErrorAction SilentlyContinue |
                Stop-Process -Force -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 2
        }
    }
}

# --- Report ---
Write-Host "`n-- Generating REPORT.md --" -ForegroundColor Yellow
& (Join-Path $PSScriptRoot "report.ps1")
Write-Host "`nDone. See benchmarks\REPORT.md" -ForegroundColor Cyan
