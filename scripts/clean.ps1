# Clean NEUPM Build Artifacts
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Write-Host "Cleaning build directory..." -ForegroundColor Cyan
cargo clean
Write-Host "Clean completed." -ForegroundColor Green
