# Shared helpers for the Spotlite benchmark harness.
#
# Fairness principle: each app is measured across its ENTIRE process tree, not a
# single process. Spotify spawns multiple Spotify.exe helper/renderer processes;
# Spotlite spawns the host Spotlite.exe plus msedgewebview2.exe children. We walk
# the real parent/child tree from the launched root PID so the comparison is
# apples-to-apples.

Set-StrictMode -Version Latest

function Get-DescendantPids {
    <#
        Returns the given root PID plus all descendant PIDs, walking
        Win32_Process.ParentProcessId. This captures WebView2 / renderer helpers
        regardless of their executable name.
    #>
    param([Parameter(Mandatory)][int]$RootPid)

    $all = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
        Select-Object ProcessId, ParentProcessId
    $childrenByParent = @{}
    foreach ($p in $all) {
        if (-not $childrenByParent.ContainsKey($p.ParentProcessId)) {
            $childrenByParent[$p.ParentProcessId] = @()
        }
        $childrenByParent[$p.ParentProcessId] += $p.ProcessId
    }

    $result = New-Object System.Collections.Generic.HashSet[int]
    $stack = New-Object System.Collections.Stack
    [void]$stack.Push($RootPid)
    while ($stack.Count -gt 0) {
        $current = $stack.Pop()
        if ($result.Add($current) -and $childrenByParent.ContainsKey($current)) {
            foreach ($child in $childrenByParent[$current]) { [void]$stack.Push($child) }
        }
    }
    return $result
}

function Get-PidsByName {
    <# Fallback: resolve a root PID from a set of process names (e.g. when
       attaching to an already-running app). Picks the oldest matching process
       as the root. #>
    param([Parameter(Mandatory)][string[]]$Names)
    $procs = Get-Process -Name $Names -ErrorAction SilentlyContinue |
        Sort-Object StartTime
    if (-not $procs) { return $null }
    return $procs[0].Id
}

function Get-TreeMemory {
    <# Sum WorkingSet64 and PrivateMemorySize64 (bytes) over a set of PIDs. #>
    param([Parameter(Mandatory)][int[]]$Pids)
    $ws = 0L; $priv = 0L
    foreach ($id in $Pids) {
        $p = Get-Process -Id $id -ErrorAction SilentlyContinue
        if ($p) { $ws += $p.WorkingSet64; $priv += $p.PrivateMemorySize64 }
    }
    [pscustomobject]@{ WorkingSetMB = [math]::Round($ws / 1MB, 1); PrivateMB = [math]::Round($priv / 1MB, 1) }
}

function Get-TreeCpuSeconds {
    <# Sum TotalProcessorTime (seconds of CPU consumed) over a set of PIDs. #>
    param([Parameter(Mandatory)][int[]]$Pids)
    $total = 0.0
    foreach ($id in $Pids) {
        $p = Get-Process -Id $id -ErrorAction SilentlyContinue
        if ($p) { $total += $p.TotalProcessorTime.TotalSeconds }
    }
    return $total
}

function Get-Median {
    param([Parameter(Mandatory)][double[]]$Values)
    if ($Values.Count -eq 0) { return 0 }
    $sorted = $Values | Sort-Object
    $mid = [int][math]::Floor($sorted.Count / 2)
    if ($sorted.Count % 2 -eq 1) { return $sorted[$mid] }
    return ($sorted[$mid - 1] + $sorted[$mid]) / 2
}

function Get-StdDev {
    param([Parameter(Mandatory)][double[]]$Values)
    if ($Values.Count -lt 2) { return 0 }
    $mean = ($Values | Measure-Object -Average).Average
    $sumSq = 0.0
    foreach ($v in $Values) { $sumSq += [math]::Pow($v - $mean, 2) }
    return [math]::Sqrt($sumSq / ($Values.Count - 1))  # sample std dev
}

function Get-DirSizeMB {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path $Path)) { return $null }
    $bytes = (Get-ChildItem -Path $Path -Recurse -File -ErrorAction SilentlyContinue |
        Measure-Object -Property Length -Sum).Sum
    return [math]::Round($bytes / 1MB, 1)
}

function Wait-ForMainWindow {
    <# Poll until the process (or its tree) shows a visible main window, or the
       timeout elapses. Returns elapsed milliseconds, or $null on timeout. This
       is the app-agnostic "launch -> window visible" startup signal. #>
    param(
        [Parameter(Mandatory)][int]$RootPid,
        [int]$TimeoutMs = 30000
    )
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $TimeoutMs) {
        $pids = Get-DescendantPids -RootPid $RootPid
        foreach ($id in $pids) {
            $p = Get-Process -Id $id -ErrorAction SilentlyContinue
            if ($p -and $p.MainWindowHandle -ne 0) {
                $sw.Stop()
                return $sw.ElapsedMilliseconds
            }
        }
        Start-Sleep -Milliseconds 50
    }
    $sw.Stop()
    return $null
}

function Import-BenchConfig {
    param([string]$Path = (Join-Path $PSScriptRoot "..\config.json"))
    if (-not (Test-Path $Path)) {
        throw "Config not found at $Path. Copy config.example.json to config.json and fill in the app paths."
    }
    return Get-Content $Path -Raw | ConvertFrom-Json
}

function Ensure-ResultsDir {
    $dir = Join-Path $PSScriptRoot "..\results"
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
    return (Resolve-Path $dir).Path
}
