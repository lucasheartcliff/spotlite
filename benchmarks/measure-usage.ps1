# Samples memory + CPU across an app's whole process tree over a fixed window,
# for a single labeled phase ("idle" or "playback"). The app must already be
# running, logged in, and in the requested state. Appends to results/usage.csv.
#
# CPU% is derived from the delta of summed TotalProcessorTime per interval,
# normalized by logical core count so 100% == one fully-used core-equivalent
# share of the whole machine (0..100 across all cores).

param(
    [Parameter(Mandatory)][string]$AppName,
    [Parameter(Mandatory)][ValidateSet("idle", "playback")][string]$Phase,
    [int]$Run = 1,
    [string]$ConfigPath
)

. (Join-Path $PSScriptRoot "lib\Common.ps1")

$config = if ($ConfigPath) { Get-Content $ConfigPath -Raw | ConvertFrom-Json } else { Import-BenchConfig }
$app = $config.apps | Where-Object { $_.name -eq $AppName }
if (-not $app) { throw "App '$AppName' not found in config." }

$rootPid = Get-PidsByName -Names $app.processNames
if (-not $rootPid) { throw "$AppName is not running. Launch and sign in first." }

$cores = [Environment]::ProcessorCount
$intervalMs = $config.sampleIntervalMs
$samples = [int]([math]::Ceiling($config.sampleSeconds * 1000 / $intervalMs))

Write-Host ("Sampling {0} [{1}] for {2}s ({3} samples)..." -f $AppName, $Phase, $config.sampleSeconds, $samples)

$wsSeries = @()
$privSeries = @()
$cpuSeries = @()

$pids = @(Get-DescendantPids -RootPid $rootPid)
$prevCpu = Get-TreeCpuSeconds -Pids $pids
$prevTime = [System.Diagnostics.Stopwatch]::StartNew()

for ($i = 0; $i -lt $samples; $i++) {
    Start-Sleep -Milliseconds $intervalMs
    # Re-resolve the tree each sample so new renderer/helper processes are caught.
    $pids = @(Get-DescendantPids -RootPid $rootPid)

    $mem = Get-TreeMemory -Pids $pids
    $wsSeries += $mem.WorkingSetMB
    $privSeries += $mem.PrivateMB

    $nowCpu = Get-TreeCpuSeconds -Pids $pids
    $elapsedSec = $prevTime.Elapsed.TotalSeconds
    $prevTime.Restart()
    $cpuPct = if ($elapsedSec -gt 0) { (($nowCpu - $prevCpu) / $elapsedSec) / $cores * 100 } else { 0 }
    if ($cpuPct -lt 0) { $cpuPct = 0 }  # process exited between samples
    $cpuSeries += [math]::Round($cpuPct, 1)
    $prevCpu = $nowCpu
}

$row = [pscustomobject]@{
    App             = $AppName
    Phase           = $Phase
    Run             = $Run
    WorkingSetMedMB = [math]::Round((Get-Median -Values ([double[]]$wsSeries)), 1)
    WorkingSetMaxMB = [math]::Round(($wsSeries | Measure-Object -Maximum).Maximum, 1)
    PrivateMedMB    = [math]::Round((Get-Median -Values ([double[]]$privSeries)), 1)
    CpuAvgPct       = [math]::Round(($cpuSeries | Measure-Object -Average).Average, 1)
    CpuMaxPct       = [math]::Round(($cpuSeries | Measure-Object -Maximum).Maximum, 1)
}

Write-Host ("  WS median {0} MB, private {1} MB, CPU avg {2}%" -f $row.WorkingSetMedMB, $row.PrivateMedMB, $row.CpuAvgPct)

$results = Ensure-ResultsDir
$row | Export-Csv -Path (Join-Path $results "usage.csv") -NoTypeInformation -Append
