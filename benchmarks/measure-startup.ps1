# Measures cold/warm startup time (launch -> main window visible) for one app,
# repeated `runs` times. Appends rows to results/startup.csv.
#
# The same app-agnostic signal (Wait-ForMainWindow) is used for both apps so the
# comparison is fair. For a precise Spotlite-only figure you can additionally run
# a debug build and read the `SPOTLITE_READY_MS=` line it prints to stdout.

param(
    [Parameter(Mandatory)][string]$AppName,
    [string]$ConfigPath
)

. (Join-Path $PSScriptRoot "lib\Common.ps1")

$config = if ($ConfigPath) { Get-Content $ConfigPath -Raw | ConvertFrom-Json } else { Import-BenchConfig }
$app = $config.apps | Where-Object { $_.name -eq $AppName }
if (-not $app) { throw "App '$AppName' not found in config." }

$results = Ensure-ResultsDir
$csv = Join-Path $results "startup.csv"
$rows = @()

for ($i = 1; $i -le $config.runs; $i++) {
    # Make sure nothing from a previous run is lingering.
    Get-Process -Name $app.processNames -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2

    $proc = Start-Process -FilePath $app.exe -PassThru
    $elapsed = Wait-ForMainWindow -RootPid $proc.Id -TimeoutMs 60000

    if ($null -eq $elapsed) {
        Write-Warning "$AppName run ${i}: window never appeared within timeout."
    } else {
        Write-Host ("{0} startup run {1}: {2} ms" -f $AppName, $i, $elapsed)
        $rows += [pscustomobject]@{ App = $AppName; Run = $i; StartupMs = $elapsed }
    }

    # Close the app before the next run.
    Get-Process -Name $app.processNames -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}

$writeHeader = -not (Test-Path $csv)
$rows | Export-Csv -Path $csv -NoTypeInformation -Append
Write-Host "Appended $($rows.Count) rows to results\startup.csv"
