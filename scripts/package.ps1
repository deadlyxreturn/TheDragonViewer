<#
.SYNOPSIS
    Stages the compiled viewer, license notices and install manifest.
.EXAMPLE
    ./scripts/package.ps1 -Exe target/release/dragon-viewer.exe -Out Dragon-Viewer-v0.1.0-windows-x64
#>
param(
    [Parameter(Mandatory)][string]$Exe,
    [Parameter(Mandatory)][string]$Out,
    [string]$Source = (Split-Path $PSScriptRoot -Parent)
)
$ErrorActionPreference = 'Stop'
$exePath = (Resolve-Path -LiteralPath $Exe).Path
$reported = (& $exePath --version | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $reported -cnotmatch '^Dragon Viewer (\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)$') {
    throw "Invalid package version: '$reported'."
}
$version = $Matches[1]
$windowsPackage = [IO.Path]::GetExtension($exePath) -ieq '.exe'
$exeName = if ($windowsPackage) { 'dragon-viewer.exe' } else { Split-Path $exePath -Leaf }

$viewerAssets = Join-Path $Source 'crates/prototype-viewer/assets'
$notices = @{
    emoji = (Join-Path $Source 'assets/emoji/LICENSE-OFL.txt')
    fonts = @(Get-ChildItem -LiteralPath (Join-Path $viewerAssets 'fonts') -File -Filter 'LICENSE-*.txt' |
        Select-Object -ExpandProperty FullName)
    icons = (Join-Path $viewerAssets 'icons/LICENSE-tabler.txt')
    lsl = (Join-Path $viewerAssets 'lsl/PREPROCESSOR-LICENSE.txt')
    stars = (Join-Path $Source 'assets/stars/LICENSE-TYCHO2.md')
}
if ($windowsPackage) { $notices.branding = Join-Path $Source 'assets/branding/dragon-viewer.ico' }
foreach ($group in $notices.Keys) {
    $directory = Join-Path $Out "assets/$group"
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    Copy-Item -LiteralPath $notices[$group] -Destination $directory
}

# The dictionary contents are embedded. Ship their notices and source list.
$dictionaries = Join-Path $viewerAssets 'dictionaries'
$dictionariesOut = Join-Path $Out 'assets/dictionaries'
New-Item -ItemType Directory -Force -Path $dictionariesOut | Out-Null
Copy-Item -LiteralPath (Join-Path $dictionaries 'README.md') -Destination $dictionariesOut
Get-ChildItem -LiteralPath $dictionaries -Directory | ForEach-Object {
    $license = Join-Path $_.FullName 'LICENSE.txt'
    if (Test-Path -LiteralPath $license) {
        Copy-Item -LiteralPath $license -Destination (Join-Path $dictionariesOut "$($_.Name)-LICENSE.txt")
    }
}

Copy-Item -LiteralPath $exePath -Destination (Join-Path $Out $exeName)
$docs = 'LICENSE', 'NOTICE.md', 'README.md', 'PRIVACY.md' | ForEach-Object { Join-Path $Source $_ }
Copy-Item -LiteralPath $docs -Destination $Out
# Preserve the inherited Linux desktop metadata and macOS bundle manifest.
if (-not [IO.Path]::GetExtension($exePath) -and -not $IsMacOS) {
    Copy-Item -LiteralPath (Join-Path $Source 'packaging/linux/prototype-viewer.desktop') -Destination $Out
    Copy-Item -LiteralPath (Join-Path $Source 'assets/branding/prototype-logo.svg') -Destination (Join-Path $Out 'prototype-viewer.svg')
}
$manifest = if ($IsMacOS) { 'prototype-install.json' } else { 'dragon-viewer-install.json' }
$manifestPath = Join-Path (Resolve-Path -LiteralPath $Out).Path $manifest
$json = @{ schema = 1; version = $version } | ConvertTo-Json
[IO.File]::WriteAllText($manifestPath, $json, [Text.UTF8Encoding]::new($false))
