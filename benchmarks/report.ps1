# Aggregates results/*.csv into a human-readable REPORT.md with median + std dev
# per app and metric, plus a "lighter by" comparison (assumes the first app in
# config is the challenger, i.e. Spotlite).

param([string]$ConfigPath)

. (Join-Path $PSScriptRoot "lib\Common.ps1")

$config = if ($ConfigPath) { Get-Content $ConfigPath -Raw | ConvertFrom-Json } else { Import-BenchConfig }
$results = Ensure-ResultsDir
$appNames = $config.apps | ForEach-Object { $_.name }
$challenger = $appNames[0]
$baseline = if ($appNames.Count -gt 1) { $appNames[1] } else { $null }

function Read-CsvSafe($name) {
    $path = Join-Path $results $name
    if (Test-Path $path) { return @(Import-Csv $path) }
    return @()
}

function Fmt($median, $std) {
    if ($null -eq $median) { return "n/a" }
    return ("{0} ± {1}" -f [math]::Round($median, 1), [math]::Round($std, 1))
}

function Ratio($challengerVal, $baselineVal) {
    if (-not $baselineVal -or $baselineVal -eq 0 -or $null -eq $challengerVal) { return "" }
    $pct = [math]::Round((1 - ($challengerVal / $baselineVal)) * 100, 0)
    if ($pct -gt 0) { return "**$pct% lighter**" }
    return "$([math]::Abs($pct))% heavier"
}

$sb = [System.Text.StringBuilder]::new()
[void]$sb.AppendLine("# Spotlite vs Spotify — Efficiency Benchmark")
[void]$sb.AppendLine()
[void]$sb.AppendLine("Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm')  ")
[void]$sb.AppendLine("Machine: $([Environment]::MachineName), $([Environment]::ProcessorCount) logical cores, Windows $([Environment]::OSVersion.Version)  ")
[void]$sb.AppendLine("Runs per metric: $($config.runs); sample window: $($config.sampleSeconds)s.")
[void]$sb.AppendLine()
[void]$sb.AppendLine("Values are **median ± sample std dev** across runs. Memory and CPU are summed over each app's **entire process tree** (Spotify helpers; Spotlite's msedgewebview2 children).")
[void]$sb.AppendLine()

# --- Memory + CPU ---
$usage = Read-CsvSafe "usage.csv"
if ($usage.Count) {
    foreach ($phase in @("idle", "playback")) {
        [void]$sb.AppendLine("## Memory & CPU — $phase")
        [void]$sb.AppendLine()
        [void]$sb.AppendLine("| App | Working set (MB) | Private (MB) | CPU (%) |")
        [void]$sb.AppendLine("|-----|------------------|--------------|---------|")
        $medWsByApp = @{}
        foreach ($appName in $appNames) {
            $rows = $usage | Where-Object { $_.App -eq $appName -and $_.Phase -eq $phase }
            if (-not $rows) {
                [void]$sb.AppendLine("| $appName | n/a | n/a | n/a |")
                continue
            }
            $ws = [double[]]($rows | ForEach-Object { [double]$_.WorkingSetMedMB })
            $priv = [double[]]($rows | ForEach-Object { [double]$_.PrivateMedMB })
            $cpu = [double[]]($rows | ForEach-Object { [double]$_.CpuAvgPct })
            $medWs = Get-Median -Values $ws
            $medWsByApp[$appName] = $medWs
            [void]$sb.AppendLine("| $appName | $(Fmt $medWs (Get-StdDev $ws)) | $(Fmt (Get-Median $priv) (Get-StdDev $priv)) | $(Fmt (Get-Median $cpu) (Get-StdDev $cpu)) |")
        }
        if ($baseline -and $medWsByApp.ContainsKey($challenger) -and $medWsByApp.ContainsKey($baseline)) {
            [void]$sb.AppendLine()
            [void]$sb.AppendLine("Working set: $challenger is $(Ratio $medWsByApp[$challenger] $medWsByApp[$baseline]) than $baseline.")
        }
        [void]$sb.AppendLine()
    }
}

# --- Startup ---
$startup = Read-CsvSafe "startup.csv"
if ($startup.Count) {
    [void]$sb.AppendLine("## Startup time (launch → main window visible)")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("| App | Startup (ms) |")
    [void]$sb.AppendLine("|-----|--------------|")
    $medByApp = @{}
    foreach ($appName in $appNames) {
        $vals = [double[]]($startup | Where-Object { $_.App -eq $appName } | ForEach-Object { [double]$_.StartupMs })
        if (-not $vals.Count) { [void]$sb.AppendLine("| $appName | n/a |"); continue }
        $med = Get-Median -Values $vals
        $medByApp[$appName] = $med
        [void]$sb.AppendLine("| $appName | $(Fmt $med (Get-StdDev $vals)) |")
    }
    if ($baseline -and $medByApp.ContainsKey($challenger) -and $medByApp.ContainsKey($baseline)) {
        [void]$sb.AppendLine()
        [void]$sb.AppendLine("$challenger starts $(Ratio $medByApp[$challenger] $medByApp[$baseline]) (faster) than $baseline.")
    }
    [void]$sb.AppendLine()
}

# --- Size ---
$size = Read-CsvSafe "size.csv"
if ($size.Count) {
    [void]$sb.AppendLine("## Disk & installer size")
    [void]$sb.AppendLine()
    [void]$sb.AppendLine("| App | Installed (MB) | Installer (MB) |")
    [void]$sb.AppendLine("|-----|----------------|----------------|")
    foreach ($appName in $appNames) {
        $r = $size | Where-Object { $_.App -eq $appName } | Select-Object -First 1
        if ($r) { [void]$sb.AppendLine("| $appName | $($r.InstalledMB) | $($r.InstallerMB) |") }
        else { [void]$sb.AppendLine("| $appName | n/a | n/a |") }
    }
    $c = $size | Where-Object { $_.App -eq $challenger } | Select-Object -First 1
    $b = if ($baseline) { $size | Where-Object { $_.App -eq $baseline } | Select-Object -First 1 } else { $null }
    if ($c -and $b -and $b.InstalledMB) {
        [void]$sb.AppendLine()
        [void]$sb.AppendLine("Installed footprint: $challenger is $(Ratio ([double]$c.InstalledMB) ([double]$b.InstalledMB)) than $baseline.")
    }
    [void]$sb.AppendLine()
}

[void]$sb.AppendLine("---")
[void]$sb.AppendLine("_Methodology: see benchmarks/README.md. Numbers depend on machine, network, and content; treat them as a comparison on one machine, not absolute figures._")

$reportPath = Join-Path $PSScriptRoot "REPORT.md"
$sb.ToString() | Set-Content -Path $reportPath -Encoding UTF8
Write-Host "Wrote $reportPath"
