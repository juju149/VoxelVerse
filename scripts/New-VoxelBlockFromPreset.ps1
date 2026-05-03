param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("natural_soil", "leafy_soil", "mossy_soil", "stone_cells", "wood_rings")]
    [string]$Preset,

    [Parameter(Mandatory = $true)]
    [string]$Name,

    [Parameter(Mandatory = $true)]
    [string]$DisplayKey,

    [Parameter(Mandatory = $true)]
    [string]$Output,

    [string]$BaseColor = "",
    [string]$TopColor = "",
    [string]$SideColor = "",
    [string]$BottomColor = "",
    [uint32]$Seed = 0,

    [string]$Tool = "shovel",
    [string]$SoundMaterial = "",
    [double]$Hardness = 0.6,
    [double]$Density = 1.3,

    [switch]$Force
)

$ErrorActionPreference = "Stop"

function Find-VoxelVerseRoot {
    $dir = (Get-Location).Path
    while ($true) {
        if ((Test-Path (Join-Path $dir "Cargo.toml")) -and
            (Test-Path (Join-Path $dir "tools\voxelverse\block_visual_presets\templates"))) {
            return (Resolve-Path $dir).Path
        }
        $parent = Split-Path $dir -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $dir) {
            throw "Repo root not found. Run this script from inside the VoxelVerse repository."
        }
        $dir = $parent
    }
}

function Write-Utf8NoBom {
    param([string]$Path, [string]$Content)
    $folder = Split-Path $Path -Parent
    if (!(Test-Path $folder)) { New-Item -ItemType Directory -Force -Path $folder | Out-Null }
    $encoding = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $encoding)
}

function Default-Value {
    param([string]$Value, [string]$Fallback)
    if ([string]::IsNullOrWhiteSpace($Value)) { return $Fallback }
    return $Value
}

$Root = Find-VoxelVerseRoot
Set-Location $Root

$templatePath = Join-Path $Root "tools\voxelverse\block_visual_presets\templates\$Preset.render.ron.template"
if (!(Test-Path $templatePath)) { throw "Preset template not found: $templatePath" }
if ((Test-Path $Output) -and !$Force) { throw "Output already exists: $Output. Use -Force to overwrite." }

$template = Get-Content -Path $templatePath -Raw

switch ($Preset) {
    "natural_soil" {
        $BaseColor = Default-Value $BaseColor "#7A4528"
        $TopColor = Default-Value $TopColor "#8B5630"
        $SideColor = Default-Value $SideColor "#7A4528"
        $BottomColor = Default-Value $BottomColor "#4E2C19"
        if ([string]::IsNullOrWhiteSpace($SoundMaterial)) { $SoundMaterial = "dirt" }
    }
    "leafy_soil" {
        $BaseColor = Default-Value $BaseColor "#8CAD52"
        $TopColor = Default-Value $TopColor "#9BC554"
        $SideColor = Default-Value $SideColor "#80502E"
        $BottomColor = Default-Value $BottomColor "#4E2C19"
        if ([string]::IsNullOrWhiteSpace($SoundMaterial)) { $SoundMaterial = "grass" }
    }
    "mossy_soil" {
        $BaseColor = Default-Value $BaseColor "#6F8E48"
        $TopColor = Default-Value $TopColor "#86AA55"
        $SideColor = Default-Value $SideColor "#62452D"
        $BottomColor = Default-Value $BottomColor "#3B281A"
        if ([string]::IsNullOrWhiteSpace($SoundMaterial)) { $SoundMaterial = "grass" }
    }
    "stone_cells" {
        $BaseColor = Default-Value $BaseColor "#787772"
        $TopColor = Default-Value $TopColor "#85847E"
        $SideColor = Default-Value $SideColor "#70706B"
        $BottomColor = Default-Value $BottomColor "#565650"
        if ([string]::IsNullOrWhiteSpace($SoundMaterial)) { $SoundMaterial = "stone" }
        if ($Tool -eq "shovel") { $Tool = "pickaxe" }
        if ($Hardness -eq 0.6) { $Hardness = 1.5 }
        if ($Density -eq 1.3) { $Density = 2.7 }
    }
    "wood_rings" {
        $BaseColor = Default-Value $BaseColor "#A76835"
        $TopColor = Default-Value $TopColor "#C98A4A"
        $SideColor = Default-Value $SideColor "#8F552B"
        $BottomColor = Default-Value $BottomColor "#9B6032"
        if ([string]::IsNullOrWhiteSpace($SoundMaterial)) { $SoundMaterial = "wood" }
        if ($Tool -eq "shovel") { $Tool = "axe" }
        if ($Hardness -eq 0.6) { $Hardness = 2.0 }
        if ($Density -eq 1.3) { $Density = 0.75 }
    }
}

if ($Seed -eq 0) {
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Name)
    $hash = [uint32]2166136261
    foreach ($b in $bytes) { $hash = [uint32](($hash -bxor $b) * 16777619) }
    $Seed = $hash
}

$render = $template
$render = $render.Replace("{{BASE_COLOR}}", $BaseColor)
$render = $render.Replace("{{TOP_COLOR}}", $TopColor)
$render = $render.Replace("{{SIDE_COLOR}}", $SideColor)
$render = $render.Replace("{{BOTTOM_COLOR}}", $BottomColor)
$render = $render.Replace("{{SEED}}", "$Seed")

$block = @"
(
    display_key: Some("$DisplayKey"),
    tags: [
        "voxelverse:solid",
        "voxelverse:natural",
        "voxelverse:mineable_$Tool",
    ],
    mining: (
        hardness: $Hardness,
        tool: $Tool,
        sound_material: $SoundMaterial,
    ),
    render: $render,
    physics: (
        phase: solid,
        density: $Density,
        collider: full,
    ),
    drops: none,
)
"@

Write-Utf8NoBom -Path $Output -Content $block
Write-Host "Generated block: $Output" -ForegroundColor Green