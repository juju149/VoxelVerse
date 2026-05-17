param(
    [string]$ManifestPath = "Cargo.toml"
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$allowPattern = "#(!)?\[allow\(dead_code\)\]"
$violations = New-Object System.Collections.Generic.List[string]

Get-ChildItem -Path (Join-Path $root "crates"), (Join-Path $root "apps") -Recurse -File -Filter "*.rs" |
    ForEach-Object {
        $path = $_.FullName
        $relativePath = Resolve-Path -Relative $path
        $lineNumber = 0
        foreach ($line in [System.IO.File]::ReadLines($path)) {
            $lineNumber += 1
            if ($line -match $allowPattern) {
                $violations.Add("${relativePath}:${lineNumber}: remove allow(dead_code)")
            }
        }
    }

if ($violations.Count -gt 0) {
    $violations | ForEach-Object { Write-Error $_ }
    exit 1
}

cargo clippy --workspace --all-targets --manifest-path $ManifestPath -- -D warnings
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "No dead code exemptions found."
