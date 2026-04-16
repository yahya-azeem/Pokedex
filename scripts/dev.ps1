# Pokedex Swarm — Development Server Launcher
# Starts both the Rust backend and Svelte frontend concurrently.

Write-Host ""
Write-Host "  🔴 Pokedex Swarm — Development Mode" -ForegroundColor Red
Write-Host "  =====================================" -ForegroundColor DarkGray
Write-Host ""

$workspace = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not (Test-Path "$workspace\Cargo.toml")) {
    $workspace = $PSScriptRoot | Split-Path -Parent
}

Write-Host "  Workspace: $workspace" -ForegroundColor DarkGray
Write-Host ""

# Start Rust backend
Write-Host "  Starting Rust backend server..." -ForegroundColor Cyan
$backend = Start-Process -FilePath "cargo" -ArgumentList "run", "-p", "pokedex-server" -WorkingDirectory $workspace -PassThru -NoNewWindow

Start-Sleep -Seconds 2

# Start Svelte frontend
Write-Host "  Starting Svelte frontend dev server..." -ForegroundColor Cyan
$frontend = Start-Process -FilePath "npm" -ArgumentList "run", "dev" -WorkingDirectory "$workspace\web" -PassThru -NoNewWindow

Write-Host ""
Write-Host "  Backend:  http://localhost:5001" -ForegroundColor Green
Write-Host "  Frontend: http://localhost:5173" -ForegroundColor Green
Write-Host "  WebSocket: ws://localhost:5001/ws" -ForegroundColor Green
Write-Host ""
Write-Host "  Press Ctrl+C to stop both servers." -ForegroundColor DarkGray
Write-Host ""

try {
    Wait-Process -Id $backend.Id, $frontend.Id
} finally {
    Write-Host ""
    Write-Host "  Shutting down..." -ForegroundColor Yellow
    Stop-Process -Id $backend.Id -ErrorAction SilentlyContinue
    Stop-Process -Id $frontend.Id -ErrorAction SilentlyContinue
    Write-Host "  Done." -ForegroundColor Green
}
