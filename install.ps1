$ErrorActionPreference = "Stop"

Write-Host "Building and installing logtok..."

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust is not installed. Install it from https://rustup.rs" -ForegroundColor Red
    exit 1
}

cargo install --path .

Write-Host ""
Write-Host "Done! logtok is now available globally." -ForegroundColor Green
Write-Host "Try: logtok --help"
