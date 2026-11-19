#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory, Position = 0)]
    [ValidateSet('save', 'status', 'verify', 'resume', 'review', 'close', 'export', 'run')][string]$Action,
    [string]$Repository = (Get-Location).ProviderPath,
    [string]$CheckpointRoot = '.cirnet/tasks',
    [string]$TaskId,
    [ValidateSet('codex', 'claude-code')][string]$AgentFamily,
    [hashtable]$RelatedRoot = @{},
    [string]$TaskFile,
    [string]$ProgressFile,
    [string]$ReferencesFile,
    [string[]]$RequiredCheck = @(),
    [string]$AcceptanceFile,
    [string]$ReviewFile,
    [switch]$ConfirmScope,
    [switch]$ConfirmContinuation,
    [switch]$AcceptGitDrift,
    [switch]$ConfirmStopped,
    [switch]$ConfirmAcceptance,
    [switch]$ConfirmExport,
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
Import-Module (Join-Path $PSScriptRoot 'continuity/Handoff.psm1')

$Action = $Action.ToLowerInvariant()
if ($AgentFamily) { $AgentFamily = $AgentFamily.ToLowerInvariant() }
if ($Kind) { $Kind = $Kind.ToLowerInvariant() }
if ($ConfirmContinuation -and ($Action -cne 'resume' -or $HandoffFile)) {
    throw 'Confirm continuation only when resuming from the original local checkpoint and worktree.'
}
if ($HandoffFile) {
    if ($Action -cnotin @('verify', 'resume') -or -not $AgentFamily) {
        throw 'Use verify or resume with -HandoffFile, -ExpectedSha256 and -AgentFamily.'
    }
    if ($Action -ceq 'resume' -and -not $ConfirmScope) { throw 'Confirm current read access with -ConfirmScope.' }
    return Read-ContinuityHandoff $HandoffFile $ExpectedSha256 $AgentFamily
}
if ($Action -cnotin @('status', 'verify') -and -not $AgentFamily) { throw 'Supply -AgentFamily.' }
if (($Action -cin @('resume', 'review', 'close', 'export', 'run') -or $AcceptGitDrift -or $AcceptanceFile) -and -not $ConfirmScope) {
    throw 'Use -ConfirmScope after reviewing current authorization for this operation.'
}
if ($Action -ceq 'export' -and (-not $ConfirmExport -or -not $OutputFile)) {
    throw 'Supply a new local -OutputFile and use -ConfirmExport after reviewing the approved recipient and portable fields.'
}
if (($AcceptanceFile -and $Action -cne 'save') -or ($ReviewFile -and $Action -cne 'review')) {
    throw 'Use -AcceptanceFile with save and -ReviewFile with review.'
}

$checkout = Get-ContinuityRepository (Get-ContinuityInputPath $Repository)
$store = Get-ContinuityStore $checkout $CheckpointRoot $TaskId

function Assert-AgentFamily($Saved) {
    if ($AgentFamily -and $AgentFamily -cne $Saved.State.agentFamily) {
        throw 'Agent family does not match the saved checkpoint.'
    }
}

function Assert-OriginalWorktree($Saved) {
    $recorded = $Saved.State.git
    if ($checkout.root -cne $recorded.worktree -or $checkout.gitDirectory -cne $recorded.gitDirectory -or
        $checkout.commonDirectory -cne $recorded.repository) {
        throw 'The checkpoint belongs to a different worktree. Use the owning worktree or an authorized portable handoff.'
    }
}

