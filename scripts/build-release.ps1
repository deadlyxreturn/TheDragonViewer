<#
.SYNOPSIS
    Builds a local Windows x64 release and writes its unsigned installer,
    portable update archive, update policy and SHA-256 checksums.
.EXAMPLE
    ./scripts/build-release.ps1 -Tag v0.1.0 -Makensis 'C:\Program Files (x86)\NSIS\makensis.exe'
.EXAMPLE
    ./scripts/build-release.ps1 -SkipBuild -Exe target/release/dragon-viewer.exe -Out C:\builds\dragon-viewer-0.1.0
#>
[CmdletBinding(DefaultParameterSetName = 'Build')]
param(
    [ValidatePattern('^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$')][string]$Tag = 'v0.1.0',
    [string]$Out,
    [string]$Makensis = 'makensis.exe',
    [Parameter(Mandatory, ParameterSetName = 'Package')][switch]$SkipBuild,
    [Parameter(Mandatory, ParameterSetName = 'Package')][string]$Exe
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$compiler = (Get-Command -Name $Makensis -CommandType Application | Select-Object -First 1).Source
if (-not $Out) { $Out = Join-Path $repo "target/dragon-viewer-dist/$Tag" }
$dist = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Out)
if (Test-Path -LiteralPath $dist) { throw "The release output path already exists: $dist" }

if (-not $SkipBuild) {
    $oldTag = [Environment]::GetEnvironmentVariable('PROTOTYPE_VERSION_TAG', 'Process')
    Push-Location -LiteralPath $repo
    try {
        $env:PROTOTYPE_VERSION_TAG = $Tag
        cargo build --release --locked --target x86_64-pc-windows-msvc --target-dir target -p prototype-viewer --bin dragon-viewer --features xr-static-loader
        if ($LASTEXITCODE -ne 0) { throw "Cargo failed with exit code $LASTEXITCODE." }
    } finally {
        [Environment]::SetEnvironmentVariable('PROTOTYPE_VERSION_TAG', $oldTag, 'Process')
        Pop-Location
    }
    $Exe = Join-Path $repo 'target/x86_64-pc-windows-msvc/release/dragon-viewer.exe'
}
$exePath = (Resolve-Path -LiteralPath $Exe).Path
# A prebuilt viewer must contain a Windows AMD64 PE header before we run it.
$pe = [IO.BinaryReader]::new([IO.File]::OpenRead($exePath))
try {
    if ($pe.ReadUInt16() -ne 0x5a4d) { throw "Invalid Windows executable: $exePath" }
    $pe.BaseStream.Position = 0x3c
    $pe.BaseStream.Position = $pe.ReadInt32()
    if ($pe.ReadUInt32() -ne 0x00004550 -or $pe.ReadUInt16() -ne 0x8664) {
        throw "The package requires an x64 Windows executable: $exePath"
    }
} finally {
    $pe.Dispose()
}
& "$PSScriptRoot/verify-version.ps1" -Exe $exePath -Tag $Tag

$version = $Tag.Substring(1)
$packageName = "Dragon-Viewer-$Tag-windows-x64"
$parent = Split-Path $dist -Parent
New-Item -ItemType Directory -Force -Path $parent | Out-Null
$parent = (Resolve-Path -LiteralPath $parent).Path
$stageName = ".dragon-viewer-staging-$([Guid]::NewGuid().ToString('N'))"
$stage = Join-Path $parent $stageName
New-Item -ItemType Directory -Path $stage | Out-Null
try {
    $release = Join-Path $stage 'release'
    New-Item -ItemType Directory -Path $release | Out-Null
    $packageDir = Join-Path $stage $packageName
    $archive = Join-Path $release "$packageName.zip"
    $setup = Join-Path $release "Dragon-Viewer-Setup-$version.exe"
    & "$PSScriptRoot/package.ps1" -Exe $exePath -Out $packageDir
    # Windows tar writes ZIP paths with forward slashes, including the package root.
    & tar.exe -a -cf $archive -C $stage $packageName
    if ($LASTEXITCODE -ne 0) { throw "ZIP creation failed with exit code $LASTEXITCODE." }
    & "$PSScriptRoot/write-update-policy.ps1" -Tag $Tag -Out (Join-Path $release 'dragon-viewer-update-policy.json')

    $installerScript = Join-Path $repo 'packaging/windows/dragon-viewer.nsi'
    & $compiler /WX /V2 "/DVERSION=$version" "/DSRC=$packageDir" "/DOUTPUT=$setup" $installerScript
    if ($LASTEXITCODE -ne 0) { throw "NSIS failed with exit code $LASTEXITCODE." }
    foreach ($artifact in Get-Item -LiteralPath $archive, $setup) {
        $hash = (Get-FileHash -LiteralPath $artifact.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $($artifact.Name)" | Set-Content -LiteralPath "$($artifact.FullName).sha256" -Encoding ASCII
    }
    [IO.Directory]::Move($release, $dist)
} finally {
    $resolved = Get-Item -LiteralPath $stage -Force
    if ($resolved.Parent.FullName -ne $parent -or $resolved.Name -cne $stageName -or
        ($resolved.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Refused to remove a staging directory outside $parent."
    }
    Remove-Item -LiteralPath $resolved.FullName -Recurse -Force
}
Write-Host "Windows release artifacts: $dist"
