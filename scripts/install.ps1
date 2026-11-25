#requires -Version 7.0
[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter(Mandatory)][string]$Source,
    [Parameter(Mandatory)][ValidateSet('Codex', 'Claude', 'Antigravity')][string]$Agent,
    [string]$HomeRoot,
    [string]$CodexHome,
    [AllowEmptyCollection()][string[]]$Skills,
    [switch]$Apply
)

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'distribution/InstallSupport.psm1')
$layout = Get-CirnetInstallLayout -Agent $Agent -HomeRoot $HomeRoot -CodexHome $CodexHome
$plan = Get-CirnetInstallPlan -Source $Source -Layout $layout -Skills $Skills
$changes = @($plan.Entries | Where-Object { $_.BeforeHash -cne $_.AfterHash })
Show-CirnetInstallPlan -Plan $plan -Action 'install'
if ($changes.Count -eq 0) {
    Write-Host 'The selected rules and skills are already installed.'
} elseif ($Apply -and $PSCmdlet.ShouldProcess($layout.Home, "Install CIRNET for $Agent")) {
    Save-CirnetInstall -Plan $plan -Changes $changes
} else {
    Write-Host 'Preview complete. No files changed.'
}
