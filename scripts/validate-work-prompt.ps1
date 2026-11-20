#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Prompt,
    [Parameter(Mandatory)][string]$OriginalTask,
    [Parameter(Mandatory)][string]$Authorization,
    [Parameter(Mandatory)][string]$AuthorizationSha256,
    [ValidatePattern('^[0-9a-f]{64}\z')][string]$ReviewedPromptSha256,
    [string]$Repository = (Get-Location).ProviderPath
)
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'shared/WorkPrompt.psm1')
$PSBoundParameters['Repository'] = $Repository
$result = Test-CirnetWorkPromptFile @PSBoundParameters
$result | ConvertTo-Json -Compress
exit (@{ PASS=0; FAIL=1; REVIEW_REQUIRED=2; BLOCKED=3 }[$result.status])
