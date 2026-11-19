#requires -Version 7.0
[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter(Mandatory)][string]$Backup,
    [Parameter(Mandatory)][ValidateSet('Codex', 'Claude', 'Antigravity')][string]$Agent,
    [string]$HomeRoot,
    [string]$CodexHome,
    [switch]$Apply
)

$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'distribution/InstallSupport.psm1')
$layout = Get-CirnetInstallLayout -Agent $Agent -HomeRoot $HomeRoot -CodexHome $CodexHome
$plan = Get-CirnetRestorePlan -Layout $layout -Backup $Backup
Show-CirnetInstallPlan -Plan $plan -Action 'restore'
if ($Apply -and $PSCmdlet.ShouldProcess($layout.Home, "Restore CIRNET for $Agent")) {
    Invoke-CirnetChanges -Layout $layout -Changes $plan.Entries
    Write-Host "Restored files recorded in $Backup"
} else {
    Write-Host 'Preview complete. No files changed.'
}
