Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$outDir = Join-Path $PSScriptRoot "..\packs\core\textures\blocks"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

function Save-Png {
    param(
        [string]$Path,
        [scriptblock]$Pixel
    )

    $size = 16
    $bitmap = [System.Drawing.Bitmap]::new($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    try {
        for ($y = 0; $y -lt $size; $y++) {
            for ($x = 0; $x -lt $size; $x++) {
                $rgba = & $Pixel $x $y
                $color = [System.Drawing.Color]::FromArgb($rgba[3], $rgba[0], $rgba[1], $rgba[2])
                $bitmap.SetPixel($x, $y, $color)
            }
        }
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    }
    finally {
        $bitmap.Dispose()
    }
}

Save-Png (Join-Path $outDir "grass_top_albedo.png") {
    param($x, $y)
    $checker = (($x * 13 + $y * 7 + (($x -band 3) * 11)) % 31)
    $blade = if ((($x + $y * 2) % 5) -eq 0) { 20 } else { 0 }
    $r = 54 + ($checker % 18)
    $g = 132 + $blade + ($checker % 22)
    $b = 44 + ($checker % 16)
    @($r, $g, $b, 255)
}

Save-Png (Join-Path $outDir "grass_top_normal.png") {
    param($x, $y)
    $dx = if ((($x + $y) % 4) -eq 0) { 8 } else { -4 }
    $dy = if ((($x * 2 + $y) % 5) -eq 0) { -7 } else { 3 }
    $r = 128 + $dx
    $g = 128 + $dy
    @($r, $g, 238, 255)
}

Save-Png (Join-Path $outDir "grass_top_roughness.png") {
    param($x, $y)
    $v = 205 + (($x * 9 + $y * 5) % 28)
    @($v, $v, $v, 255)
}

Write-Host "Wrote grass_top PNG set to $outDir"
