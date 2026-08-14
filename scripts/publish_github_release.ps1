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
    # Doesn't exist
}

if ($existingRelease) {
    Write-Host "Deleting existing release id $($existingRelease.id)..."
    Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/$($existingRelease.id)" -Headers $headers -Method Delete
    Start-Sleep -Seconds 1
}

Write-Host "Reading RELEASE_NOTES.md with UTF-8 encoding..."
$notesPath = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Path) "..\docs\RELEASE_NOTES.md"
if (-not (Test-Path $notesPath)) {
    $notesPath = ".\docs\RELEASE_NOTES.md"
}
$releaseBody = [System.IO.File]::ReadAllText((Resolve-Path $notesPath).Path, [System.Text.Encoding]::UTF8)

Add-Type -AssemblyName System.Web.Extensions
$serializer = New-Object System.Web.Script.Serialization.JavaScriptSerializer
$payloadObj = @{
    tag_name = $tag
    target_commitish = "main"
    name = "ETDPicker v1.0.0 - Official Release"
    body = $releaseBody
    draft = $false
    prerelease = $false
}
$json = $serializer.Serialize($payloadObj)

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$tempPayload = Join-Path ([System.IO.Path]::GetTempPath()) "etd_release_payload.json"
[System.IO.File]::WriteAllText($tempPayload, $json, $utf8NoBom)

Write-Host "Creating new GitHub Release for '$tag' via curl..."
$authHeader = "Authorization: token $token"
$resp = curl.exe -s -X POST -H $authHeader -H "Accept: application/vnd.github.v3+json" -H "Content-Type: application/json; charset=utf-8" "https://api.github.com/repos/$repo/releases" --data-binary "@$tempPayload"
Remove-Item $tempPayload -Force

$newRelease = $resp | ConvertFrom-Json
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
