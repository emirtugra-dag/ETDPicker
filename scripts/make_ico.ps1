Add-Type -AssemblyName System.Drawing

$srcPath = "c:\Users\vboxuser\Desktop\ETDPicker\logo.jpg"
$assetsDir = "c:\Users\vboxuser\Desktop\ETDPicker\assets"
if (!(Test-Path $assetsDir)) {
    New-Item -ItemType Directory -Path $assetsDir | Out-Null
}
$icoPath = Join-Path $assetsDir "app_icon.ico"
$pngPath = Join-Path $assetsDir "app_icon.png"

$srcBmp = [System.Drawing.Image]::FromFile($srcPath)

# Save PNG version for UI displays
$srcBmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

# Target sizes for multi-resolution Windows icon
$sizes = @(16, 24, 32, 48, 64, 128, 256)

$pngStreams = @()

foreach ($size in $sizes) {
    $resized = New-Object System.Drawing.Bitmap $size, $size
    $g = [System.Drawing.Graphics]::FromImage($resized)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    
    $g.DrawImage($srcBmp, 0, 0, $size, $size)
    $g.Dispose()
    
    $ms = New-Object System.IO.MemoryStream
    $resized.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $resized.Dispose()
    
    $pngStreams += @{
        Size = $size
        Bytes = $ms.ToArray()
    }
    $ms.Dispose()
}

$srcBmp.Dispose()

# Write ICO format
# Header: 2 bytes reserved (0), 2 bytes type (1=ICO), 2 bytes image count
$fs = [System.IO.File]::OpenWrite($icoPath)
$bw = New-Object System.IO.BinaryWriter $fs

$bw.Write([UInt16]0)
$bw.Write([UInt16]1)
$bw.Write([UInt16]$pngStreams.Count)

$offset = 6 + ($pngStreams.Count * 16)

foreach ($entry in $pngStreams) {
    $w = if ($entry.Size -ge 256) { 0 } else { [byte]$entry.Size }
    $h = if ($entry.Size -ge 256) { 0 } else { [byte]$entry.Size }
    $bw.Write([byte]$w)
    $bw.Write([byte]$h)
    $bw.Write([byte]0) # Color count (0 = no palette / 256+ colors)
    $bw.Write([byte]0) # Reserved
    $bw.Write([UInt16]1) # Color planes
    $bw.Write([UInt16]32) # Bits per pixel
    $bw.Write([UInt32]$entry.Bytes.Length) # Image size in bytes
    $bw.Write([UInt32]$offset) # Offset to image data
    
    $offset += $entry.Bytes.Length
}

foreach ($entry in $pngStreams) {
    $bw.Write($entry.Bytes)
}

$bw.Flush()
$bw.Close()
$fs.Close()

Write-Host "app_icon.ico and app_icon.png generated successfully at $assetsDir"
