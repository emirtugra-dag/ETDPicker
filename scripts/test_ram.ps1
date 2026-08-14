$proc = Start-Process -FilePath "c:\Users\vboxuser\Desktop\ETDPicker\dist\ETDPicker_Portable.exe" -PassThru
Start-Sleep -Milliseconds 1200
$p = Get-Process -Id $proc.Id
$ramMB = [math]::Round($p.WorkingSet64 / 1MB, 2)
$privateMB = [math]::Round($p.PrivateMemorySize64 / 1MB, 2)
Write-Host "=== ETDPicker RAM Performance Test ==="
Write-Host "Process Name: $($p.ProcessName)"
Write-Host "Process ID: $($p.Id)"
Write-Host "Working Set (RAM): $ramMB MB"
Write-Host "Private Memory: $privateMB MB"
Stop-Process -Id $proc.Id -Force
Write-Host "Process terminated cleanly."
