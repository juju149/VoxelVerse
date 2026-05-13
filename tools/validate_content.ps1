#requires -Version 5.1
<#
.SYNOPSIS
    Thin alias for the canonical content validator.

.DESCRIPTION
    Historically there were two PowerShell scripts performing overlapping
    content validation. Per the VoxelVerse no-duplication rule, the single
    source of truth is now `vv-pack-doctor`. This file remains so older
    scripts and CI hooks keep working — it simply forwards to
    `tools/pack_doctor.ps1`.

.EXAMPLE
    pwsh tools/validate_content.ps1
#>
param(
    [string]$PackRoot = "assets/packs/core"
)

$here = Split-Path -Parent $MyInvocation.MyCommand.Path
& (Join-Path $here "pack_doctor.ps1") -PackRoot $PackRoot
exit $LASTEXITCODE
