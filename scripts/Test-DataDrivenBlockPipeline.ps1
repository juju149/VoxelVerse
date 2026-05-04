param(
    [switch]$RunCargoChecks,
    [switch]$Strict
)

$ErrorActionPreference = "Stop"

function Find-VoxelVerseRoot {
    $dir = (Get-Location).Path

    while ($true) {
        if ((Test-Path (Join-Path $dir "Cargo.toml")) -and
            (Test-Path (Join-Path $dir "assets")) -and
            (Test-Path (Join-Path $dir "src"))) {
            return (Resolve-Path $dir).Path
        }

        $parent = Split-Path $dir -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $dir) {
            throw "Repo root not found."
        }

        $dir = $parent
    }
}

function Add-Issue {
    param(
        [System.Collections.Generic.List[string]]$Issues,
        [string]$Message
    )

    $Issues.Add($Message) | Out-Null
    Write-Host $Message -ForegroundColor Yellow
}

function Assert-OrWarn {
    param(
        [bool]$Condition,
        [string]$Message,
        [System.Collections.Generic.List[string]]$Issues,
        [switch]$Strict
    )

    if ($Condition) {
        return
    }

    Add-Issue -Issues $Issues -Message $Message

    if ($Strict) {
        throw $Message
    }
}

function Get-FirstRegexGroup {
    param(
        [string]$Text,
        [string]$Pattern,
        [string]$Default = ""
    )

    $match = [regex]::Match($Text, $Pattern)
    if ($match.Success) {
        return $match.Groups[1].Value
    }

    return $Default
}

$Root = Find-VoxelVerseRoot
Set-Location $Root

$issues = [System.Collections.Generic.List[string]]::new()

$BlocksRoot = Join-Path $Root "assets\packs\voxelverse_core\defs\blocks"
$GeneratedRoot = Join-Path $BlocksRoot "generated"
$BlueprintRoot = Join-Path $Root "tools\voxelverse\block_blueprints"
$TemplateRoot = Join-Path $Root "tools\voxelverse\block_visual_presets\templates"
$ShaderPath = Join-Path $Root "src\vv-render\src\shaders\block_albedo.wgsl"
$RuntimePath = Join-Path $Root "src\vv-registry\src\block\runtime.rs"
$SchemaDetailsPath = Join-Path $Root "src\vv-schema\src\block\render\details.rs"

Write-Host ""
Write-Host "VoxelVerse data-driven block pipeline audit" -ForegroundColor Cyan
Write-Host "Root: $Root"
Write-Host ""

Assert-OrWarn -Condition (Test-Path $BlocksRoot) -Message "Missing blocks root: $BlocksRoot" -Issues $issues -Strict:$Strict
Assert-OrWarn -Condition (Test-Path $ShaderPath) -Message "Missing shader: $ShaderPath" -Issues $issues -Strict:$Strict
Assert-OrWarn -Condition (Test-Path $RuntimePath) -Message "Missing runtime file: $RuntimePath" -Issues $issues -Strict:$Strict

if (Test-Path $ShaderPath) {
    $shader = Get-Content -Path $ShaderPath -Raw

    Assert-OrWarn -Condition ($shader.Contains("fn patterned_block_albedo")) -Message "Shader missing patterned_block_albedo." -Issues $issues -Strict:$Strict
    Assert-OrWarn -Condition ($shader.Contains("fn vv_apply_runtime_details")) -Message "Shader missing runtime detail application." -Issues $issues -Strict:$Strict
    Assert-OrWarn -Condition ($shader.Contains("DETAIL_PEBBLE")) -Message "Shader missing detail constants." -Issues $issues -Strict:$Strict
}

if (Test-Path $RuntimePath) {
    $runtime = Get-Content -Path $RuntimePath -Raw

    Assert-OrWarn -Condition ($runtime.Contains("RUNTIME_BLOCK_DETAIL_PEBBLE")) -Message "Runtime missing detail constants." -Issues $issues -Strict:$Strict
    Assert-OrWarn -Condition ($runtime.Contains("detail_count")) -Message "Runtime comments/layout do not mention detail_count." -Issues $issues -Strict:$Strict
}

if (Test-Path $SchemaDetailsPath) {
    $schema = Get-Content -Path $SchemaDetailsPath -Raw

    Assert-OrWarn -Condition ($schema.Contains("BlockDetailDef")) -Message "Schema details missing BlockDetailDef." -Issues $issues -Strict:$Strict
    Assert-OrWarn -Condition ($schema.Contains("BlockDetailKind")) -Message "Schema details missing BlockDetailKind." -Issues $issues -Strict:$Strict
}
else {
    Add-Issue -Issues $issues -Message "Schema details file missing: $SchemaDetailsPath"
}

$templateFiles = @()
if (Test-Path $TemplateRoot) {
    $templateFiles = Get-ChildItem -Path $TemplateRoot -Filter "*.template" -Recurse
}

Assert-OrWarn -Condition ($templateFiles.Count -ge 5) -Message "Expected at least 5 visual preset templates, found $($templateFiles.Count)." -Issues $issues -Strict:$Strict

