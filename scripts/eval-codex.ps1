#requires -Version 7.0
[CmdletBinding()]
param(
    [string]$Cohort,
    [string]$RulesRoot,
    [string]$OutputRoot,
    [string]$Session,
    [string]$SourceRevision,
    [string]$CaseId,
    [int]$Attempt,
    [int]$TimeoutSeconds,
    [string]$Model,
    [string]$Effort,
    [string]$Role,
    [string]$HandoffPath,
    [switch]$ValidateCandidateHandoff,
    [string]$ExpectedHandoffSha256,
    [string]$ReviewedHandoffSha256,
    [Alias('BenchmarkRoot')][string]$ComparisonRoot,
    [string]$SuiteRoot,
    [string]$CodexPath,
    [string]$AuthHome,
    [string]$ToolPath,
    [switch]$UseApiKey,
    [switch]$Execute
)

& (Join-Path $PSScriptRoot 'benchmark/role.ps1') @PSBoundParameters
