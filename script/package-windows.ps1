# Package Windows zip of the release binary (no MSI yet — GPUI Windows unverified).
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$Version = (Select-String -Path Cargo.toml -Pattern '^version = "([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value
if (-not $Version) { $Version = "0.0.0" }
$Dist = "dist"
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

$Bin = "target\release\devtools.exe"
if (-not (Test-Path $Bin)) {
  $Bin = "target\x86_64-pc-windows-msvc\release\devtools.exe"
}
if (-not (Test-Path $Bin)) { throw "devtools.exe not found" }

$Name = "devtools-$Version-windows-x86_64"
$Stage = Join-Path $Dist $Name
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item $Bin (Join-Path $Stage "devtools.exe")
Copy-Item README.md $Stage
$Zip = Join-Path $Dist "$Name.zip"
if (Test-Path $Zip) { Remove-Item $Zip }
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip
Write-Host "Wrote $Zip"
