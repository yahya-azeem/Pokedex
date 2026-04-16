# setup-container.ps1
# Builds the Alpine or Kali Linux WASM image for Pokedex containerized agents.
#
# Requirements:
#   - Docker Desktop installed and running
#
# Usage:
#   .\scripts\setup-container.ps1 -Distribution Alpine
#   .\scripts\setup-container.ps1 -Distribution Kali
#   .\scripts\setup-container.ps1 -Force   # Rebuild even if image exists

param(
    [ValidateSet("Alpine", "Kali", "OpenBSD")]
    [string]$Distribution = "Alpine",
    [switch]$Force  # Rebuild even if image already exists
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$ImagesDir = Join-Path $ProjectRoot ".pokedex\images"

# Configure source and output based on distribution
$SourceImage = ""
$WasmFilename = ""
$InstallCommands = ""
$RawImageURL = ""

if ($Distribution -eq "Alpine") {
    $SourceImage = "alpine:3.20"
    $WasmFilename = "alpine-amd64.wasm"
    # Basic tools already in Alpine or handled by shim
} elseif ($Distribution -eq "Kali") {
    $SourceImage = "kalilinux/kali-rolling"
    $WasmFilename = "kali-amd64.wasm"
    # Install security essentials in Kali
    $InstallCommands = "apt-get update && apt-get install -y nmap netcat-traditional curl wget socat git && apt-get clean && rm -rf /var/lib/apt/lists/*"
} elseif ($Distribution -eq "OpenBSD") {
    $WasmFilename = "openbsd-amd64.wasm"
    # Official OpenBSD 7.5 AMD64 raw image
    $RawImageURL = "https://cdn.openbsd.org/pub/OpenBSD/7.5/amd64/miniroot75.img"
}

$WasmPath = Join-Path $ImagesDir $WasmFilename

# Ensure output directory exists
if (-not (Test-Path $ImagesDir)) {
    New-Item -ItemType Directory -Path $ImagesDir -Force | Out-Null
}

# Check if image already exists
if ((Test-Path $WasmPath) -and -not $Force) {
    $fileSize = (Get-Item $WasmPath).Length
    if ($fileSize -gt 1024) {
        Write-Host "$Distribution container image already exists at $WasmPath ($([math]::Round($fileSize / 1MB, 1)) MB)" -ForegroundColor Green
        Write-Host "Use -Force to rebuild." -ForegroundColor DarkGray
        exit 0
    }
}

# Check Docker is available
try {
    docker info 2>&1 | Out-Null
} catch {
    Write-Error "Docker is not running or not installed. This script requires Docker Desktop."
    exit 1
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Building $Distribution AMD64 WASM Image" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Source image : $SourceImage" -ForegroundColor White
Write-Host "Output       : $WasmPath" -ForegroundColor White
Write-Host ""
Write-Host "This will take several minutes..." -ForegroundColor DarkGray
Write-Host ""

$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

if ($RawImageURL) {
    # Path B: Raw Image conversion (OpenBSD)
    Write-Host "Step 1: Downloading raw image from $RawImageURL..." -ForegroundColor DarkGray
    $rawPath = Join-Path $ImagesDir "source-$($Distribution.ToLower()).img"
    Invoke-WebRequest -Uri $RawImageURL -OutFile $rawPath
    
    Write-Host "Step 2: Converting raw image to WASM (amd64)..." -ForegroundColor DarkGray
    docker run --rm --privileged -v .:/work -w /work docker:cli sh -c "
        apk add --no-cache curl tar &&
        curl -fsSL https://github.com/container2wasm/container2wasm/releases/download/v0.8.4/container2wasm-v0.8.4-linux-amd64.tar.gz | tar xz -C /usr/local/bin/ &&
        c2w --target-arch amd64 --image /work/.pokedex/images/$($rawPath | Split-Path -Leaf) /work/.pokedex/images/$WasmFilename
    "
    Remove-Item $rawPath -ErrorAction SilentlyContinue
} else {
    # Path A: Docker-based conversion (Linux)
    Write-Host "Step 1: Pulling $SourceImage..." -ForegroundColor DarkGray
    docker pull $SourceImage
    if ($LASTEXITCODE -ne 0) { Write-Error "Failed to pull image"; exit 1 }

    # Step 2: Prepare the image
    $PreparationTag = "pokedex-prep-$($Distribution.ToLower())"
    if ($InstallCommands) {
        Write-Host "Step 2: Installing essentials in $Distribution..." -ForegroundColor DarkGray
        docker run --name "$PreparationTag-container" $SourceImage sh -c "$InstallCommands"
        if ($LASTEXITCODE -ne 0) { Write-Error "Failed to install tools"; exit 1 }
        docker commit "$PreparationTag-container" $PreparationTag
        docker rm "$PreparationTag-container" | Out-Null
    } else {
        $PreparationTag = $SourceImage
    }

    # Step 3: Save to tarball
    $tarPath = Join-Path $ImagesDir "prep-$($Distribution.ToLower()).tar"
    Write-Host "Step 3: Saving image to tarball..." -ForegroundColor DarkGray
    docker save -o $tarPath $PreparationTag
    if ($LASTEXITCODE -ne 0) { Write-Error "Failed to save Docker image"; exit 1 }

    # Step 4: Convert to WASM
    Write-Host "Step 4: Converting to WASM (amd64)..." -ForegroundColor DarkGray
    docker run --rm --privileged -v .:/work -v /var/run/docker.sock:/var/run/docker.sock -w /work docker:cli sh -c "
        apk add --no-cache curl tar &&
        curl -fsSL https://github.com/container2wasm/container2wasm/releases/download/v0.8.4/container2wasm-v0.8.4-linux-amd64.tar.gz | tar xz -C /usr/local/bin/ &&
        docker load -i /work/.pokedex/images/prep-$($Distribution.ToLower()).tar &&
        c2w --target-arch amd64 $PreparationTag /work/.pokedex/images/$WasmFilename
    "

    # Clean up
    if ($PreparationTag -ne $SourceImage) { docker rmi $PreparationTag | Out-Null }
    Remove-Item $tarPath -ErrorAction SilentlyContinue
}

$stopwatch.Stop()
$elapsed = $stopwatch.Elapsed

if (Test-Path $WasmPath) {
    $fileSize = (Get-Item $WasmPath).Length
    if ($fileSize -gt 1024) {
        Write-Host "Successfully built $Distribution container image!" -ForegroundColor Green
        Write-Host "  Path : $WasmPath" -ForegroundColor White
        Write-Host "  Size : $([math]::Round($fileSize / 1MB, 1)) MB" -ForegroundColor White
        Write-Host "  Time : $($elapsed.Minutes)m $($elapsed.Seconds)s" -ForegroundColor White
    } else {
        Write-Error "Output file exists but appears too small ($fileSize bytes). Build may have failed."
        exit 1
    }
} else {
    Write-Error "Build appeared to succeed but output file not found at $WasmPath"
    exit 1
}
