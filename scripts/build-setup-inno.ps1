# Genera el instalador EXE con Inno Setup 6 (después de cargo build --release).
# Requisitos: Inno Setup 6 (ISCC.exe en PATH o INNO_SETUP).

param(
    [string]$InnoCompiler = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

Write-Host "Compilando tonet en release..." -ForegroundColor Cyan
cargo build --release -p tonet

$Exe = Join-Path $Root "target\release\tonet.exe"
if (-not (Test-Path $Exe)) {
    throw "No existe $Exe tras cargo build --release."
}

$iscc = $InnoCompiler
if (-not $iscc) {
    $envPath = $env:INNO_SETUP
    if ($envPath -and (Test-Path $envPath)) {
        $iscc = $envPath
    }
}
if (-not $iscc) {
    $candidates = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
    )
    foreach ($p in $candidates) {
        if (Test-Path $p) { $iscc = $p; break }
    }
}
if (-not $iscc -or -not (Test-Path $iscc)) {
    throw "No se encontró ISCC.exe. Instala Inno Setup 6 o define INNO_SETUP con la ruta completa a ISCC.exe."
}

$Iss = Join-Path $Root "installer\tonet.iss"
Write-Host "Inno Setup: $iscc $Iss" -ForegroundColor Cyan
& $iscc $Iss
Write-Host "Revisa la carpeta dist\ para Tonet-Setup-*-x64.exe" -ForegroundColor Green
