$toolsDir = "tools"
if (!(Test-Path $toolsDir)) { New-Item -ItemType Directory -Path $toolsDir }
$tccDir = "$toolsDir/tcc"
if (!(Test-Path $tccDir)) {
    Write-Host "[ BOOTSTRAP ] Downloading TCC..."
    # Using a more reliable TCC source
    $tccUrl = "http://download.savannah.gnu.org/releases/tinycc/tcc-0.9.27-win64-bin.zip"
    try {
        Invoke-WebRequest -Uri $tccUrl -OutFile "tcc.zip"
        Expand-Archive -Path "tcc.zip" -DestinationPath $tccDir -Force
        Remove-Item "tcc.zip"
    } catch {
        Write-Host "[ ERROR ] Failed to download TCC. Please ensure you have internet access."
        exit 1
    }
}

Write-Host "[ BOOTSTRAP ] Downloading Raylib binaries..."
# Raylib 5.0
$raylibUrl = "https://github.com/raysan5/raylib/releases/download/5.0/raylib-5.0_win64_mingw-w64.zip"
try {
    Invoke-WebRequest -Uri $raylibUrl -OutFile "raylib.zip"
    Expand-Archive -Path "raylib.zip" -DestinationPath "raylib_temp" -Force
    Copy-Item "raylib_temp/raylib-5.0_win64_mingw-w64/bin/raylib.dll" "./raylib.dll"
    Copy-Item "raylib_temp/raylib-5.0_win64_mingw-w64/include/raylib.h" "./raylib.h"
    Copy-Item "raylib_temp/raylib-5.0_win64_mingw-w64/lib/libraylib.a" "./raylib.lib"
    Remove-Item "raylib.zip"
    Remove-Item "raylib_temp" -Recurse
} catch {
    Write-Host "[ ERROR ] Failed to download Raylib. Please ensure you have internet access."
    exit 1
}
Write-Host "[ BOOTSTRAP ] Ready."
