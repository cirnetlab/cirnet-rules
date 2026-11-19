#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory, Position = 0)]
    [ValidateSet('save', 'status', 'verify', 'resume', 'close', 'export', 'run')][string]$Action,
    [string]$Repository = (Get-Location).ProviderPath,
    [string]$CheckpointRoot = '.cirnet/tasks',
    [string]$TaskId,
    [ValidateSet('codex', 'claude-code')][string]$AgentFamily,
    [hashtable]$RelatedRoot = @{},
    [string]$TaskFile,
    [string]$ProgressFile,
    [string]$ReferencesFile,
    [string[]]$RequiredCheck = @(),
    [switch]$ConfirmScope,
    [switch]$AcceptGitDrift,
    [switch]$ConfirmAcceptance,
    [string]$EvidenceId,
    [ValidateSet('edit', 'build', 'test', 'handoff')][string]$Kind,
    [string]$Executable,
    [string[]]$Arguments = @(),
    [string[]]$InputPath = @(),
    [ValidateRange(1, 86400)][int]$TimeoutSeconds = 600,
    [string]$OutputFile,
    [string]$HandoffFile,
    [string]$ExpectedSha256
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot 'continuity/Checkpoint.psm1')
Import-Module (Join-Path $PSScriptRoot 'continuity/Verification.psm1')

$Action = $Action.ToLowerInvariant()
if ($AgentFamily) { $AgentFamily = $AgentFamily.ToLowerInvariant() }
if ($Kind) { $Kind = $Kind.ToLowerInvariant() }
if ($HandoffFile) {
    if ($Action -cne 'verify' -or -not $AgentFamily) { throw 'Use verify with -HandoffFile, -ExpectedSha256 and -AgentFamily.' }
    return Read-ContinuityHandoff $HandoffFile $ExpectedSha256 $AgentFamily
}
if ($Action -cnotin @('status', 'verify') -and -not $AgentFamily) { throw 'Supply -AgentFamily.' }
if (($Action -cin @('resume', 'close', 'export') -or $AcceptGitDrift) -and -not $ConfirmScope) {
    throw 'Use -ConfirmScope after reviewing the task, current authorization, applicable references and next action.'
}

$checkout = Get-ContinuityRepository (Get-ContinuityInputPath $Repository)
$store = Get-ContinuityStore $checkout $CheckpointRoot $TaskId

function Assert-AgentFamily($Saved) {
    if ($AgentFamily -and $AgentFamily -cne $Saved.State.agentFamily) {
        throw 'Agent family does not match the saved checkpoint.'
    }
}

function Get-SavedResult($Saved) {
    [pscustomobject]@{
        TaskId = $Saved.State.taskId
        CheckpointId = $Saved.State.checkpointId
        Lifecycle = $Saved.State.lifecycle
        CheckpointDirectory = $Saved.Directory
    }
}

function Assert-Ready($Report) {
    if (-not $Report.Ready) { throw ('The checkpoint is not ready. ' + ($Report.Blockers -join ' ')) }
}

if ($Action -cin @('status', 'verify')) {
    $checkpoint = Read-ContinuityCheckpoint $store $TaskId
    Assert-AgentFamily $checkpoint
    $report = Get-ContinuityStatus $checkout $store $checkpoint (Get-ContinuityGitSnapshot $checkout) $RelatedRoot
    $report
    if ($Action -ceq 'verify' -and -not $report.Ready) { exit 1 }
    exit 0
}

