param()

$CertPath = Join-Path $PSScriptRoot "TrulyScarlet.cer"

if (-not (Test-Path -LiteralPath $CertPath)) {
    Write-Error "TrulyScarlet.cer not found in $PSScriptRoot."
    exit 1
}

Write-Host "Installing TrulyScarlet certificate into Trusted Root and Trusted Publisher stores..." -ForegroundColor Cyan

try {
    Import-Certificate -FilePath $CertPath -CertStoreLocation "Cert:\CurrentUser\Root" | Out-Null
    Import-Certificate -FilePath $CertPath -CertStoreLocation "Cert:\CurrentUser\TrustedPublisher" | Out-Null
    Write-Host "Done! TrulyScarlet is now a trusted publisher on this computer." -ForegroundColor Green
    Write-Host "All ScarCord installers and updates will now show 'Verified Publisher: TrulyScarlet'." -ForegroundColor Green
} catch {
    Write-Error "Failed to install certificate: $_"
}
