# Knockd Client Extension Installer — Windows
param(
    [Parameter(Mandatory=$true)]
    [string]$ExtensionId,
    [ValidateSet("chrome","edge")]
    [string]$Browser = "chrome"
)

$ErrorActionPreference = "Stop"
$BIN_PATH = "$env:ProgramFiles\Knockd Client\knockd-client.exe"
if (-not (Test-Path $BIN_PATH)) {
    $BIN_PATH = "$env:LOCALAPPDATA\Knockd Client\knockd-client.exe"
}

if ($Browser -eq "edge") {
    $REG_KEY = "HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\com.knockd.client"
} else {
    $REG_KEY = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.knockd.client"
}
$SCRIPT_DIR = Split-Path -Parent $PSCommandPath
$PROJECT_DIR = Split-Path -Parent $SCRIPT_DIR

Write-Host "=== Knockd Client Extension Installer ===" -ForegroundColor Cyan
Write-Host "Extension: $ExtensionId  Browser: $Browser" -ForegroundColor Gray

if (-not (Test-Path $BIN_PATH)) {
    Write-Host "WARNING: $BIN_PATH not found. Run the installer first." -ForegroundColor Yellow
    Write-Host "Using project binary path..." -ForegroundColor Yellow
    $BIN_PATH = "$PROJECT_DIR\src-tauri\target\release\knockd-client.exe"
}

Write-Host "[1/2] Registering Native Messaging Host..." -ForegroundColor Cyan
$manifest = @"
{
  "name": "com.knockd.client",
  "description": "Knockd Client - SPA auth gateway",
  "path": "$($BIN_PATH -replace '\\','\\')",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://$ExtensionId/"]
}
"@
$MANIFEST_DIR = "$env:LOCALAPPDATA\knockd-client"
New-Item -ItemType Directory -Force -Path $MANIFEST_DIR | Out-Null
$manifest | Out-File "$MANIFEST_DIR\com.knockd.client.json" -Encoding UTF8
New-Item -Path $REG_KEY -Force | Out-Null
Set-ItemProperty -Path $REG_KEY -Name "(Default)" -Value "$MANIFEST_DIR\com.knockd.client.json"
Write-Host "  Registry: $REG_KEY" -ForegroundColor Gray

Write-Host "[2/2] Load Extension" -ForegroundColor Cyan
Write-Host "  chrome://extensions -> Developer mode -> Load unpacked" -ForegroundColor Gray
Write-Host "  Select: $PROJECT_DIR\extension\" -ForegroundColor Gray
Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Green
Write-Host "Restart browser for changes to take effect."
