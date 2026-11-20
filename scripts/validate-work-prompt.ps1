#requires -Version 7.0
[CmdletBinding(DefaultParameterSetName = 'Validate')]
param(
    [Parameter(Mandatory, ParameterSetName='Validate')][string]$Prompt,
    [Parameter(Mandatory)][string]$OriginalTask,
    [Parameter(Mandatory, ParameterSetName='Validate')][string]$Authorization,
    [Parameter(Mandatory, ParameterSetName='Validate')][string]$AuthorizationSha256,
    [Parameter(ParameterSetName='Validate')]
    [ValidatePattern('^[0-9a-f]{64}\z')][string]$ReviewedPromptSha256,
    [Parameter(ParameterSetName='Validate')][switch]$RequirePreventiveScope,
    [Parameter(ParameterSetName='Validate')][switch]$CaptureScope,
    [Parameter(ParameterSetName='Validate')][string]$ScopeSnapshot,
    [Parameter(ParameterSetName='Validate')][ValidatePattern('^[0-9a-f]{64}\z')][string]$ScopeSnapshotSha256,
    [Parameter(Mandatory, ParameterSetName='Prepare')][switch]$Prepare,
    [Parameter(ParameterSetName='Prepare')][string[]]$SourcePath,
    [Parameter(ParameterSetName='Prepare')][switch]$Sensitive,
    [Parameter(ParameterSetName='Prepare')][ValidateSet('PROMPT_READY','PROMPT_BLOCKED')][string]$Readiness,
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
