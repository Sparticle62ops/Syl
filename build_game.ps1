$ErrorActionPreference = "Stop"
$ProjectDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $ProjectDir
try {
    cargo run --bin syl -- build examples/web_game.syl
} finally {
    Pop-Location
}
