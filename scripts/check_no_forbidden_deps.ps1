param(
    [string]$ManifestPath = "Cargo.toml"
)

$ErrorActionPreference = "Stop"

$metadataJson = cargo metadata --no-deps --format-version 1 --manifest-path $ManifestPath
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$metadata = $metadataJson | ConvertFrom-Json
$packagesByName = @{}
foreach ($package in $metadata.packages) {
    $packagesByName[$package.name] = $package
}

function Assert-CrateExists {
    param([string]$Crate)
    if (-not $packagesByName.ContainsKey($Crate)) {
        throw "crate '$Crate' not found in cargo metadata"
    }
}

function Assert-NoDirectDependency {
    param(
        [string]$From,
        [string[]]$Forbidden
    )

    Assert-CrateExists $From
    $dependencyNames = @($packagesByName[$From].dependencies | ForEach-Object { $_.name })
    foreach ($blocked in $Forbidden) {
        if ($dependencyNames -contains $blocked) {
            throw "forbidden dependency: $From -> $blocked"
        }
    }
}

Assert-NoDirectDependency "vv-render" @("vv-gameplay")
Assert-NoDirectDependency "vv-gameplay" @("winit")
Assert-NoDirectDependency "vv-world" @("voxelverse")
Assert-NoDirectDependency "vv-worldgen" @("vv-render")
Assert-NoDirectDependency "vv-pack-loader" @("vv-render", "vv-gameplay", "vv-world")

Write-Host "Forbidden dependency check passed."
