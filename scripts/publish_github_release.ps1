[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "Getting GitHub credentials from Git Credential Manager..."
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "git.exe"
$psi.Arguments = "credential fill"
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$p = [System.Diagnostics.Process]::Start($psi)
$p.StandardInput.WriteLine("protocol=https")
$p.StandardInput.WriteLine("host=github.com")
$p.StandardInput.WriteLine("")
$credOutput = $p.StandardOutput.ReadToEnd()
$p.WaitForExit()

$token = ""
foreach ($line in ($credOutput -split "`r?`n")) {
    if ($line.StartsWith("password=")) {
        $token = $line.Substring("password=".Length).Trim()
    }
}

if (-not $token) {
    Write-Error "Could not retrieve GitHub token from credential manager."
    exit 1
}

$repo = "emirtugra-dag/ETDPicker"
$tag = "v1.0.0"
$headers = @{
    "Authorization" = "token $token"
    "Accept" = "application/vnd.github.v3+json"
    "User-Agent" = "ETDPicker-Publisher"
}

Write-Host "Checking if release '$tag' already exists..."
$existingRelease = $null
try {
    $existingRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/tags/$tag" -Headers $headers -Method Get -ErrorAction Stop
} catch {
    # Release doesn't exist yet
}

if ($existingRelease) {
    Write-Host "Deleting existing release id $($existingRelease.id)..."
    Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/$($existingRelease.id)" -Headers $headers -Method Delete
    Start-Sleep -Seconds 1
}

Write-Host "Creating new GitHub Release for '$tag'..."
$releaseBody = @"
# 🎯 ETDPicker v1.0.0

Ultra-lightweight, high-performance screen color picker for Windows 10 & 11 built with pure Rust and native Win32.

### 📦 Release Assets
- **🚀 `ETDPicker_Portable.exe`** - Standalone portable executable. No installation required.
- **📦 `ETDPicker_Setup.exe`** - Clean installer wizard with startup, desktop, and start menu shortcuts.

### ✨ Highlights
- **Lightning Fast (<10 MB RAM)**: Minimal memory footprint.
- **Precision Magnifier**: Press `Alt + P` for live 8x pixel loupe with isolated arrow key navigation.
- **True Foreground Activation**: Instantly opens in front of all open windows upon picking.
- **All Color Formats**: Instant HEX, RGB, HSL, CMYK conversion + 10-slot dynamic history.
- **Multi-Language**: Türkçe (TR) & English (EN) support.
- **Authenticode Signed**: Digitally signed binaries for security.
"@

$releasePayload = @{
    tag_name = $tag
    target_commitish = "main"
    name = "ETDPicker v1.0.0 - Official Release"
    body = $releaseBody
    draft = $false
    prerelease = $false
} | ConvertTo-Json

$newRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases" -Headers $headers -Method Post -Body $releasePayload -ContentType "application/json; charset=utf-8"
$releaseId = $newRelease.id
Write-Host "Created release id: $releaseId"

# Upload assets
$assets = @(
    @{ Path = "dist\ETDPicker_Portable.exe"; Name = "ETDPicker_Portable.exe" },
    @{ Path = "dist\ETDPicker_Setup.exe"; Name = "ETDPicker_Setup.exe" }
)

foreach ($asset in $assets) {
    if (Test-Path $asset.Path) {
        Write-Host "Uploading $($asset.Name)..."
        $fileBytes = [System.IO.File]::ReadAllBytes((Resolve-Path $asset.Path).Path)
        $uploadUri = "https://uploads.github.com/repos/$repo/releases/$releaseId/assets?name=$($asset.Name)"
        
        $uploadHeaders = @{
            "Authorization" = "token $token"
            "Accept" = "application/vnd.github.v3+json"
            "Content-Type" = "application/octet-stream"
            "User-Agent" = "ETDPicker-Publisher"
        }

        $res = Invoke-RestMethod -Uri $uploadUri -Headers $uploadHeaders -Method Post -Body $fileBytes
        Write-Host "Successfully uploaded $($asset.Name) (Size: $([math]::Round($fileBytes.Length / 1KB, 2)) KB)"
    } else {
        Write-Error "File not found: $($asset.Path)"
    }
}

Write-Host "`n🎉 All release assets uploaded successfully to https://github.com/$repo/releases/tag/$tag"
