# ============================================================
# Syl Language SDK Installer v1.3
# Installs Syl to the current user's AppData (no Admin required)
# ============================================================

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  [SYL INSTALLER] Syl Language SDK v1.3.0" -ForegroundColor Cyan
Write-Host "  ========================================" -ForegroundColor Cyan
Write-Host ""

# --- 1. Define Paths ---
$SylRoot   = "$env:LOCALAPPDATA\Syl"
$BinDir    = "$SylRoot\bin"
$LibDir    = "$SylRoot\lib"
$TccDir    = "$BinDir\tcc"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "  [1/6] Target: $SylRoot" -ForegroundColor Yellow

# --- 2. Create Directories ---
Write-Host "  [2/6] Creating SDK directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $LibDir | Out-Null
New-Item -ItemType Directory -Force -Path $TccDir | Out-Null

# --- 3. Build the Compiler (if cargo is available) ---
$CargoPath = "$ScriptDir\tools\rust\.cargo\bin\cargo.exe"
$ReleaseBin = "$ScriptDir\syl\target\release\syl.exe"

if (Test-Path $CargoPath) {
    Write-Host "  [3/6] Building syl.exe (release mode)..." -ForegroundColor Yellow
    try {
        Push-Location "$ScriptDir\syl"
        & $CargoPath build --release 2>&1 | Out-Null
        Pop-Location
    } catch {
        Write-Host "  [3/6] SKIP: Cargo build failed (Zig linker issue). Using pre-built binary if available." -ForegroundColor DarkYellow
    }
} else {
    Write-Host "  [3/6] SKIP: Cargo not found. Using pre-built binary if available." -ForegroundColor DarkYellow
}

# --- 4. Copy Binaries ---
Write-Host "  [4/6] Copying binaries to $BinDir ..." -ForegroundColor Yellow

# Copy syl.exe (prefer release build, fall back to debug)
if (Test-Path $ReleaseBin) {
    Copy-Item $ReleaseBin -Destination $BinDir -Force
    Write-Host "         -> syl.exe (release)" -ForegroundColor Green
} elseif (Test-Path "$ScriptDir\syl\target\debug\syl.exe") {
    Copy-Item "$ScriptDir\syl\target\debug\syl.exe" -Destination $BinDir -Force
    Write-Host "         -> syl.exe (debug)" -ForegroundColor Green
} else {
    Write-Host "         -> syl.exe NOT FOUND. Build the compiler first." -ForegroundColor Red
}

# Copy raylib.dll if present
if (Test-Path "$ScriptDir\syl\raylib.dll") {
    Copy-Item "$ScriptDir\syl\raylib.dll" -Destination $BinDir -Force
    Write-Host "         -> raylib.dll" -ForegroundColor Green
}

# Copy TCC portable compiler
$TccSource = "$ScriptDir\tools\tcc\tcc"
if (Test-Path $TccSource) {
    Copy-Item "$TccSource\*" -Destination $TccDir -Recurse -Force
    Write-Host "         -> tcc/ (portable C compiler)" -ForegroundColor Green
}

# --- 5. Copy Standard Library ---
Write-Host "  [5/6] Installing standard library to $LibDir ..." -ForegroundColor Yellow
$LibSource = "$ScriptDir\syl\lib"
if (Test-Path $LibSource) {
    Get-ChildItem "$LibSource\*.syl" | ForEach-Object {
        Copy-Item $_.FullName -Destination $LibDir -Force
        Write-Host "         -> $($_.Name)" -ForegroundColor Green
    }
} else {
    Write-Host "         -> No lib/ directory found." -ForegroundColor Red
}

# --- 6. Update User PATH (no Admin required) ---
Write-Host "  [6/6] Updating User PATH..." -ForegroundColor Yellow

$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -and $CurrentPath.Split(";") -contains $BinDir) {
    Write-Host "         -> PATH already contains $BinDir (skipped)" -ForegroundColor Green
} else {
    # Append to user PATH via the registry (survives reboots, no Admin)
    if ($CurrentPath) {
        $NewPath = "$CurrentPath;$BinDir"
    } else {
        $NewPath = $BinDir
    }
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "         -> Added $BinDir to User PATH" -ForegroundColor Green
    Write-Host "         -> NOTE: Restart your terminal for PATH changes to take effect." -ForegroundColor DarkYellow
}

# --- 7. Set SYL_LIB_PATH ---
[Environment]::SetEnvironmentVariable("SYL_LIB_PATH", $LibDir, "User")
Write-Host "         -> Set SYL_LIB_PATH = $LibDir" -ForegroundColor Green

# --- Done ---
Write-Host ""
Write-Host "  [DONE] Syl SDK installed successfully!" -ForegroundColor Cyan
Write-Host "         Run 'syl' from any terminal to verify." -ForegroundColor Cyan
Write-Host ""
