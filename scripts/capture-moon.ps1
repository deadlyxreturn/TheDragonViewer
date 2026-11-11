[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Exe,
    [Parameter(Mandatory)][string]$Out,
    [ValidateSet('Login', 'Preferences', 'Messenger')][string]$Scene = 'Login',
    [int]$Width = 1672,
    [int]$Height = 941
)
$ErrorActionPreference = 'Stop'
$exePath = (Resolve-Path -LiteralPath $Exe).Path
$output = (Resolve-Path -LiteralPath $Out).Path
$sceneName = $Scene.ToLowerInvariant()
$image = Join-Path $output "dragon-viewer-$sceneName.png"
$env:PROTOTYPE_DEMO = '1'
$env:PROTOTYPE_DEMO_POINTER = "$($Width - 32),$($Height - 36)"
$env:PROTOTYPE_WINDOW_SIZE = "$Width,$Height"
$env:PROTOTYPE_CAPTURE = $image
$env:PROTOTYPE_CAPTURE_FRAMES = '100'
$env:PROTOTYPE_CAPTURE_EXIT = '1'
$env:PROTOTYPE_DEMO_LOGIN = if ($Scene -eq 'Messenger') { $null } else { 'first' }
$env:PROTOTYPE_DEMO_OPTIONS = if ($Scene -eq 'Preferences') { 'sound/0' } else { $null }
$env:PROTOTYPE_DEMO_CONV = if ($Scene -eq 'Messenger') { 'prototype:im:1060' } else { $null }
$process = Start-Process -FilePath $exePath -ArgumentList '--language en', "--title Moon-$Scene-Capture", '--fps-limit 60' -WorkingDirectory (Split-Path $exePath) -WindowStyle Hidden -PassThru
if (-not $process.WaitForExit(150000)) {
    $process.Kill()
    throw "Dragon Viewer timed out during the $Scene capture."
}
if ($process.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $image -PathType Leaf)) {
    throw "Dragon Viewer $Scene capture failed (exit $($process.ExitCode))."
}
Get-Item -LiteralPath $image | Select-Object FullName, Length
