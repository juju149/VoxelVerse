<#
.SYNOPSIS
    Generate procedural foliage textures (albedo + normal + roughness) for the
    leaf voxel blocks.

.DESCRIPTION
    Each leaf preset gets three PNGs at the chosen resolution:
        <out>/<preset>/<preset>_all_albedo.png
        <out>/<preset>/<preset>_all_normal.png
        <out>/<preset>/<preset>_all_roughness.png

    The pixel-level work is JITed via Add-Type so producing 256x256 textures
    for every preset takes about a second instead of several minutes.

    Albedo: stacked Worley (cellular) noise mapped through a 3-stop palette,
            plus a fine speckle for organic micro-detail.
    Normal: tangent-space normal derived from the albedo luminance gradient.
    Roughness: high mid-grey with mild noise so specular looks foliage-y.

.EXAMPLE
    pwsh -File tools/generate_leaf_textures.ps1

.EXAMPLE
    pwsh -File tools/generate_leaf_textures.ps1 -Resolution 128 -Presets oak_leaves,birch_leaves
#>
[CmdletBinding()]
param(
    [int] $Resolution = 256,
    [string] $OutputRoot,
    [string[]] $Presets,
    [int] $Seed = 7919
)

if (-not $OutputRoot) {
    $scriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }
    $OutputRoot = Join-Path $scriptDir '..\packs\core\textures\blocks'
}
$OutputRoot = (Resolve-Path $OutputRoot -ErrorAction SilentlyContinue)
if (-not $OutputRoot) {
    throw 'Could not resolve output root.'
}

Add-Type -AssemblyName System.Drawing

# --- Compiled pixel kernel ----------------------------------------------------

Add-Type -TypeDefinition @'
using System;
public static class LeafKernel
{
    static double Hash(int x, int y, int salt)
    {
        long h = (long)x * 374761393L ^ (long)y * 668265263L ^ (long)salt * 1274126177L;
        h = (h ^ (h >> 13)) * 1274126177L;
        h ^= (h >> 16);
        return ((uint)h) / 4294967295.0;
    }

    static double Worley(double sx, double sy, double freq, int salt)
    {
        double u = sx * freq;
        double v = sy * freq;
        int cx = (int)Math.Floor(u);
        int cy = (int)Math.Floor(v);
        double best = 9.0;
        for (int oy = -1; oy <= 1; oy++)
        for (int ox = -1; ox <= 1; ox++)
        {
            int ncx = cx + ox;
            int ncy = cy + oy;
            double jx = Hash(ncx, ncy, salt);
            double jy = Hash(ncx, ncy, salt + 17);
            double dx = (ncx + jx) - u;
            double dy = (ncy + jy) - v;
            double d  = dx * dx + dy * dy;
            if (d < best) best = d;
        }
        double r = Math.Sqrt(best);
        if (r > 1.0) r = 1.0;
        return r;
    }

    static double Lum(int x, int y, int size, int salt)
    {
        double u = (double)x / size;
        double v = (double)y / size;
        double w1 = Worley(u, v,  6.0, salt);
        double w2 = Worley(u, v, 12.0, salt + 101);
        double w3 = Worley(u, v, 24.0, salt + 211);
        double cluster = 0.55 * (1.0 - w1) + 0.30 * (1.0 - w2) + 0.15 * (1.0 - w3);
        double speck = Hash(x, y, salt + 313) * 0.10 - 0.05;
        double t = cluster + speck;
        if (t < 0.0) t = 0.0;
        if (t > 1.0) t = 1.0;
        return t;
    }

    static byte Clamp255(double v)
    {
        if (v <= 0.0) return 0;
        if (v >= 255.0) return 255;
        return (byte)Math.Round(v);
    }

