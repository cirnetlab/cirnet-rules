#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Repository,
    [Parameter(Mandatory)][string]$TaskId,
    [Parameter(Mandatory)][string]$ClusterId,
    [Parameter(Mandatory)][ValidateSet('codex', 'claude-code')][string]$AgentFamily,
    [ValidateNotNullOrEmpty()][string]$CheckpointRoot,
    [string]$RelatedRootsJson = '{}',
    [switch]$ConfirmScope
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Checkpoint.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Verification.psm1')

if (-not $ConfirmScope) { throw 'Confirm authorized checkpoint read access with -ConfirmScope.' }
Assert-ContinuityId $ClusterId
if ($ClusterId.Length -gt 59) { throw 'A correction cluster ID has at most 59 characters.' }
if ($IsWindows -and ($Repository -match '^[\\/]{2}' -or $CheckpointRoot -match '^[\\/]{2}')) {
    throw 'Correction history requires local worktree and checkpoint paths.'
}
$related = ConvertFrom-ContinuityJson ([Text.Encoding]::UTF8.GetBytes($RelatedRootsJson)) -UniqueProperties
foreach ($id in $related.psbase.Keys) {
    Assert-ContinuityId $id
    $path = $related[$id]
    if ($path -isnot [string] -or -not [IO.Path]::IsPathFullyQualified($path) -or
        ($IsWindows -and $path -match '^[\\/]{2}')) { throw 'Use a local absolute related worktree path.' }
}
$checkout = Get-ContinuityRepository (Get-ContinuityInputPath $Repository)
$store = Get-ContinuityStore $checkout $CheckpointRoot $TaskId
$prefix = $ClusterId + '-a'
$attemptIdPattern = [regex]::Escape($prefix) + '[0-9]{3}'
$hypothesisPattern = '^- cirnet-correction (' + $attemptIdPattern + ') ([a-f0-9]{64})\z'

function Get-CorrectionAttempts($Checkpoint) {
    $notes = Get-ContinuityNoteSections $Checkpoint.Progress progress
    $hypotheses = [ordered]@{}
    foreach ($line in ($notes['Decisions & Blockers'] -split '\r?\n')) {
        if ($line -cnotmatch ('^- cirnet-correction ' + $attemptIdPattern + '(?: |\z)')) { continue }
        if ($line -cnotmatch $hypothesisPattern -or $hypotheses.Contains($Matches[1])) {
            throw 'Invalid or duplicate correction hypothesis entry.'
        }
        $hypotheses.Add($Matches[1], $Matches[2])
    }

    $checks = $Checkpoint.State.checks
    $ids = @($checks.psbase.Keys | Where-Object { $_ -cmatch ('^' + $attemptIdPattern + '\z') } | Sort-Object -CaseSensitive)
    if ($ids.Count -gt 128) { throw 'Correction history exceeds its attempt limit.' }
    $last = $null
    for ($index = 0; $index -lt $ids.Count; $index++) {
        $id = $ids[$index]
        $measurement = $checks[$id]
        if ($id -cne ($prefix + ($index + 1).ToString('D3', [Globalization.CultureInfo]::InvariantCulture)) -or
            $measurement.kind -cnotin @('build', 'test', 'edit') -or ($index -eq 0 -and $measurement.kind -ceq 'edit') -or
            ($null -ne $last -and ($null -eq $last.finishedAtUnixMs -or $last.finishedAtUnixMs -gt $measurement.startedAtUnixMs))) {
            throw 'Correction attempt history has a gap, invalid order or unsupported milestone.'
        }
        $last = $measurement
        $isRepair = $measurement.kind -ceq 'edit'
        if ($isRepair -ne $hypotheses.Contains($id)) { throw 'Correction hypotheses must match controlled edit evidence IDs.' }
        $hypothesis = if ($isRepair) { $hypotheses[$id] } else { $null }
        if ($isRepair) { $hypotheses.Remove($id) }
        $definition = Get-ContinuityValueHash @{
            kind = $measurement.kind; executable = $measurement.executable; arguments = $measurement.arguments
            toolPath = $measurement.toolPath; toolSha256 = $measurement.toolSha256; references = $measurement.referenceFingerprint
            inputPaths = @($measurement.inputs.psbase.Keys | Sort-Object -CaseSensitive)
            environment = if ($measurement.Contains('environment')) { $measurement.environment } else { @{} }
        }
        $identity = Get-ContinuityValueHash @{
            definition = $definition; git = $measurement.gitFingerprint; inputs = $measurement.inputs
        }
        [ordered]@{
            id = $id; clusterId = $ClusterId; kind = if ($isRepair) { 'repair' } else { 'check' }
            identity = $identity; definition = $definition; phase = $measurement.phase; status = $measurement.result
            launched = $measurement.phase -cne 'blocked'; exitCode = $measurement.exitCode; hypothesis = $hypothesis
            invocation = Get-ContinuityValueHash @{
                identity = $identity; hypothesis = $hypothesis; startedAtUnixMs = $measurement.startedAtUnixMs
            }
            observation = Get-ContinuityValueHash $measurement
        }
    }
    if ($hypotheses.Count) { throw 'Correction hypothesis is missing its controlled edit evidence.' }
}

