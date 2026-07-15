param(
    [Parameter(Mandatory = $true)]
    [string]$AppExecutable,

    [Parameter(Mandatory = $true)]
    [string]$OutputPath,

    [Parameter(Mandatory = $true)]
    [string]$Version,

    [string]$IdentityName = "dotSlashZ.MRRM",
    [string]$Publisher = "CN=175EBAB6-5A58-4E85-84CA-602CD5A1C63D",
    [string]$PublisherDisplayName = "dotSlashZ",
    [string]$StoreDisplayName = "MRRM"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $AppExecutable -PathType Leaf)) {
    throw "Application executable not found: $AppExecutable"
}

$versionParts = $Version.Split('.')
if ($versionParts.Count -gt 4 -or $versionParts.Count -lt 1 -or
    ($versionParts | Where-Object { $_ -notmatch '^\d+$' })) {
    throw "MSIX version must contain one to four numeric components: $Version"
}
$msixVersion = (($versionParts + @('0', '0', '0', '0'))[0..3] -join '.')

$makeAppx = Get-Command makeappx.exe -ErrorAction SilentlyContinue
if (-not $makeAppx) {
    $kitsRoot = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
    $makeAppxPath = Get-ChildItem -Path $kitsRoot -Filter makeappx.exe -Recurse -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match '\\x64\\makeappx\.exe$' } |
        Sort-Object FullName -Descending |
        Select-Object -First 1
    if (-not $makeAppxPath) {
        throw "makeappx.exe was not found. Install the Windows SDK."
    }
    $makeAppx = $makeAppxPath.FullName
}
else {
    $makeAppx = $makeAppx.Source
}

$projectRoot = Split-Path -Parent $PSScriptRoot
$iconRoot = Join-Path $projectRoot "src-tauri\icons"
$stagingRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("mrrm-msix-" + [guid]::NewGuid())
$assetsRoot = Join-Path $stagingRoot "Assets"

try {
    New-Item -ItemType Directory -Force -Path $assetsRoot | Out-Null
    Copy-Item -LiteralPath $AppExecutable -Destination (Join-Path $stagingRoot "ModernRetroRomManager.exe")

    foreach ($asset in @("StoreLogo.png", "Square44x44Logo.png", "Square150x150Logo.png")) {
        $source = Join-Path $iconRoot $asset
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
            throw "MSIX asset not found: $source"
        }
        Copy-Item -LiteralPath $source -Destination (Join-Path $assetsRoot $asset)
    }

    $escapedIdentity = [System.Security.SecurityElement]::Escape($IdentityName)
    $escapedPublisher = [System.Security.SecurityElement]::Escape($Publisher)
    $escapedPublisherDisplayName = [System.Security.SecurityElement]::Escape($PublisherDisplayName)
    $escapedStoreDisplayName = [System.Security.SecurityElement]::Escape($StoreDisplayName)

    $manifest = @"
<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap rescap">
  <Identity
    Name="$escapedIdentity"
    Publisher="$escapedPublisher"
    Version="$msixVersion"
    ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>$escapedStoreDisplayName</DisplayName>
    <PublisherDisplayName>$escapedPublisherDisplayName</PublisherDisplayName>
    <Logo>Assets\StoreLogo.png</Logo>
  </Properties>
  <Resources>
    <Resource Language="en-us" />
    <Resource Language="zh-cn" />
    <Resource Language="zh-tw" />
    <Resource Language="fr-fr" />
    <Resource Language="de-de" />
    <Resource Language="it-it" />
    <Resource Language="es-es" />
    <Resource Language="ru-ru" />
  </Resources>
  <Dependencies>
    <TargetDeviceFamily
      Name="Windows.Desktop"
      MinVersion="10.0.17763.0"
      MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Applications>
    <Application
      Id="ModernRetroRomManager"
      Executable="ModernRetroRomManager.exe"
      EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="$escapedStoreDisplayName"
        Description="ROM library and metadata manager"
        BackgroundColor="transparent"
        Square150x150Logo="Assets\Square150x150Logo.png"
        Square44x44Logo="Assets\Square44x44Logo.png" />
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
"@

    Set-Content -LiteralPath (Join-Path $stagingRoot "AppxManifest.xml") -Value $manifest -Encoding utf8
    $outputDirectory = Split-Path -Parent $OutputPath
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
    if (Test-Path -LiteralPath $OutputPath) {
        Remove-Item -LiteralPath $OutputPath -Force
    }

    & $makeAppx pack /d $stagingRoot /p $OutputPath /o
    if ($LASTEXITCODE -ne 0) {
        throw "makeappx.exe failed with exit code $LASTEXITCODE"
    }
}
finally {
    Remove-Item -LiteralPath $stagingRoot -Recurse -Force -ErrorAction SilentlyContinue
}
