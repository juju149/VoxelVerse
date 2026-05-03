param(
    [string]$BlueprintRoot = "tools\voxelverse\block_blueprints",
    [string]$TemplateRoot = "tools\voxelverse\block_visual_presets\templates",
    [string]$OutputRoot = "assets\packs\voxelverse_core\defs\blocks\generated\blueprints",
    [string]$Family = "all",
    [switch]$CleanOutput
)

$ErrorActionPreference = "Stop"

function Find-VoxelVerseRoot {
    $dir = (Get-Location).Path
    while ($true) {
        if ((Test-Path (Join-Path $dir "Cargo.toml")) -and (Test-Path (Join-Path $dir "assets"))) {
            return (Resolve-Path $dir).Path
        }
        $parent = Split-Path $dir -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $dir) { throw "Repo root not found." }
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

function Get-Field {
    param([string]$Text, [string]$Name, [string]$Default = "")
    $pattern = "$Name\s*:\s*""([^""]*)"""
    $match = [regex]::Match($Text, $pattern)
    if ($match.Success) { return $match.Groups[1].Value }
    return $Default
}

function Get-Number {
    param([string]$Text, [string]$Name, [string]$Default = "0")
    $pattern = "$Name\s*:\s*([0-9]+(?:\.[0-9]+)?)"
    $match = [regex]::Match($Text, $pattern)
    if ($match.Success) { return $match.Groups[1].Value }
    return $Default
}

function Assert-Hex {
    param([string]$Value, [string]$Field, [string]$Path)
    if ($Value -notmatch '^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$') {
        throw "Invalid hex color in $Path field '$Field': $Value"
    }
}

function Expand-Blueprint {
    param(
        [string]$BlueprintFile,
        [string]$TemplateDir,
        [string]$OutRoot
    )

    $text = Get-Content -Path $BlueprintFile -Raw

    $family = Get-Field $text "family" "misc"
    $preset = Get-Field $text "preset"
    $name = Get-Field $text "name"
    $displayKey = Get-Field $text "display_key"
    $base = Get-Field $text "base"
    $top = Get-Field $text "top"
    $side = Get-Field $text "side"
    $bottom = Get-Field $text "bottom"
    $tool = Get-Field $text "tool" "shovel"
    $sound = Get-Field $text "sound" "dirt"
    $hardness = Get-Number $text "hardness" "0.6"
    $density = Get-Number $text "density" "1.3"
    $seed = Get-Number $text "seed" "0"

    if ([string]::IsNullOrWhiteSpace($preset)) { throw "Missing preset in $BlueprintFile" }
    if ([string]::IsNullOrWhiteSpace($name)) { throw "Missing name in $BlueprintFile" }
    if ([string]::IsNullOrWhiteSpace($displayKey)) { throw "Missing display_key in $BlueprintFile" }

    foreach ($entry in @(
        @{ Field = "base"; Value = $base },
        @{ Field = "top"; Value = $top },
        @{ Field = "side"; Value = $side },
        @{ Field = "bottom"; Value = $bottom }
    )) {
        Assert-Hex -Value $entry.Value -Field $entry.Field -Path $BlueprintFile
    }

    if ($preset -notin @("natural_soil", "leafy_soil", "mossy_soil", "stone_cells", "wood_rings")) {
        throw "Unknown preset in ${BlueprintFile}: $preset"
    }

    $templatePath = Join-Path $TemplateDir "$preset.render.ron.template"
    if (!(Test-Path $templatePath)) {
        throw "Missing preset template for '$preset': $templatePath"
    }

    $render = Get-Content -Path $templatePath -Raw
    $render = $render.Replace("{{BASE_COLOR}}", $base)
    $render = $render.Replace("{{TOP_COLOR}}", $top)
    $render = $render.Replace("{{SIDE_COLOR}}", $side)
    $render = $render.Replace("{{BOTTOM_COLOR}}", $bottom)
    $render = $render.Replace("{{SEED}}", $seed)

    $block = @"
(
    display_key: Some("$displayKey"),
    tags: [
        "voxelverse:solid",
        "voxelverse:natural",
        "voxelverse:mineable_$tool",
    ],
    mining: (
        hardness: $hardness,
        tool: $tool,
        sound_material: $sound,
    ),
    render: $render,
    physics: (
        phase: solid,
        density: $density,
        collider: full,
    ),
    drops: none,
)
"@

    $output = Join-Path (Join-Path $OutRoot $family) "$name.ron"
    Write-Utf8NoBom -Path $output -Content $block

    return $output
}

$Root = Find-VoxelVerseRoot
Set-Location $Root

$BlueprintPath = Join-Path $Root $BlueprintRoot
$TemplatePath = Join-Path $Root $TemplateRoot
$OutPath = Join-Path $Root $OutputRoot

if (!(Test-Path $BlueprintPath)) { throw "Blueprint root missing: $BlueprintPath" }
if (!(Test-Path $TemplatePath)) { throw "Template root missing: $TemplatePath" }

if ($CleanOutput -and (Test-Path $OutPath)) {
    Remove-Item -Path $OutPath -Recurse -Force
}

$files = Get-ChildItem -Path $BlueprintPath -Filter "*.ron" -Recurse
$count = 0

foreach ($file in $files) {
    $text = Get-Content -Path $file.FullName -Raw
    $familyValue = Get-Field $text "family" "misc"

    if ($Family -ne "all" -and $familyValue -ne $Family) {
        continue
    }

    $output = Expand-Blueprint -BlueprintFile $file.FullName -TemplateDir $TemplatePath -OutRoot $OutPath
    Write-Host "Compiled blueprint: $($file.Name) -> $output" -ForegroundColor Green
    $count++
}

Write-Host "Compiled $count block blueprints." -ForegroundColor Green