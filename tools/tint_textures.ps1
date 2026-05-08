<#
.SYNOPSIS
    Bake a multiplicative RGB tint into texture PNGs.

.DESCRIPTION
    Minecraft-style grass and leaf textures are authored in grayscale and
    tinted at runtime with biome-specific colors.  This engine does not yet
    apply that biome tint, so we bake a fixed green tint into the source
    PNGs.  Each pixel's RGB is multiplied by the tint, alpha is preserved.

    Run from the project root:
        pwsh -File tools/tint_textures.ps1

    Or with custom inputs:
        pwsh -File tools/tint_textures.ps1 -Tint 0.45,0.78,0.32 -Targets grass,leaves

.PARAMETER Tint
    [r, g, b] multiplier in 0..1.

.PARAMETER Targets
    Subset of preset groups to tint: grass, leaves.

.PARAMETER PackRoot
    Path to the texture pack (defaults to packs/core/textures/blocks).

.PARAMETER Backup
    If set, copies the original PNG to *.original.png before overwriting.
#>
[CmdletBinding()]
param(
    [float[]] $Tint = @(0.45, 0.78, 0.32),
    [ValidateSet('grass', 'leaves')]
    [string[]] $Targets = @('grass', 'leaves'),
    [string] $PackRoot = (Join-Path $PSScriptRoot '..\packs\core\textures\blocks'),
    [switch] $Backup
)

if ($Tint.Count -ne 3) {
    throw 'Tint must be exactly 3 floats: r, g, b.'
}

Add-Type -AssemblyName System.Drawing

# Map each preset to the file-name patterns whose albedo PNGs should be tinted.
$presets = @{
    grass  = @('short_grass_all_albedo', 'tall_grass_*_albedo', 'short_dry_grass_*_albedo')
    leaves = @(
        'oak_leaves_all_albedo',
        'birch_leaves_all_albedo',
        'spruce_leaves_all_albedo',
        'jungle_leaves_all_albedo',
        'acacia_leaves_all_albedo',
        'dark_oak_leaves_all_albedo',
        'mangrove_leaves_all_albedo',
        'azalea_leaves_all_albedo'
    )
}

function Get-TargetFiles {
    param([string] $Root, [string[]] $Patterns)
    $files = @()
    foreach ($pattern in $Patterns) {
        $files += Get-ChildItem -Path $Root -Recurse -Filter "$pattern.png" -ErrorAction SilentlyContinue
    }
    return $files | Sort-Object FullName -Unique
}

function Invoke-TintImage {
    param(
        [string] $Path,
        [float] $R,
        [float] $G,
        [float] $B,
        [bool]  $MakeBackup
    )

    $bmp = [System.Drawing.Bitmap]::FromFile($Path)
    try {
        $rect = [System.Drawing.Rectangle]::new(0, 0, $bmp.Width, $bmp.Height)
        $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadWrite,
                              [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        try {
            $stride = $data.Stride
            $bytes = New-Object byte[] ($stride * $bmp.Height)
            [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)

            for ($y = 0; $y -lt $bmp.Height; $y++) {
                $row = $y * $stride
                for ($x = 0; $x -lt $bmp.Width; $x++) {
                    $i = $row + $x * 4
                    # Pixel layout in Format32bppArgb on little-endian: B, G, R, A.
                    $bytes[$i]     = [byte]([math]::Min(255, [math]::Round($bytes[$i]     * $B)))
                    $bytes[$i + 1] = [byte]([math]::Min(255, [math]::Round($bytes[$i + 1] * $G)))
                    $bytes[$i + 2] = [byte]([math]::Min(255, [math]::Round($bytes[$i + 2] * $R)))
                    # Alpha (i+3) untouched.
                }
            }

            [System.Runtime.InteropServices.Marshal]::Copy($bytes, 0, $data.Scan0, $bytes.Length)
        } finally {
            $bmp.UnlockBits($data)
        }

        if ($MakeBackup) {
            $backup = [System.IO.Path]::ChangeExtension($Path, '.original.png')
            if (-not (Test-Path $backup)) {
                Copy-Item -Path $Path -Destination $backup -Force
            }
        }

        # Save through a temp file then atomically replace, avoids GDI+ file lock.
        $tmp = "$Path.tmp"
        $bmp.Save($tmp, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
        Move-Item -Path $tmp -Destination $Path -Force
    } catch {
        $bmp.Dispose()
        throw
    }
}

if (-not (Test-Path $PackRoot)) {
    throw "Pack root not found: $PackRoot"
}

$patterns = @()
foreach ($t in $Targets) { $patterns += $presets[$t] }

$files = Get-TargetFiles -Root $PackRoot -Patterns $patterns
if ($files.Count -eq 0) {
    Write-Warning "No matching PNGs under $PackRoot for targets: $($Targets -join ', ')"
    return
}

Write-Host "Tint = ($($Tint[0]), $($Tint[1]), $($Tint[2])) -- $($files.Count) file(s)"
foreach ($f in $files) {
    Write-Host "  -> $($f.FullName)"
    Invoke-TintImage -Path $f.FullName -R $Tint[0] -G $Tint[1] -B $Tint[2] -MakeBackup:$Backup
}
Write-Host 'Done.'