    public static void Generate(
        int size, int salt,
        int[] paletteRGB,         // length 9: r0,g0,b0, r1,g1,b1, r2,g2,b2
        byte[] albedo,            // size*size*4 BGRA
        byte[] normal,
        byte[] rough)
    {
        double[] lum = new double[size * size];
        for (int y = 0; y < size; y++)
            for (int x = 0; x < size; x++)
                lum[y * size + x] = Lum(x, y, size, salt);

        // Albedo
        for (int y = 0; y < size; y++)
        {
            int row = y * size * 4;
            for (int x = 0; x < size; x++)
            {
                double t = lum[y * size + x];
                int    a, b;
                double k;
                if (t < 0.5) { a = 0; b = 1; k = t / 0.5; }
                else         { a = 1; b = 2; k = (t - 0.5) / 0.5; }
                int r = (int)Math.Round(paletteRGB[a*3+0] + (paletteRGB[b*3+0] - paletteRGB[a*3+0]) * k);
                int g = (int)Math.Round(paletteRGB[a*3+1] + (paletteRGB[b*3+1] - paletteRGB[a*3+1]) * k);
                int bl= (int)Math.Round(paletteRGB[a*3+2] + (paletteRGB[b*3+2] - paletteRGB[a*3+2]) * k);
                int i = row + x * 4;
                albedo[i    ] = (byte)bl;
                albedo[i + 1] = (byte)g;
                albedo[i + 2] = (byte)r;
                albedo[i + 3] = 255;
            }
        }

        // Normal from luminance gradient
        double strength = 1.6;
        for (int y = 0; y < size; y++)
        {
            int yp = (y == 0) ? 0 : y - 1;
            int yn = (y == size - 1) ? size - 1 : y + 1;
            int row = y * size * 4;
            for (int x = 0; x < size; x++)
            {
                int xp = (x == 0) ? 0 : x - 1;
                int xn = (x == size - 1) ? size - 1 : x + 1;
                double dx = (lum[y * size + xn] - lum[y * size + xp]) * strength;
                double dy = (lum[yn * size + x] - lum[yp * size + x]) * strength;
                double nxv = -dx; if (nxv < -1) nxv = -1; if (nxv > 1) nxv = 1;
                double nyv = -dy; if (nyv < -1) nyv = -1; if (nyv > 1) nyv = 1;
                byte nx = Clamp255(nxv * 127.5 + 127.5);
                byte ny = Clamp255(nyv * 127.5 + 127.5);
                int i = row + x * 4;
                normal[i    ] = 255;
                normal[i + 1] = ny;
                normal[i + 2] = nx;
                normal[i + 3] = 255;
            }
        }

        // Roughness
        for (int y = 0; y < size; y++)
        {
            int row = y * size * 4;
            for (int x = 0; x < size; x++)
            {
                double r = 0.78 + (Hash(x, y, salt + 991) * 0.10 - 0.05);
                byte v = Clamp255(r * 255.0);
                int i = row + x * 4;
                rough[i    ] = v;
                rough[i + 1] = v;
                rough[i + 2] = v;
                rough[i + 3] = 255;
            }
        }
    }
}
'@ -Language CSharp

# --- Palettes -----------------------------------------------------------------

$leafPresets = [ordered]@{
    oak_leaves      = @( @(34,56,18),  @(63,118,38),  @(140,176,72) )
    birch_leaves    = @( @(58,90,24),  @(120,160,56), @(192,212,108) )
    spruce_leaves   = @( @(20,42,18),  @(40,82,32),   @(96,140,68) )
    jungle_leaves   = @( @(24,68,14),  @(48,124,28),  @(132,200,80) )
    acacia_leaves   = @( @(58,80,22),  @(126,158,52), @(208,212,98) )
    dark_oak_leaves = @( @(20,38,14),  @(46,82,30),   @(110,148,60) )
    mangrove_leaves = @( @(24,72,30),  @(54,128,58),  @(140,196,108) )
    azalea_leaves   = @( @(40,68,28),  @(96,140,60),  @(176,196,118) )
}

if ($Presets) {
    $selected = [ordered]@{}
    foreach ($k in $Presets) {
        if ($leafPresets.Contains($k)) { $selected[$k] = $leafPresets[$k] }
        else { Write-Warning "Unknown preset '$k' (skipped)." }
    }
    $leafPresets = $selected
}

# --- Bitmap save helper -------------------------------------------------------

function Save-Bgra32 {
    param([int] $Size, [byte[]] $Bytes, [string] $Path)
    $bmp = New-Object System.Drawing.Bitmap $Size, $Size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    try {
        $rect = [System.Drawing.Rectangle]::new(0, 0, $Size, $Size)
        $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::WriteOnly,
                              [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        try {
            [System.Runtime.InteropServices.Marshal]::Copy($Bytes, 0, $data.Scan0, $Bytes.Length)
        } finally {
            $bmp.UnlockBits($data)
        }
        $tmp = "$Path.tmp"
        $bmp.Save($tmp, [System.Drawing.Imaging.ImageFormat]::Png)
        Move-Item -Path $tmp -Destination $Path -Force
    } finally {
        $bmp.Dispose()
    }
}

# --- Main ---------------------------------------------------------------------

if (-not (Test-Path $OutputRoot)) {
    throw "Output root does not exist: $OutputRoot"
}

Write-Host "Generating leaf textures @ ${Resolution}x${Resolution}"
Write-Host "Output: $OutputRoot"

$pixels = $Resolution * $Resolution * 4
$index = 0
foreach ($preset in $leafPresets.Keys) {
    $palette = $leafPresets[$preset]
    $dir = Join-Path $OutputRoot $preset
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }

    $flat = New-Object 'int[]' 9
    for ($k = 0; $k -lt 3; $k++) {
        $flat[$k * 3 + 0] = $palette[$k][0]
        $flat[$k * 3 + 1] = $palette[$k][1]
        $flat[$k * 3 + 2] = $palette[$k][2]
    }

    $albedo = New-Object byte[] $pixels
    $normal = New-Object byte[] $pixels
    $rough  = New-Object byte[] $pixels

    $salt = $Seed + $index * 137
    Write-Host "  $preset (salt=$salt) ..."
    [LeafKernel]::Generate($Resolution, $salt, $flat, $albedo, $normal, $rough)

    Save-Bgra32 $Resolution $albedo (Join-Path $dir "${preset}_all_albedo.png")
    Save-Bgra32 $Resolution $normal (Join-Path $dir "${preset}_all_normal.png")
    Save-Bgra32 $Resolution $rough  (Join-Path $dir "${preset}_all_roughness.png")
    $index++
}
Write-Host 'Done.'
