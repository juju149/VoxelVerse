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

function Clamp-Byte {
    param([int]$Value)
    [Math]::Max(0, [Math]::Min(255, $Value))
}

function Save-Material {
    param(
        [string]$Name,
        [int]$R,
        [int]$G,
        [int]$B,
        [int]$Roughness,
        [int]$Noise,
        [int]$NormalStrength
    )

    Save-Png (Join-Path $outDir "$($Name)_albedo.png") {
        param($x, $y)
        $n = (($x * 17 + $y * 31 + (($x -band 3) * 19) + (($y -band 2) * 23)) % ($Noise + 1)) - [int]($Noise / 2)
        $rr = Clamp-Byte ($R + $n)
        $gg = Clamp-Byte ($G + $n)
        $bb = Clamp-Byte ($B + $n)
        @($rr, $gg, $bb, 255)
    }

    Save-Png (Join-Path $outDir "$($Name)_normal.png") {
        param($x, $y)
        $dx = ((($x * 11 + $y * 5) % 9) - 4) * $NormalStrength
        $dy = ((($x * 3 + $y * 13) % 9) - 4) * $NormalStrength
        $rr = Clamp-Byte (128 + $dx)
        $gg = Clamp-Byte (128 + $dy)
        @($rr, $gg, 238, 255)
    }

    Save-Png (Join-Path $outDir "$($Name)_roughness.png") {
        param($x, $y)
        $n = (($x * 7 + $y * 11) % 21) - 10
        $v = Clamp-Byte ($Roughness + $n)
        @($v, $v, $v, 255)
    }
}

Save-Material "core_all" 48 42 42 232 22 1
Save-Material "dirt_all" 128 82 38 218 34 2
Save-Material "grass_top" 56 142 48 222 38 2
Save-Material "grass_side" 82 126 48 224 32 2
Save-Material "gravel_all" 104 102 98 236 44 3
Save-Material "ice_all" 174 220 240 92 24 1
Save-Material "sand_all" 202 184 112 204 26 1
Save-Material "snow_all" 226 236 244 188 18 1
Save-Material "stone_all" 112 112 116 228 30 2

Write-Host "Wrote core block PBR-lite PNG sets to $outDir"
