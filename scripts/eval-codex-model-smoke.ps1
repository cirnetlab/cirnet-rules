#requires -Version 7.0
[CmdletBinding()]
param(
    [string]$Session,
    [string]$OutputRoot,
    [string]$ReviewFile,
    [string]$CaseId,
    [string]$Model,
    [string]$Effort,
    [int]$TimeoutSeconds,
    [string]$AuthHome,
    [string]$CodexPath,
    [string]$ToolPath,
    [switch]$UseApiKey,
    [switch]$Execute,
    [switch]$ReviewHandoffs,
    [Alias('ApprovedRoleCalls')][int]$MaxNativeCalls
)

& (Join-Path $PSScriptRoot 'benchmark/run.ps1') @PSBoundParameters
exit $LASTEXITCODE
