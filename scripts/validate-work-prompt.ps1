#requires -Version 7.0
[CmdletBinding(DefaultParameterSetName = 'Validate')]
param(
    [Parameter(Mandatory, ParameterSetName='Validate')][string]$Prompt,
    [Parameter(Mandatory)][string]$OriginalTask,
    [Parameter(Mandatory, ParameterSetName='Validate')][Parameter(ParameterSetName='Prepare')][string]$Authorization,
    [Parameter(Mandatory, ParameterSetName='Validate')][Parameter(ParameterSetName='Prepare')][string]$AuthorizationSha256,
    [Parameter(ParameterSetName='Validate')]
    [ValidatePattern('^[0-9a-f]{64}\z')][string]$ReviewedPromptSha256,
    [switch]$RequirePreventiveScope,
    [Parameter(ParameterSetName='Validate')][switch]$ExecutionPreflight,
    [Parameter(ParameterSetName='Validate')][switch]$CaptureScope,
    [Parameter(ParameterSetName='Validate')][string]$ScopeSnapshot,
    [Parameter(ParameterSetName='Validate')][ValidatePattern('^[0-9a-f]{64}\z')][string]$ScopeSnapshotSha256,
    [Parameter(Mandatory, ParameterSetName='Prepare')][switch]$Prepare,
    [Parameter(ParameterSetName='Prepare')][string[]]$SourcePath,
    [Parameter(ParameterSetName='Prepare')][switch]$Sensitive,
    [Parameter(ParameterSetName='Prepare')][ValidateSet('PROMPT_READY','PROMPT_BLOCKED')][string]$Readiness,
    [Parameter(ParameterSetName='Prepare')][switch]$Autonomous,
    [Parameter(ParameterSetName='Prepare')][ValidateSet('ReadOnly','Edit')][string]$AuthorizedScope,
    [Parameter(ParameterSetName='Prepare')][string[]]$ReadPath,
    [Parameter(ParameterSetName='Prepare')][string[]]$WritePath,
    [Parameter(ParameterSetName='Prepare')][ValidateSet('LIGHT','STANDARD','DEEP')][string]$Risk,
    [string]$Repository = (Get-Location).ProviderPath
)
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'shared/WorkPrompt.psm1')
$PSBoundParameters['Repository'] = $Repository
$result = if ($Prepare) {
    [void]$PSBoundParameters.Remove('Prepare')
    New-CirnetWorkPromptProposal @PSBoundParameters
} else { Test-CirnetWorkPromptFile @PSBoundParameters }
$result | ConvertTo-Json -Depth 24 -Compress
exit (@{ PASS=0; FAIL=1; REVIEW_REQUIRED=2; BLOCKED=3 }[$result.status])