$lock = Enter-ContinuityLock $store
try {
    $remaining = 32MB
    $current = Read-ContinuityCheckpoint $store $TaskId -RemainingBytes ([ref]$remaining)
    $previous = Read-ContinuityCheckpoint $store $TaskId -Previous -RemainingBytes ([ref]$remaining)
    if ($current.State.agentFamily -cne $AgentFamily -or $previous.State.agentFamily -cne $AgentFamily -or
        -not (Test-ContinuityWorktree $checkout $current.State.git) -or -not (Test-ContinuityWorktree $checkout $previous.State.git) -or
        $current.State.lifecycle -cne 'open' -or $current.State.taskKey -cne $previous.State.taskKey -or
        $current.State.checkpointId -ceq $previous.State.checkpointId -or $current.Task -cne $previous.Task) {
        throw 'The checkpoints do not identify an open task in the same owning worktree.'
    }
    $attempts = @(Get-CorrectionAttempts $current)
    $prior = @(Get-CorrectionAttempts $previous)
    if ($attempts.Count -eq 0 -or $attempts.Count -lt $prior.Count -or $attempts.Count -gt ($prior.Count + 1)) {
        throw 'Correction history cannot be reconciled with the previous checkpoint.'
    }
    for ($index = 0; $index -lt $prior.Count; $index++) {
        if ($attempts[$index].invocation -cne $prior[$index].invocation -or
            ($prior[$index].phase -cne 'running' -and $attempts[$index].observation -cne $prior[$index].observation)) {
            throw 'A recorded correction attempt changed between checkpoints.'
        }
    }
    $definition = $attempts[0].definition
    $latest = $attempts[-1]
    $fresh = $false
    $running = @($current.State.checks.psbase.Values | Where-Object phase -CEQ 'running')
    if ($latest.kind -ceq 'check' -and $latest.status -ceq 'FAIL' -and $running.Count -eq 0) {
        $git = Get-ContinuityGitSnapshot $checkout
        $fingerprint = Get-ContinuityGitFingerprint $git
        $problems = @(Get-ContinuityReferenceProblems $checkout $store $current.State.references $related)
        $check = Get-EffectiveCheck $checkout $store $latest.id $current.State.checks[$latest.id] $fingerprint `
            (Get-ContinuityValueHash $current.State.references) ($problems.Count -eq 0)
        $fresh = $check.Status -ceq 'FAIL' -and $fingerprint -ceq $current.State.gitFingerprint -and
            (-not $git.Contains('unmerged') -or $git.unmerged.Count -eq 0)
    }
    [ordered]@{
        taskKey = $current.State.taskKey; checkpointId = $current.State.checkpointId
        definition = $definition; fresh = $fresh; attempts = $attempts
    } | ConvertTo-Json -Depth 6 -Compress
}
finally { $lock.Dispose() }
