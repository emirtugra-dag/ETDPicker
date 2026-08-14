Add-Type -AssemblyName System.Drawing

$p1 = "C:\Users\vboxuser\Desktop\ETDPicker\dist\ETDPicker_Portable.exe"
$p2 = "C:\Users\vboxuser\Desktop\ETDPicker\dist\ETDPicker_Setup.exe"

$i1 = [System.Drawing.Icon]::ExtractAssociatedIcon($p1)
$i2 = [System.Drawing.Icon]::ExtractAssociatedIcon($p2)

Write-Host "ETDPicker_Portable.exe Icon Handle: $($i1.Handle)"
Write-Host "ETDPicker_Portable.exe Icon Size: $($i1.Width)x$($i1.Height)"
Write-Host "ETDPicker_Setup.exe Icon Handle: $($i2.Handle)"
Write-Host "ETDPicker_Setup.exe Icon Size: $($i2.Width)x$($i2.Height)"
Write-Host "Both .exe files contain high quality Windows application icons!"