function Set-AcceptanceDeclaration($State, [string]$Task) {
    $criteria = ConvertFrom-ContinuityJson (Read-ContinuityBytes (Get-ContinuityInputPath $AcceptanceFile) 1048576)
    Assert-ContinuityCriteria $criteria @(Get-ContinuityCriterionText $Task)
    $ids = @($criteria | ForEach-Object { $_.checks } | Sort-Object -CaseSensitive -Unique)
    foreach ($id in $State.requiredChecks) {
        if ($id -cnotin $ids) { throw 'The acceptance declaration must retain all existing required checks.' }
    }
    $State.schemaVersion = 2
    $State.requiredChecks = $ids
    $State.criteria = $criteria
    $State.reviews = [ordered]@{}
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

if ($Action -cin @('status', 'verify', 'resume')) {
    $checkpoint = Read-ContinuityCheckpoint $store $TaskId
    Assert-AgentFamily $checkpoint
    if ($Action -ceq 'resume') { Assert-OriginalWorktree $checkpoint }
    # A failed live capture is a recovery blocker, not a failure of the sealed notes.
    try { $git = Get-ContinuityGitSnapshot $checkout }
    catch { $git = $null }
    $report = Get-ContinuityStatus $checkout $store $checkpoint $git $RelatedRoot
    if ($Action -ceq 'resume') {
        $canContinue = $ConfirmContinuation -and $report.Ready -and $report.Lifecycle -ceq 'open'
        if ($report.Lifecycle -ceq 'closed') { $report.Blockers += 'The task was recorded as closed.' }
        elseif (-not $ConfirmContinuation) { $report.Blockers += 'Current implementation authorization has not been confirmed.' }
        Add-Member -InputObject $report -NotePropertyMembers @{
            Task = $checkpoint.Task
            Progress = $checkpoint.Progress
            ContextRecovered = $true
            CanContinue = $canContinue
            RecoveryOnly = -not $canContinue
        }
    }
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
        Assert-OriginalWorktree $checkpoint
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
            if (-not $AcceptanceFile -and -not $PSBoundParameters.ContainsKey('RequiredCheck')) {
                throw 'Supply an acceptance declaration with -AcceptanceFile.'
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
            if ($AcceptanceFile -and $state.schemaVersion -eq 2) { throw 'The acceptance declaration is already fixed.' }
            $comparison = Compare-ContinuityGit $state.git $git
            if (-not $comparison.Matches -and -not $AcceptGitDrift) {
                throw 'Git changed. Run verify, review the differences and save with -AcceptGitDrift -ConfirmScope.'
            }
            if (-not $comparison.Matches -and -not $ConfirmStopped -and
                @($state.checks.psbase.Values | Where-Object phase -CEQ 'running').Count) {
                throw 'Confirm the interrupted process has stopped with -ConfirmStopped before accepting Git changes.'
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
        if ($AcceptanceFile) { Set-AcceptanceDeclaration $state $task }
        return Get-SavedResult (Save-ContinuityCheckpoint $store $state $task $progress)
    }

    $state = $checkpoint.State
    if ($Action -ceq 'run') {
        Assert-ContinuityId $EvidenceId
        if (-not ($Kind -and $Executable)) { throw 'Run requires -EvidenceId, -Kind and -Executable.' }
        $beforeRun = Get-ContinuityStatus $checkout $store $checkpoint $git $RelatedRoot
        $referenceProblems = $beforeRun.ReferenceProblems
        if (-not $beforeRun.Git.Matches -or $referenceProblems.Count) {
            throw 'Verify the saved Git state and references before running a controlled milestone.'
        }
        if (-not $ConfirmStopped -and @($beforeRun.Checks | Where-Object Phase -CEQ 'running').Count) {
            throw 'Confirm that interrupted processes stopped with -ConfirmStopped before running another milestone.'
        }
        if ($Kind -ceq 'edit' -and @($beforeRun.Checks | Where-Object {
            $_.Id -cne $EvidenceId -and $_.Status -cin @('NOT_RUN', 'BLOCKED')
        }).Count) {
            throw 'Resolve outstanding verification and milestone blockers before making a controlled edit.'
        }
        if ($EvidenceId -cin $state.requiredChecks -and $Kind -cnotin @('build', 'test')) {
            throw 'A required acceptance check must be a build or test.'
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
                $changed = Compare-ContinuityGit $git $after
                $ownershipChange = @($changed.Changes.Fields | Where-Object {
                    $_ -cin @('worktree', 'gitDirectory', 'repository', 'branch', 'head', 'indexSha256', 'entriesSha256')
                }).Count -gt 0
                if ($Kind -ceq 'edit' -and $measurement.result -ceq 'PASS' -and -not $ownershipChange -and $referenceProblems.Count -eq 0) {
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
    switch ($Action) {
        'review' {
            if (-not $ReviewFile) { throw 'Supply -ReviewFile with the authorized criterion reviews and evidence.' }
            if (-not $report.Git.Matches -or $report.ReferenceProblems.Count) {
                throw 'Reconcile Git and pinned references before recording an acceptance review.'
            }
            Set-ContinuityReviews $state $ReviewFile
            $progress = if ($ProgressFile) { Read-ContinuityInput $ProgressFile } else { $checkpoint.Progress }
            return Get-SavedResult (Save-ContinuityCheckpoint $store $state $checkpoint.Task $progress)
        }
        'close' {
            Assert-Ready $report
            if (-not $ConfirmAcceptance) { throw 'Review the acceptance criteria and supply -ConfirmAcceptance.' }
            if ($state.schemaVersion -eq 1) { throw 'Declare the original criteria with save -AcceptanceFile before closure.' }
            if (-not $report.AcceptanceReady) {
                $pending = @($report.Acceptance | Where-Object Status -CNE 'PASS' | ForEach-Object { "$($_.Id)=$($_.Status)" })
                throw ('Each criterion needs a current PASS review and any declared checks before closure. ' + ($pending -join ', '))
            }
            $state.lifecycle = 'closed'
            return Get-SavedResult (Save-ContinuityCheckpoint $store $state $checkpoint.Task $checkpoint.Progress)
        }
        'export' {
            return Export-ContinuityHandoff $checkout $store $checkpoint $report $OutputFile
        }
    }
}
finally { $lock.Dispose() }
