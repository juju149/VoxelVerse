param(
    [string[]]$Roots = @("crates", "apps")
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$extensions = @(".rs", ".toml", ".ps1")
$forbidden = @(
    "temporary",
    "for now",
    "will fix later",
    "quick hack",
    "old system",
    "backward compatibility",
    "backwards compatibility",
    "legacy"
)

$violations = New-Object System.Collections.Generic.List[string]

foreach ($relativeRoot in $Roots) {
    $scanRoot = Join-Path $root $relativeRoot
    if (-not (Test-Path $scanRoot)) {
        continue
    }

    Get-ChildItem -Path $scanRoot -Recurse -File | Where-Object {
        $extensions -contains $_.Extension.ToLowerInvariant()
    } | ForEach-Object {
        $path = $_.FullName
        $relativePath = Resolve-Path -Relative $path
        $lineNumber = 0
        foreach ($line in [System.IO.File]::ReadLines($path)) {
            $lineNumber += 1
            $lower = $line.ToLowerInvariant()
            if ($lower.Contains('"legacy_imports"')) {
                continue
            }
            foreach ($term in $forbidden) {
                if ($lower.Contains($term)) {
                    $violations.Add("${relativePath}:${lineNumber}: forbidden marker '$term'")
                    break
                }
            }
        }
    }
}

if ($violations.Count -gt 0) {
    $violations | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Host "No legacy markers found."