$lock = Enter-ContinuityLock $store -Create:($Action -ceq 'save')
try {
    $checkpoint = $null
    if ($Action -cne 'save' -or [IO.File]::Exists((Join-Path $store 'current.json'))) {
        $checkpoint = Read-ContinuityCheckpoint $store $TaskId
        Assert-AgentFamily $checkpoint
    }
    if ($null -ne $checkpoint -and $checkpoint.State.lifecycle -ceq 'closed' -and $Action -cne 'export') {
        throw 'The task is already closed. Use status or verify to inspect its evidence.'
    }
    $git = Get-ContinuityGitSnapshot $checkout

    if ($Action -ceq 'save') {
        if ($null -eq $checkpoint) {
            if (-not ($TaskFile -and $ProgressFile -and $ReferencesFile)) {
                throw 'The first save requires -TaskFile, -ProgressFile and -ReferencesFile.'
            }
            $task = Read-ContinuityInput $TaskFile
            $progress = Read-ContinuityInput $ProgressFile
            $state = [ordered]@{
                schemaVersion = 1
                taskId = $TaskId
                taskKey = [guid]::NewGuid().ToString('N')
                checkpointId = ''
                agentFamily = $AgentFamily
                savedAtUnixMs = 0
                lifecycle = 'open'
                git = $git
                gitFingerprint = Get-ContinuityGitFingerprint $git
                references = New-ContinuityReferences $checkout $store $ReferencesFile $RelatedRoot $git.head
                requiredChecks = $RequiredCheck
                checks = [ordered]@{}
            }
        }
        else {
            if ($TaskFile -or $PSBoundParameters.ContainsKey('RequiredCheck')) {
                throw 'The original task and required checks are fixed by the first save.'
            }
            $state = $checkpoint.State
            $comparison = Compare-ContinuityGit $state.git $git
            if (@($comparison.Changes.Fields | Where-Object { $_ -cin @('worktree', 'gitDirectory', 'repository') }).Count) {
                throw 'The checkpoint belongs to a different worktree.'
            }
            if (-not $comparison.Matches -and -not $AcceptGitDrift) {
                throw 'Git changed. Run verify, review the differences and save with -AcceptGitDrift -ConfirmScope.'
            }
            if ($ReferencesFile) {
                if (-not $ConfirmScope) { throw 'Review the reference changes before saving with -ConfirmScope.' }
                $state.references = New-ContinuityReferences $checkout $store $ReferencesFile $RelatedRoot $git.head
            }
            else {
                $problems = @(Get-ContinuityReferenceProblems $checkout $store $state.references $RelatedRoot)
                if ($problems.Count) { throw ('Verify the original references before saving. ' + ($problems -join ' ')) }
            }
            $task = $checkpoint.Task
            $progress = if ($ProgressFile) { Read-ContinuityInput $ProgressFile } else { $checkpoint.Progress }
            $state.git = $git
            $state.gitFingerprint = Get-ContinuityGitFingerprint $git
        }
        return Get-SavedResult (Save-ContinuityCheckpoint $store $state $task $progress)
    }

    $state = $checkpoint.State
    if ($Action -ceq 'run') {
        Assert-ContinuityId $EvidenceId
        if (-not ($Kind -and $Executable)) { throw 'Run requires -EvidenceId, -Kind and -Executable.' }
        $referenceProblems = @(Get-ContinuityReferenceProblems $checkout $store $state.references $RelatedRoot)
        if ($state.gitFingerprint -cne (Get-ContinuityGitFingerprint $git) -or $referenceProblems.Count) {
            throw 'Verify the saved Git state and references before running a controlled milestone.'
        }
        if ($state.checks.Contains($EvidenceId) -and $state.checks[$EvidenceId].kind -cne $Kind) {
            throw 'Use a distinct evidence ID for each kind of milestone.'
        }
        $progress = if ($ProgressFile) { Read-ContinuityInput $ProgressFile } else { $checkpoint.Progress }
        $measurement = New-ContinuityMeasurement $checkout $store $state $Kind $Executable $Arguments $InputPath
        $state.checks[$EvidenceId] = $measurement
        $confirmed = Save-ContinuityCheckpoint $store $state $checkpoint.Task $progress
        if ($measurement.phase -ceq 'running') {
            Invoke-ContinuityMeasurement $checkout $measurement $TimeoutSeconds
            $after = Get-ContinuityGitSnapshot $checkout
            $measurement.afterGitFingerprint = Get-ContinuityGitFingerprint $after
            $referenceProblems = @(Get-ContinuityReferenceProblems $checkout $store $state.references $RelatedRoot)
            if ($measurement.phase -ceq 'finished') {
                if ($Kind -ceq 'edit' -and $measurement.result -ceq 'PASS' -and $referenceProblems.Count -eq 0) {
                    $state.git = $after
                    $state.gitFingerprint = $measurement.afterGitFingerprint
                }
                elseif ($measurement.afterGitFingerprint -cne $measurement.gitFingerprint -or $referenceProblems.Count -or
                    -not (Get-ProcessInputsMatch $checkout $store $measurement)) {
                    $measurement.cause = 'changed-during-run'
                }
            }
            $confirmed = Save-ContinuityCheckpoint $store $state $checkpoint.Task $progress
        }
        $result = Get-EffectiveCheck $checkout $store $EvidenceId $measurement $state.gitFingerprint $measurement.referenceFingerprint ($referenceProblems.Count -eq 0)
        Add-Member -InputObject $result -NotePropertyName CheckpointId -NotePropertyValue $confirmed.State.checkpointId
        $result
        if ($result.Status -ceq 'PASS') { exit 0 }
        if ($result.Status -ceq 'FAIL') { exit $result.ExitCode }
        exit 1
    }

    $report = Get-ContinuityStatus $checkout $store $checkpoint $git $RelatedRoot
    Assert-Ready $report
    switch ($Action) {
        'resume' {
            Add-Member -InputObject $report -NotePropertyMembers @{ Task = $checkpoint.Task; Progress = $checkpoint.Progress }
            return $report
        }
        'close' {
            if (-not $ConfirmAcceptance) { throw 'Review the acceptance criteria and supply -ConfirmAcceptance.' }
            if ($state.requiredChecks.Count -eq 0) { throw 'Closure requires at least one acceptance check declared on the first save.' }
            foreach ($id in $state.requiredChecks) {
                if (-not ($report.Checks | Where-Object { $_.Id -ceq $id -and $_.Status -ceq 'PASS' -and $_.Kind -cin @('build', 'test') })) {
                    throw "Required check '$id' needs fresh PASS evidence before closure."
                }
            }
            $state.lifecycle = 'closed'
            return Get-SavedResult (Save-ContinuityCheckpoint $store $state $checkpoint.Task $checkpoint.Progress)
        }
        'export' {
            if (-not $OutputFile) { throw 'Supply a new local -OutputFile for the handoff.' }
            return Export-ContinuityHandoff $checkout $store $checkpoint $report $OutputFile
        }
    }
}
finally { $lock.Dispose() }
