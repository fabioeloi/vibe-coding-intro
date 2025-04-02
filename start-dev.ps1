# Safari History Knowledge Graph - Development Environment Setup and Start Script
# This script ensures the proper environment paths are set and starts the development server

# Add Node.js to the current session path if needed
$nodePath = "C:\Program Files\nodejs"
if ($env:Path -notlike "*$nodePath*") {
    $env:Path = "$nodePath;$env:Path"
    Write-Host "Added Node.js to PATH for this session" -ForegroundColor Green
}

# Check if Node.js and npm are accessible
try {
    $nodeVersion = node --version
    $npmVersion = npm --version
    Write-Host "Node.js $nodeVersion and npm $npmVersion are available" -ForegroundColor Green
} catch {
    Write-Host "Failed to access Node.js or npm. Please ensure Node.js is installed properly." -ForegroundColor Red
    exit 1
}

# Add .cargo\bin to PATH for Rust tools if needed
$userProfile = $env:USERPROFILE
$cargoPath = "$userProfile\.cargo\bin"
if ($env:Path -notlike "*$cargoPath*") {
    $env:Path = "$cargoPath;$env:Path"
    Write-Host "Added Cargo to PATH for this session" -ForegroundColor Green
}

# Check if Rust and Cargo are accessible
try {
    $rustVersion = rustc --version
    $cargoVersion = cargo --version
    Write-Host "Rust $rustVersion and $cargoVersion are available" -ForegroundColor Green
} catch {
    Write-Host "Failed to access Rust or Cargo. Please ensure Rust is installed properly." -ForegroundColor Red
    Write-Host "You may need to install Rust using rustup-init.exe" -ForegroundColor Yellow
    exit 1
}

# Kill any existing Tauri processes
Write-Host "Checking for existing Tauri processes..." -ForegroundColor Yellow
$processes = Get-Process | Where-Object { $_.ProcessName -like "*tauri*" -or $_.ProcessName -like "*node*" -or $_.ProcessName -like "*cargo*" } | Where-Object { $_.Path -like "*Environments\windows-windsurf01*" }
if ($processes) {
    Write-Host "Killing existing processes..." -ForegroundColor Yellow
    $processes | ForEach-Object {
        Write-Host "Stopping process: $($_.ProcessName) (ID: $($_.Id))" -ForegroundColor Yellow
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    }
}

# Install project dependencies with legacy peer deps to bypass version conflicts
Write-Host "Installing npm dependencies..." -ForegroundColor Cyan
npm install --legacy-peer-deps

# Make sure Tauri CLI is available
Write-Host "Installing Tauri CLI globally..." -ForegroundColor Cyan
npm install -g @tauri-apps/cli

# Start the development server
Write-Host "Starting the development server..." -ForegroundColor Green
npm run tauri dev

# If npm run tauri dev fails, try using cargo directly
if ($LASTEXITCODE -ne 0) {
    Write-Host "Trying alternative method with cargo..." -ForegroundColor Yellow
    cargo tauri dev
}