foreach ($template in $templateFiles) {
    $text = Get-Content -Path $template.FullName -Raw

    foreach ($placeholder in @("{{BASE_COLOR}}", "{{TOP_COLOR}}", "{{SIDE_COLOR}}", "{{BOTTOM_COLOR}}", "{{SEED}}")) {
        Assert-OrWarn -Condition ($text.Contains($placeholder)) -Message "Template missing placeholder $placeholder in $($template.FullName)" -Issues $issues -Strict:$Strict
    }

    Assert-OrWarn -Condition ($text.Contains('type: "patterned"')) -Message "Template is not patterned: $($template.FullName)" -Issues $issues -Strict:$Strict
}

$allBlockFiles = @()
if (Test-Path $BlocksRoot) {
    $allBlockFiles = Get-ChildItem -Path $BlocksRoot -Filter "*.ron" -Recurse
}

Assert-OrWarn -Condition ($allBlockFiles.Count -gt 0) -Message "No block RON files found." -Issues $issues -Strict:$Strict

$displayKeys = @{}
$fileNames = @{}

foreach ($file in $allBlockFiles) {
    $text = Get-Content -Path $file.FullName -Raw

    if ($text.Contains("{{") -or $text.Contains("}}")) {
        Add-Issue -Issues $issues -Message "Unresolved template placeholder in block file: $($file.FullName)"
        if ($Strict) { throw "Unresolved template placeholder in block file: $($file.FullName)" }
    }

    $display = Get-FirstRegexGroup -Text $text -Pattern 'display_key:\s*Some\("([^"]+)"\)'
    if (![string]::IsNullOrWhiteSpace($display)) {
        if ($displayKeys.ContainsKey($display)) {
            Add-Issue -Issues $issues -Message "Duplicate display_key '$display' in $($file.FullName) and $($displayKeys[$display])"
            if ($Strict) { throw "Duplicate display_key '$display'" }
        }
        else {
            $displayKeys[$display] = $file.FullName
        }
    }

    $name = [System.IO.Path]::GetFileNameWithoutExtension($file.Name)
    $relative = Resolve-Path -Path $file.FullName -Relative

    if ($fileNames.ContainsKey($name)) {
        Add-Issue -Issues $issues -Message "Duplicate block filename '$name' in $relative and $($fileNames[$name])"
        if ($Strict) { throw "Duplicate block filename '$name'" }
    }
    else {
        $fileNames[$name] = $relative
    }

    if ($relative.Contains("\generated\") -or $relative.Contains("/generated/")) {
        foreach ($marker in @("render:", "material:", "shape:", "program:", "meshing:", "physics:", "drops:")) {
            if (!$text.Contains($marker)) {
                Add-Issue -Issues $issues -Message "Generated block missing marker '$marker': $relative"
                if ($Strict) { throw "Generated block missing marker '$marker': $relative" }
            }
        }
    }
}

$generatedFiles = @()
if (Test-Path $GeneratedRoot) {
    $generatedFiles = Get-ChildItem -Path $GeneratedRoot -Filter "*.ron" -Recurse
}

Assert-OrWarn -Condition ($generatedFiles.Count -ge 12) -Message "Expected at least 12 generated block files, found $($generatedFiles.Count)." -Issues $issues -Strict:$Strict

if ($generatedFiles.Count -gt 0) {
    $generatedText = ($generatedFiles | ForEach-Object { Get-Content $_.FullName -Raw }) -join "`n"

    foreach ($marker in @(
        'type: "patterned"',
        'pattern: "natural_cells"',
        'pattern: "layered_surface"',
        'pattern: "rings"',
        'details: [',
        'kind: pebble',
        'kind: leaf_lobe',
        'kind: grain',
        'kind: crack'
    )) {
        Assert-OrWarn -Condition ($generatedText.Contains($marker)) -Message "Generated block set missing marker: $marker" -Issues $issues -Strict:$Strict
    }
}

# Soft guard against the architectural monster hiding under the bed:
# direct block-specific renderer functions.
$sourceFiles = Get-ChildItem -Path (Join-Path $Root "src") -Include "*.rs", "*.wgsl" -Recurse
$badFunctionPattern = 'fn\s+[a-zA-Z0-9_]*(grass|dirt|wood_log|stone_bricks|mossy_dirt)[a-zA-Z0-9_]*\s*\('

foreach ($src in $sourceFiles) {
    $text = Get-Content -Path $src.FullName -Raw

    if ([regex]::IsMatch($text, $badFunctionPattern)) {
        Add-Issue -Issues $issues -Message "Possible block-specific render/code function in $($src.FullName)"
        if ($Strict) { throw "Possible block-specific render/code function in $($src.FullName)" }
    }
}

Write-Host ""
if ($issues.Count -eq 0) {
    Write-Host "Audit passed. The data-driven block pipeline looks healthy." -ForegroundColor Green
}
else {
    Write-Host "Audit completed with $($issues.Count) warning(s)." -ForegroundColor Yellow
    if ($Strict) {
        throw "Strict audit failed."
    }
}

if ($RunCargoChecks) {
    Write-Host ""
    Write-Host "Running cargo checks..." -ForegroundColor Cyan

    cargo fmt
    cargo test -p vv-compiler --test compile_voxelverse_core
    cargo check -p vv-schema
    cargo check -p vv-registry
    cargo check -p vv-compiler
    cargo check -p vv-render
}