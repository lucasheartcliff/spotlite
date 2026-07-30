# Measures installed footprint and installer download size for each app.
# Writes results/size.csv. No app launch required.

param([string]$ConfigPath)

. (Join-Path $PSScriptRoot "lib\Common.ps1")

$config = if ($ConfigPath) { Get-Content $ConfigPath -Raw | ConvertFrom-Json } else { Import-BenchConfig }
$results = Ensure-ResultsDir
$rows = @()

foreach ($app in $config.apps) {
    $installMB = Get-DirSizeMB -Path $app.installDir
    $installerMB = $null
    if ($app.installer) {
        $installerPath = $app.installer
        if (-not [System.IO.Path]::IsPathRooted($installerPath)) {
            $installerPath = Join-Path $PSScriptRoot $installerPath
        }
        if (Test-Path $installerPath) {
            $installerMB = [math]::Round((Get-Item $installerPath).Length / 1MB, 1)
        }
    }
    Write-Host ("{0}: installed {1} MB, installer {2} MB" -f $app.name, $installMB, $installerMB)
    $rows += [pscustomobject]@{
        App          = $app.name
        InstalledMB  = $installMB
        InstallerMB  = $installerMB
    }
}

$rows | Export-Csv -Path (Join-Path $results "size.csv") -NoTypeInformation
Write-Host "Wrote results\size.csv"
