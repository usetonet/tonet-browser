#Requires -Version 5.1
<#
.SYNOPSIS
  Run cargo for Tonet (Servo is enabled by default) after resolving LIBCLANG_PATH on Windows.

.DESCRIPTION
  The Servo dependency chain (mozjs / bindgen) needs libclang.dll. This script:
  1) Uses LIBCLANG_PATH if it already points to a folder containing libclang.dll
  2) Otherwise searches common install locations (LLVM installer, Scoop, Chocolatey)
  3) Runs cargo with the remaining arguments (default: run -p tonet; Servo is the default engine feature)

  First Servo build on Windows compiles ANGLE (mozangle C++); needs MSVC "Desktop development with C++".
  Tonet's build.rs copies libEGL.dll / libGLESv2.dll next to tonet.exe so surfman can load EGL.

.EXAMPLE
  .\scripts\tonet-servo-windows.ps1
  # PowerShell may swallow "-p"; use stop-parsing (--%) if cargo misses "-p tonet":
  .\scripts\tonet-servo-windows.ps1 --% run -p tonet
  .\scripts\tonet-servo-windows.ps1 --% check -p tonet
#>
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArguments = @('run', '-p', 'tonet')
)

$ErrorActionPreference = 'Stop'

function Test-LibClangDir([string]$Dir) {
    if (-not $Dir) { return $false }
    $dll = Join-Path $Dir 'libclang.dll'
    return (Test-Path -LiteralPath $dll)
}

function Find-LibClangDirectory {
    if (Test-LibClangDir $env:LIBCLANG_PATH) {
        return (Resolve-Path -LiteralPath $env:LIBCLANG_PATH).Path
    }

    $searchRoots = @(
        'C:\Program Files\LLVM\bin',
        'C:\Program Files (x86)\LLVM\bin',
        'C:\LLVM\bin',
        (Join-Path $env:ProgramFiles 'LLVM\bin'),
        (Join-Path ${env:ProgramFiles(x86)} 'LLVM\bin')
    )

    $u = $env:USERPROFILE
    if ($u) {
        $searchRoots += (Join-Path $u 'scoop\apps\llvm\current\bin')
        $searchRoots += (Join-Path $u 'chocolatey\lib\llvm\tools\llvm\bin')
        $scoopGlob = Join-Path $u 'scoop\apps\llvm\*\bin'
        $scoopHit = Get-Item -Path $scoopGlob -ErrorAction SilentlyContinue |
            Sort-Object { $_.FullName } -Descending |
            Select-Object -First 1 -ExpandProperty FullName
        if ($scoopHit) { $searchRoots += $scoopHit }
    }

    foreach ($root in $searchRoots) {
        if (-not $root) { continue }
        if ($root -match '\*') {
            $resolved = Get-Item -Path $root -ErrorAction SilentlyContinue | Sort-Object FullName -Descending | Select-Object -First 1
            if ($resolved -and (Test-LibClangDir $resolved.FullName)) {
                return $resolved.FullName
            }
            continue
        }
        if (Test-LibClangDir $root) {
            return (Resolve-Path -LiteralPath $root).Path
        }
    }

    return $null
}

Write-Host "=== Tonet + Servo (Windows) ===" -ForegroundColor Cyan
Write-Host "MSVC: use 'x64 Native Tools Command Prompt' or install VS workload 'Desktop development with C++' if link fails."
$link = Get-Command link.exe -ErrorAction SilentlyContinue
if ($link) {
    Write-Host "link.exe: $($link.Source)" -ForegroundColor Green
} else {
    Write-Host "link.exe not in PATH (Rust MSVC may still work; if link fails, use VS Developer shell)." -ForegroundColor Yellow
}

$clangDir = Find-LibClangDirectory
if (-not $clangDir) {
    Write-Host ""
    Write-Host "libclang.dll not found (bindgen / mozjs)." -ForegroundColor Red
    Write-Host "Current LIBCLANG_PATH: '$($env:LIBCLANG_PATH)'"
    Write-Host ""
    Write-Host "Install LLVM (includes libclang.dll), e.g.:"
    Write-Host "  winget install -e --id LLVM.LLVM"
    Write-Host "Then re-run this script, or set manually for one session:"
    Write-Host '  $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"'
    Write-Host "  cargo build -p tonet"
    Write-Host ""
    exit 1
}

$env:LIBCLANG_PATH = $clangDir
Write-Host "Using LIBCLANG_PATH=$clangDir" -ForegroundColor Green

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Push-Location $repoRoot
try {
    # PowerShell can swallow `-p tonet` unless `--%` is used. If only `run`/`check`/`build`
    # was forwarded, keep the common Tonet target by default.
    if ($CargoArguments.Count -ge 1) {
        $cmd0 = $CargoArguments[0].ToLowerInvariant()
        if (($cmd0 -eq 'run' -or $cmd0 -eq 'check' -or $cmd0 -eq 'build') -and -not ($CargoArguments -contains '-p')) {
            $CargoArguments += @('-p', 'tonet')
        }
    }
    Write-Host "cargo $($CargoArguments -join ' ')" -ForegroundColor Cyan
    & cargo @CargoArguments
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
