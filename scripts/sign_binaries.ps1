# Build release binaries
Write-Host "Building release binaries..."
& "C:\Users\vboxuser\.cargo\bin\cargo.exe" build --release
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
$certSubject = "CN=Emir Tuğra Dağ, O=Emir Tuğra Dağ, C=TR"
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -like "*Emir Tuğra Dağ*" -and $_.HasPrivateKey } | Select-Object -First 1

if (-not $cert) {
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject $certSubject `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -NotAfter (Get-Date).AddYears(10) `
        -FriendlyName "Emir Tuğra Dağ Code Signing Certificate"
    
    # Also trust in Root / TrustedPublisher for CurrentUser
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
