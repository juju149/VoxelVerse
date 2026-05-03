param(
    [string]$Matrix = "tools\voxelverse\block_visual_presets\matrices\terrain_blocks.matrix.ron",
    [string]$OutputRoot = "assets\packs\voxelverse_core\defs\blocks\generated\matrix",
    [string]$Family = "all",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

function Find-VoxelVerseRoot {
    $dir = (Get-Location).Path

    while ($true) {
        if ((Test-Path (Join-Path $dir "Cargo.toml")) -and
            (Test-Path (Join-Path $dir "assets"))) {
            return (Resolve-Path $dir).Path
        }

        $parent = Split-Path $dir -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $dir) {
            throw "Repo root not found."
        }

        $dir = $parent
    }
}

function Get-Field {
    param(
        [Parameter(Mandatory = $true)][string]$ObjectText,
        [Parameter(Mandatory = $true)][string]$Name,
        [string]$Default = ""
    )

    $pattern = "$Name\s*:\s*""([^""]*)"""
    $match = [regex]::Match($ObjectText, $pattern)

    if ($match.Success) {
        return $match.Groups[1].Value
    }

    return $Default
}

function Get-NumberField {
    param(
        [Parameter(Mandatory = $true)][string]$ObjectText,
        [Parameter(Mandatory = $true)][string]$Name,
        [string]$Default = "0"
    )

    $pattern = "$Name\s*:\s*([0-9]+(?:\.[0-9]+)?)"
    $match = [regex]::Match($ObjectText, $pattern)

    if ($match.Success) {
        return $match.Groups[1].Value
    }

    return $Default
}

function Invoke-PresetGenerator {
    param(
        [Parameter(Mandatory = $true)][string]$Generator,
        [Parameter(Mandatory = $true)][string]$Preset,
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$DisplayKey,
        [Parameter(Mandatory = $true)][string]$Output,
        [Parameter(Mandatory = $true)][string]$BaseColor,
        [Parameter(Mandatory = $true)][string]$TopColor,
        [Parameter(Mandatory = $true)][string]$SideColor,
        [Parameter(Mandatory = $true)][string]$BottomColor,
        [Parameter(Mandatory = $true)][string]$Tool,
        [Parameter(Mandatory = $true)][string]$SoundMaterial,
        [Parameter(Mandatory = $true)][string]$Hardness,
        [Parameter(Mandatory = $true)][string]$Density,
        [Parameter(Mandatory = $true)][string]$Seed,
        [switch]$Force
    )

    # IMPORTANT:
    # Do not build an array like @("-Preset", $Preset, ...).
    # Array splatting passes values positionally and makes "-Preset" become
    # the value for the Preset parameter. Use hashtable splatting for named params.
    $invokeArgs = @{
        Preset = $Preset
        Name = $Name
        DisplayKey = $DisplayKey
        Output = $Output
        BaseColor = $BaseColor
        TopColor = $TopColor
        SideColor = $SideColor
        BottomColor = $BottomColor
        Tool = $Tool
        SoundMaterial = $SoundMaterial
        Hardness = [double]$Hardness
        Density = [double]$Density
        Seed = [uint32]$Seed
    }

    if ($Force) {
        $invokeArgs["Force"] = $true
    }

    & $Generator @invokeArgs
}

$Root = Find-VoxelVerseRoot
Set-Location $Root

$MatrixPath = Join-Path $Root $Matrix
$Generator = Join-Path $Root "scripts\New-VoxelBlockFromPreset.ps1"
$OutRoot = Join-Path $Root $OutputRoot

if (!(Test-Path $MatrixPath)) {
    throw "Matrix file not found: $MatrixPath"
}

if (!(Test-Path $Generator)) {
    throw "Preset generator missing: $Generator"
}

$source = Get-Content -Path $MatrixPath -Raw
$objects = [regex]::Matches($source, "\((?s:.*?)\)")

$count = 0
$skipped = 0

foreach ($object in $objects) {
    $text = $object.Value

    $familyValue = Get-Field -ObjectText $text -Name "family"
    if ($Family -ne "all" -and $familyValue -ne $Family) {
        continue
    }

    $preset = Get-Field -ObjectText $text -Name "preset"
    $name = Get-Field -ObjectText $text -Name "name"
    $displayKey = Get-Field -ObjectText $text -Name "display_key"
    $base = Get-Field -ObjectText $text -Name "base"
    $top = Get-Field -ObjectText $text -Name "top"
    $side = Get-Field -ObjectText $text -Name "side"
    $bottom = Get-Field -ObjectText $text -Name "bottom"
    $tool = Get-Field -ObjectText $text -Name "tool" -Default "shovel"
    $sound = Get-Field -ObjectText $text -Name "sound" -Default "dirt"
    $hardness = Get-NumberField -ObjectText $text -Name "hardness" -Default "0.6"
    $density = Get-NumberField -ObjectText $text -Name "density" -Default "1.3"
    $seed = Get-NumberField -ObjectText $text -Name "seed" -Default "0"

    if ([string]::IsNullOrWhiteSpace($preset) -or
        [string]::IsNullOrWhiteSpace($name) -or
        [string]::IsNullOrWhiteSpace($displayKey)) {
        $skipped++
        continue
    }

    $familyDir = if ([string]::IsNullOrWhiteSpace($familyValue)) { "misc" } else { $familyValue }
    $output = Join-Path (Join-Path $OutRoot $familyDir) "$name.ron"

    Invoke-PresetGenerator `
        -Generator $Generator `
        -Preset $preset `
        -Name $name `
        -DisplayKey $displayKey `
        -Output $output `
        -BaseColor $base `
        -TopColor $top `
        -SideColor $side `
        -BottomColor $bottom `
        -Tool $tool `
        -SoundMaterial $sound `
        -Hardness $hardness `
        -Density $density `
        -Seed $seed `
        -Force:$Force

    $count++
}

Write-Host "Generated $count blocks from matrix. Skipped $skipped invalid/comment entries." -ForegroundColor Green