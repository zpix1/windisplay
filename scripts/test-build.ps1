# PowerShell script to test the Tauri build locally before creating a release

Write-Host "Testing Tauri build for Windows..." -ForegroundColor Green

# Check if required tools are installed
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check Node.js
if (Get-Command node -ErrorAction SilentlyContinue) {
    $nodeVersion = node --version
    Write-Host "✓ Node.js: $nodeVersion" -ForegroundColor Green
} else {
    Write-Host "✗ Node.js not found. Please install Node.js" -ForegroundColor Red
    exit 1
}

# Check Rust
if (Get-Command rustc -ErrorAction SilentlyContinue) {
    $rustVersion = rustc --version
    Write-Host "✓ Rust: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "✗ Rust not found. Please install Rust" -ForegroundColor Red
    exit 1
}

# Check npm
if (Get-Command npm -ErrorAction SilentlyContinue) {
    $npmVersion = npm --version
    Write-Host "✓ npm: $npmVersion" -ForegroundColor Green
} else {
    Write-Host "✗ npm not found" -ForegroundColor Red
    exit 1
}

Write-Host "`nInstalling dependencies..." -ForegroundColor Yellow
npm install

if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Failed to install npm dependencies" -ForegroundColor Red
    exit 1
}

Write-Host "✓ Dependencies installed successfully" -ForegroundColor Green

Write-Host "`nBuilding Tauri application..." -ForegroundColor Yellow
npm run tauri build

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✓ Build completed successfully!" -ForegroundColor Green
    Write-Host "Check the 'src-tauri/target/release/bundle' directory for the generated installers." -ForegroundColor Cyan
    
    # List generated files
    $bundleDir = "src-tauri/target/release/bundle"
    if (Test-Path $bundleDir) {
        Write-Host "`nGenerated files:" -ForegroundColor Cyan
        Get-ChildItem -Recurse $bundleDir -File | ForEach-Object {
            Write-Host "  $($_.FullName)" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "`n✗ Build failed" -ForegroundColor Red
    exit 1
}
