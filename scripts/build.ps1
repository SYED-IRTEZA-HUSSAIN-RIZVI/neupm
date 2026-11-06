# Build NEUPM Release Binary
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Write-Host "Formatting code..." -ForegroundColor Cyan
cargo fmt --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "Code formatting issues found. Formatting..." -ForegroundColor Yellow
    cargo fmt
}

Write-Host "Compiling release binary..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -eq 0) {
    Write-Host "Build succeeded: target\release\neupm.exe" -ForegroundColor Green
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
