# Run NEUPM Automated Test Suite
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Write-Host "Running NEUPM unit and integration tests..." -ForegroundColor Cyan
cargo test --verbose
if ($LASTEXITCODE -eq 0) {
    Write-Host "All tests passed!" -ForegroundColor Green
} else {
    Write-Host "Tests failed!" -ForegroundColor Red
    exit 1
}
