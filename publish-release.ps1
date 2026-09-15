param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    [string]$CommitMessage = ""
)

$ErrorActionPreference = "Stop"

Write-Host "=== Building and Publishing ScarCord OTA Release v$Version ===" -ForegroundColor Cyan

# 1. Verify key exists
$KeyPath = "$HOME\.tauri\scarcord.key"
if (-not (Test-Path -LiteralPath $KeyPath)) {
    Write-Error "Signing key not found at $KeyPath. Please ensure the private key exists."
    exit 1
}

# 2. Update version in Cargo.toml and tauri.conf.json
Write-Host "Updating version to $Version..." -ForegroundColor Yellow
$CargoTomlPath = "D:\ProjectP\ScarCord\src-tauri\Cargo.toml"
$TauriConfPath = "D:\ProjectP\ScarCord\src-tauri\tauri.conf.json"

$content = Get-Content $CargoTomlPath -Raw
$content = $content -replace '(?m)^version = ".*"', "version = `"$Version`""
Set-Content -Path $CargoTomlPath -Value $content

$tauriConf = Get-Content $TauriConfPath -Raw
$tauriConf = $tauriConf -replace '"version": ".*"', "`"version`": `"$Version`""
Set-Content -Path $TauriConfPath -Value $tauriConf

# 3. Build bundle using npx @tauri-apps/cli build with signing key
Write-Host "Building Tauri release..." -ForegroundColor Yellow
$env:CI = "true"
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $KeyPath -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""

Set-Location -LiteralPath "D:\ProjectP\ScarCord"
npx @tauri-apps/cli build

# 4. Sign executables with Authenticode certificate (TrulyScarlet)
Write-Host "Signing binary with TrulyScarlet certificate..." -ForegroundColor Yellow
$Cert = Get-ChildItem -Path Cert:\CurrentUser\My -CodeSigningCert | Where-Object { $_.Subject -like "*TrulyScarlet*" } | Select-Object -First 1

if ($Cert) {
    Set-AuthenticodeSignature -FilePath "D:\ProjectP\ScarCord\src-tauri\target\release\scarcord.exe" -Certificate $Cert -TimestampServer "http://timestamp.digicert.com"
    Get-ChildItem "D:\ProjectP\ScarCord\src-tauri\target\release\bundle\nsis\*.exe" | ForEach-Object {
        Set-AuthenticodeSignature -FilePath $_.FullName -Certificate $Cert -TimestampServer "http://timestamp.digicert.com"
    }
    Get-ChildItem "D:\ProjectP\ScarCord\src-tauri\target\release\bundle\msi\*.msi" | ForEach-Object {
        Set-AuthenticodeSignature -FilePath $_.FullName -Certificate $Cert -TimestampServer "http://timestamp.digicert.com"
    }
} else {
    Write-Warning "Could not find TrulyScarlet certificate in CurrentUser\My store."
}

# 5. Re-sign installer with Tauri Minisign AFTER Authenticode signing so hash matches exact bytes
Write-Host "Generating accurate Minisign OTA signature for signed installer..." -ForegroundColor Yellow
$SetupExe = Get-Item "D:\ProjectP\ScarCord\src-tauri\target\release\bundle\nsis\*_$($Version)_x64-setup.exe" | Select-Object -First 1
npx @tauri-apps/cli signer sign --password "" "$($SetupExe.FullName)"

$sig = Get-Content "$($SetupExe.FullName).sig" -Raw
$latestJson = @{
    version = "v$Version"
    notes = "ScarCord v$Version - OTA Release"
    pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = $sig.Trim()
            url = "https://github.com/TrulyScarlet/ScarCord/releases/download/v$Version/$($SetupExe.Name)"
        }
    }
} | ConvertTo-Json -Depth 5

Set-Content -Path "D:\ProjectP\ScarCord\latest.json" -Value $latestJson

# 5. Git Commit & Push
Write-Host "Committing changes and pushing to Git..." -ForegroundColor Yellow
$Msg = if ($CommitMessage) { $CommitMessage } else { "chore(release): v$Version" }
git add .
git commit -m $Msg
git push origin master

# 6. Upload release assets to GitHub
Write-Host "Creating GitHub release v$Version with OTA updater assets..." -ForegroundColor Green

$Assets = @(
    "D:\ProjectP\ScarCord\src-tauri\target\release\scarcord.exe",
    "D:\ProjectP\ScarCord\latest.json"
)

# Include installer assets for current version only
if (Test-Path "D:\ProjectP\ScarCord\src-tauri\target\release\bundle\nsis") {
    Get-ChildItem "D:\ProjectP\ScarCord\src-tauri\target\release\bundle\nsis\*$Version*" | ForEach-Object {
        $Assets += $_.FullName
    }
}
if (Test-Path "D:\ProjectP\ScarCord\src-tauri\target\release\bundle\msi") {
    Get-ChildItem "D:\ProjectP\ScarCord\src-tauri\target\release\bundle\msi\*$Version*" | ForEach-Object {
        $Assets += $_.FullName
    }
}
if (Test-Path "D:\ProjectP\ScarCord\TrulyScarlet.cer") {
    $Assets += "D:\ProjectP\ScarCord\TrulyScarlet.cer"
}
if (Test-Path "D:\ProjectP\ScarCord\install-certificate.ps1") {
    $Assets += "D:\ProjectP\ScarCord\install-certificate.ps1"
}

gh release create "v$Version" @Assets --title "ScarCord v$Version" --notes "ScarCord v$Version - OTA Auto-Updating release"

Write-Host "=== Done! Release v$Version is live on GitHub! ===" -ForegroundColor Green
