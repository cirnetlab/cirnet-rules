#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Source,
    [Parameter(Mandatory)][ValidateSet('Codex', 'Claude', 'Antigravity')][string]$Agent,
    [string]$HomeRoot,
    [string]$CodexHome,
    [AllowEmptyCollection()][string[]]$Skills
)

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'distribution/InstallSupport.psm1')
$layout = Get-CirnetInstallLayout -Agent $Agent -HomeRoot $HomeRoot -CodexHome $CodexHome
$plan = Get-CirnetInstallPlan -Source $Source -Layout $layout -Skills $Skills
Show-CirnetInstallPlan -Plan $plan -Action 'verify'
$differences = @($plan.Entries | Where-Object { $_.BeforeHash -cne $_.AfterHash })
if ($differences.Count -ne 0) {
    throw ('Installation differs from the selected bundle: ' + ($differences.Path -join ', '))
}
Write-Host 'Verified selected installed files, hashes, and references. Client loading was not tested.'
