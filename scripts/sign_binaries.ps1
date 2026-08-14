[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

# Build release binaries with resource compiler (icon & metadata)
Write-Host "Building release binaries with embedded icons..."
$env:PATH = "C:\Users\vboxuser\Desktop\mingw64\bin;" + $env:PATH
& "C:\Users\vboxuser\.cargo\bin\cargo.exe" build --release --package etd-picker
& "C:\Users\vboxuser\.cargo\bin\cargo.exe" build --release --package etd-installer

# Prepare dist directory
if (-not (Test-Path ".\dist")) {
    New-Item -ItemType Directory -Path ".\dist" | Out-Null
}

Copy-Item ".\target\release\etd-picker.exe" ".\dist\ETDPicker_Portable.exe" -Force
Copy-Item ".\target\release\etd-installer.exe" ".\dist\ETDPicker_Setup.exe" -Force

if (Test-Path ".\dist\ETDPicker.exe") {
    Remove-Item ".\dist\ETDPicker.exe" -Force
}

# Certificate creation and code signing
Write-Host "Setting up Code Signing Certificate for 'Emir Tuğra Dağ'..."
$certSubject = "CN=Emir Tuğra Dağ, O=ETDPicker, C=TR"
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq $certSubject -and $_.HasPrivateKey } | Select-Object -First 1

if (-not $cert) {
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject $certSubject `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -NotAfter (Get-Date).AddYears(10) `
        -FriendlyName "Emir Tuğra Dağ - ETDPicker Code Signing"
    
    $rootStore = New-Object System.Security.Cryptography.X509Certificates.X509Store("Root", "CurrentUser")
    $rootStore.Open("ReadWrite")
    $rootStore.Add($cert)
    $rootStore.Close()

    $pubStore = New-Object System.Security.Cryptography.X509Certificates.X509Store("TrustedPublisher", "CurrentUser")
    $pubStore.Open("ReadWrite")
    $pubStore.Add($cert)
    $pubStore.Close()
}

Write-Host "Signing ETDPicker_Portable.exe..."
$sig1 = Set-AuthenticodeSignature -FilePath ".\dist\ETDPicker_Portable.exe" -Certificate $cert -TimestampServer "http://timestamp.digicert.com" -ErrorAction SilentlyContinue
if ($sig1.Status -ne "Valid") {
    $sig1 = Set-AuthenticodeSignature -FilePath ".\dist\ETDPicker_Portable.exe" -Certificate $cert
}

Write-Host "Signing ETDPicker_Setup.exe..."
$sig2 = Set-AuthenticodeSignature -FilePath ".\dist\ETDPicker_Setup.exe" -Certificate $cert -TimestampServer "http://timestamp.digicert.com" -ErrorAction SilentlyContinue
if ($sig2.Status -ne "Valid") {
    $sig2 = Set-AuthenticodeSignature -FilePath ".\dist\ETDPicker_Setup.exe" -Certificate $cert
}

Write-Host "`n=== Final Files in dist/ ==="
Get-ChildItem ".\dist\*.exe" | Select-Object Name, Length

Write-Host "`n=== Authenticode Signatures ==="
Get-AuthenticodeSignature ".\dist\*.exe" | Format-List Path, Status, StatusMessage, SignerCertificate
