Add-Type -AssemblyName System.Drawing

$srcPath = "c:\Users\vboxuser\Desktop\ETDPicker\logo.jpg"
$assetsDir = "c:\Users\vboxuser\Desktop\ETDPicker\assets"
$icoPath = Join-Path $assetsDir "app_icon.ico"
$pngPath = Join-Path $assetsDir "app_icon.png"

$srcBmp = [System.Drawing.Image]::FromFile($srcPath)
$srcBmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

$sizes = @(16, 24, 32, 48, 64, 128, 256)
$entries = @()

foreach ($size in $sizes) {
    $resized = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($resized)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    
    $g.DrawImage($srcBmp, 0, 0, $size, $size)
    $g.Dispose()

    if ($size -eq 256) {
        # 256x256 can be PNG format
        $ms = New-Object System.IO.MemoryStream
        $resized.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $bytes = $ms.ToArray()
        $ms.Dispose()

        $entries += @{
            Width = 0 # 0 means 256
            Height = 0
            ColorCount = 0
            Planes = 1
            BitCount = 32
            Bytes = $bytes
        }
    } else {
        # Sizes < 256 use pure DIB (BITMAPINFOHEADER + BGRA pixels bottom-to-top + AND mask)
        $ms = New-Object System.IO.MemoryStream
        $bw = New-Object System.IO.BinaryWriter $ms

        $andRowBytes = [int][Math]::Ceiling($size / 32.0) * 4
        $andMaskSize = $andRowBytes * $size
        $xorSize = $size * $size * 4

        # BITMAPINFOHEADER (40 bytes)
        $bw.Write([UInt32]40)              # biSize
        $bw.Write([Int32]$size)            # biWidth
        $bw.Write([Int32]($size * 2))      # biHeight (XOR + AND)
        $bw.Write([UInt16]1)               # biPlanes
        $bw.Write([UInt16]32)              # biBitCount
        $bw.Write([UInt32]0)               # biCompression (BI_RGB)
        $bw.Write([UInt32]($xorSize + $andMaskSize)) # biSizeImage
        $bw.Write([Int32]0)                # biXPelsPerMeter
        $bw.Write([Int32]0)                # biYPelsPerMeter
        $bw.Write([UInt32]0)               # biClrUsed
        $bw.Write([UInt32]0)               # biClrImportant

        # XOR image (BGRA bottom to top)
        for ($y = $size - 1; $y -ge 0; $y--) {
            for ($x = 0; $x -lt $size; $x++) {
                $pixel = $resized.GetPixel($x, $y)
                $bw.Write([byte]$pixel.B)
                $bw.Write([byte]$pixel.G)
                $bw.Write([byte]$pixel.R)
                $bw.Write([byte]$pixel.A)
            }
        }

        # AND mask (1 bit per pixel, bottom to top, 0 = opaque)
        for ($y = $size - 1; $y -ge 0; $y--) {
            $rowBits = New-Object byte[] $andRowBytes
            for ($x = 0; $x -lt $size; $x++) {
                $pixel = $resized.GetPixel($x, $y)
                if ($pixel.A -eq 0) {
                    $byteIdx = [int]($x / 8)
                    $bitIdx = 7 - ($x % 8)
                    $rowBits[$byteIdx] = $rowBits[$byteIdx] -bor (1 -shl $bitIdx)
                }
            }
            $bw.Write($rowBits)
        }

        $bw.Flush()
        $bytes = $ms.ToArray()
        $bw.Close()
        $ms.Dispose()

        $entries += @{
            Width = [byte]$size
            Height = [byte]$size
            ColorCount = 0
            Planes = 1
            BitCount = 32
            Bytes = $bytes
        }
    }

    $resized.Dispose()
}

$srcBmp.Dispose()

# Now write complete ICO file
$fs = [System.IO.File]::Create($icoPath)
$bw = New-Object System.IO.BinaryWriter $fs

# ICO Header
$bw.Write([UInt16]0) # Reserved
$bw.Write([UInt16]1) # Type (1 = ICO)
$bw.Write([UInt16]$entries.Count)

$offset = 6 + ($entries.Count * 16)

# Directory entries
foreach ($entry in $entries) {
    $bw.Write([byte]$entry.Width)
    $bw.Write([byte]$entry.Height)
    $bw.Write([byte]$entry.ColorCount)
    $bw.Write([byte]0) # Reserved
    $bw.Write([UInt16]$entry.Planes)
    $bw.Write([UInt16]$entry.BitCount)
    $bw.Write([UInt32]$entry.Bytes.Length)
    $bw.Write([UInt32]$offset)
    $offset += $entry.Bytes.Length
}

# Image Data
foreach ($entry in $entries) {
    $bw.Write($entry.Bytes)
}

$bw.Flush()
$bw.Close()
$fs.Close()

Write-Host "Perfect DIB Windows ICO generated at $icoPath ($(Get-Item $icoPath | Select-Object -ExpandProperty Length) bytes)"
