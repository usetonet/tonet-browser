# Compila Tonet en release y genera el MSI con WiX Toolset 3.x (candle + light).
# Requisitos: Rust, WiX en PATH (candle.exe, light.exe) o variable WIX.
# Uso: .\scripts\build-installer.ps1  [-Version 0.1.0]

param(
    [string]$Version = "",
    [string]$WixRoot = $env:WIX
)

$ErrorActionPreference = "Stop"
# Raíz del repositorio (scripts/ -> padre)
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

if (-not $Version) {
    $meta = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    $Version = ($meta.packages | Where-Object { $_.name -eq "tonet" } | Select-Object -First 1).version
    if (-not $Version) { throw "No se pudo leer la versión del crate tonet desde Cargo.toml." }
}

# WiX 3.x suele instalarse en %WIX% o Program Files
$WixBin = $null
if ($WixRoot -and (Test-Path "$WixRoot\bin\candle.exe")) {
    $WixBin = "$WixRoot\bin"
}
if (-not $WixBin) {
    $candle = Get-Command candle.exe -ErrorAction SilentlyContinue
    if ($candle) { $WixBin = Split-Path $candle.Source }
}
if (-not $WixBin) {
    throw "No se encontró WiX (candle.exe). Instala WiX Toolset v3.11+ o define WIX apuntando a la carpeta de instalación."
}

Write-Host "Compilando tonet en release..." -ForegroundColor Cyan
cargo build --release

$Exe = Join-Path $Root "target\release\tonet.exe"
if (-not (Test-Path $Exe)) {
    throw "No existe $Exe tras cargo build --release."
}

$Dist = Join-Path $Root "dist"
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

$Wxs = Join-Path $Root "wix\Main.wxs"
$Obj = Join-Path $Dist "Main.wixobj"
$MsiName = "Tonet-$Version-x64.msi"
$Msi = Join-Path $Dist $MsiName

$SourceDir = Join-Path $Root "target\release"

Write-Host "WiX candle (x64)..." -ForegroundColor Cyan
# Las comillas evitan que PowerShell trocee valores como 0.1.0 en varios argumentos.
& "$WixBin\candle.exe" -nologo -arch x64 `
    "-dProductVersion=$Version" `
    "-dSourceDir=$SourceDir" `
    -out $Dist\ `
    -ext WixUIExtension `
    $Wxs

Write-Host "WiX light..." -ForegroundColor Cyan
& "$WixBin\light.exe" -nologo -out $Msi `
    -ext WixUIExtension `
    -cultures:en-us `
    (Join-Path $Dist "Main.wixobj")

Write-Host "MSI generado: $Msi" -ForegroundColor Green
