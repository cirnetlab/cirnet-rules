#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory, Position = 0)]
    [ValidateSet('save', 'status', 'verify', 'resume', 'review', 'close', 'export', 'run', 'discover', 'bootstrap', 'bind')][string]$Action,
    [string]$Repository = (Get-Location).ProviderPath,
    [string]$CheckpointRoot = '.cirnet/tasks',
    [string]$TaskId,
    [ValidateSet('codex', 'claude-code')][string]$AgentFamily,
    [hashtable]$RelatedRoot = @{},
    [string]$TaskFile,
    [string]$ProgressFile,
    [string]$ExpectedCheckpointId,
    [string]$ProgressReason,
    [string]$ReferencesFile,
    [string[]]$RequiredCheck = @(),
    [string]$AcceptanceFile,
    [string]$ReviewFile,
    [switch]$ConfirmScope,
    [switch]$ConfirmHumanReview,
    [switch]$ConfirmProgressChange,
    [switch]$ConfirmContinuation,
    [switch]$RecoverPrevious,
    [switch]$AcceptGitDrift,
    [switch]$ConfirmStopped,
    [switch]$Corrective,
    [switch]$ConfirmAcceptance,
    [switch]$ConfirmExport,
    [string]$EvidenceId,
    [ValidateSet('edit', 'build', 'test', 'handoff')][string]$Kind,
    [string]$Executable,
    [string[]]$Arguments = @(),
    [string[]]$InputPath = @(),
    [string[]]$EnvironmentName = @(),
    [ValidateRange(1, 86400)][int]$TimeoutSeconds = 600,
    [string]$OutputFile,
    [string]$ClaimOutputPath,
    [string]$HandoffFile,
    [string]$ExpectedSha256,
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,63}\z')][string]$SessionKey,
    [switch]$ReplaceBinding,
    [switch]$Compact
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
if ($RecoverPrevious -and ($Action -cne 'resume' -or $HandoffFile -or $ConfirmContinuation)) {
    throw 'Use -RecoverPrevious only for a read-only local resume.'
}
if ($ConfirmContinuation -and ($Action -cnotin @('resume', 'bootstrap') -or $HandoffFile)) {
    throw 'Confirm continuation only when resuming from the original local checkpoint and worktree.'
}
if (($SessionKey -and $Action -cnotin @('discover', 'bootstrap', 'bind', 'save')) -or ($ReplaceBinding -and $Action -cne 'bind')) {
    throw 'Use -SessionKey with discover, bootstrap, bind or the first save and -ReplaceBinding only with bind.'
}
if ($Compact -and $Action -cne 'bootstrap') {
    throw 'Use -Compact with bootstrap.'
}
if ($Action -ceq 'bootstrap' -and $ConfirmContinuation -and -not $ExpectedCheckpointId) {
    throw 'Supply the reviewed -ExpectedCheckpointId with -ConfirmContinuation for bootstrap.'
}
if ($HandoffFile) {
    if ($Action -cnotin @('verify', 'resume') -or -not $AgentFamily) {
        throw 'Use verify or resume with -HandoffFile, -ExpectedSha256 and -AgentFamily.'
    }
    if ($Action -ceq 'resume' -and -not $ConfirmScope) { throw 'Confirm current read access with -ConfirmScope.' }
    return Read-ContinuityHandoff $HandoffFile $ExpectedSha256 $AgentFamily
}
if ($Action -cnotin @('status', 'verify') -and -not $AgentFamily) { throw 'Supply -AgentFamily.' }
$initializeBinding = $Action -ceq 'save' -and $SessionKey
if (($Action -cin @('resume', 'review', 'close', 'export', 'run', 'discover', 'bootstrap', 'bind') -or
        $initializeBinding -or $AcceptGitDrift -or $AcceptanceFile -or $ConfirmStopped) -and -not $ConfirmScope) {
    throw 'Use -ConfirmScope after reviewing current authorization for this operation.'
}
if ($Action -ceq 'export' -and (-not $ConfirmExport -or -not $OutputFile)) {
    throw 'Supply a new local -OutputFile and use -ConfirmExport after reviewing the approved recipient and portable fields.'
}
if (($AcceptanceFile -and $Action -cne 'save') -or ($ReviewFile -and $Action -cne 'review')) {
    throw 'Use -AcceptanceFile with save and -ReviewFile with review.'
}
if ($Corrective -and ($Action -cne 'run' -or $Kind -cne 'edit')) {
    throw 'Use -Corrective only for an authorized controlled edit.'
}
if ($ClaimOutputPath -and ($Action -cne 'run' -or $Kind -cnotin @('build','test') -or $EnvironmentName.Count)) {
    throw 'Use -ClaimOutputPath only for a direct claim verification build or test without custom environment inputs.'
}

$usesDiscovery = $Action -cin @('discover', 'bootstrap', 'bind')
if (($usesDiscovery -or $initializeBinding) -and $IsWindows -and ($Repository -match '^[\\/]{2}' -or $CheckpointRoot -match '^[\\/]{2}')) {
    throw 'Discovery and bindings require local worktree and checkpoint paths.'
}
$checkout = Get-ContinuityRepository (Get-ContinuityInputPath $Repository)
$selection = $null
if ($usesDiscovery -or $initializeBinding) {
    Import-Module (Join-Path $PSScriptRoot 'continuity/Discovery.psm1')
    Assert-NoCredential $SessionKey
}
if ($usesDiscovery) {
    $storage = Get-ContinuityStorageRoot $checkout $CheckpointRoot
    if ([IO.File]::Exists($storage)) { throw 'CheckpointRoot must be a directory.' }
    if ($Action -ceq 'bind') {
        Set-ContinuityBinding $checkout $storage $AgentFamily $TaskId $SessionKey $ExpectedCheckpointId -Replace:$ReplaceBinding
        exit 0
    }
    $selection = Get-ContinuitySelection $checkout $storage $AgentFamily $TaskId $SessionKey
    if ($Action -ceq 'discover' -or $selection.Result.Status -cne 'SELECTED') {
        $selection.Result
        exit 0
    }
    if ($ExpectedCheckpointId -and $ExpectedCheckpointId -cne $selection.Result.CheckpointId) {
        New-ContinuitySelection BLOCKED 'The selected checkpoint changed since review. Run bootstrap again and review the current context.'
        exit 0
    }
    $TaskId = $selection.Result.TaskId
    $store = $selection.Store
}
else { $store = Get-ContinuityStore $checkout $CheckpointRoot $TaskId }

function Assert-AgentFamily($Saved) {
    if ($AgentFamily -and $AgentFamily -cne $Saved.State.agentFamily) {
        throw 'Agent family does not match the saved checkpoint.'
    }
}

function Assert-OriginalWorktree($Saved) {
    if (-not (Test-ContinuityWorktree $checkout $Saved.State.git)) {
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

function Enable-TraceableReviews($State) {
    if ($State.schemaVersion -eq 2) {
        $State.schemaVersion = 3
        $State.completedReview = $null
        $State.progressReconciliation = $null
    }
}

function Get-UpdatedProgress($Checkpoint, $State) {
    if (-not $ProgressFile) { return $Checkpoint.Progress }
    $newProgress = Read-ContinuityInput $ProgressFile
    if ($newProgress -ceq $Checkpoint.Progress) { return $newProgress }
    if (-not $ConfirmScope -or $ExpectedCheckpointId -cne $Checkpoint.State.checkpointId) {
        throw 'For a progress update, supply -ConfirmScope and the reviewed -ExpectedCheckpointId.'
    }
    $before = Get-ContinuityNoteSections $Checkpoint.Progress progress
    $after = Get-ContinuityNoteSections $newProgress progress
    $changed = @(foreach ($name in @('Completed Work', 'Pending Work', 'Decisions & Blockers', 'Re-verification')) {
        $oldText = $before[$name] -replace '\r\n', "`n"
        $newText = $after[$name] -replace '\r\n', "`n"
        if ($newText -cne $oldText -and -not $newText.StartsWith($oldText + "`n", [StringComparison]::Ordinal)) { $name }
    })
    if ($changed.Count) {
        if (-not $ConfirmProgressChange -or [string]::IsNullOrWhiteSpace($ProgressReason)) {
            throw 'To revise previously recorded work, decisions or blockers, supply -ConfirmProgressChange and -ProgressReason.'
        }
        Assert-NoCredential $ProgressReason
        if ($State.schemaVersion -eq 3) {
            $State.progressReconciliation = [ordered]@{
                priorCheckpointId = $Checkpoint.State.checkpointId
                sections = $changed
                reason = $ProgressReason
            }
        }
        elseif (-not $after['Decisions & Blockers'].Contains($ProgressReason)) {
            throw 'For a legacy checkpoint, also record the reconciliation reason in Decisions & Blockers.'
        }
    }
    $newProgress
}

function Resolve-InterruptedMeasurements($State) {
    if (-not $ConfirmStopped) { return }
    foreach ($check in $State.checks.psbase.Values) {
        if ($check.phase -ceq 'running') {
            $check.phase = 'interrupted'
            $check.cause = 'operator-confirmed-stopped'
            $check.finishedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        }
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

if ($Action -cin @('status', 'verify', 'resume', 'bootstrap')) {
    $usingPrevious = $false
    try {
        $checkpoint = if ($selection) { $selection.Checkpoint } else { Read-ContinuityCheckpoint $store $TaskId }
    }
    catch {
        if (-not $RecoverPrevious) { throw }
        $checkpoint = Read-ContinuityCheckpoint $store $TaskId -Previous
        $usingPrevious = $true
    }
    Assert-AgentFamily $checkpoint
    if ($Action -cin @('resume', 'bootstrap')) { Assert-OriginalWorktree $checkpoint }
    # A failed live capture is a recovery blocker, not a failure of the sealed notes.
    try { $git = Get-ContinuityGitSnapshot $checkout }
    catch { $git = $null }
    $report = Get-ContinuityStatus $checkout $store $checkpoint $git $RelatedRoot
    Add-Member -InputObject $report -NotePropertyName AgentFamilyMatched -NotePropertyValue $(if ($AgentFamily) { $true } else { $null })
    if ($Action -cin @('resume', 'bootstrap')) {
        if ($selection) {
            if (-not (Test-ContinuitySelectionCurrent $checkout $storage $AgentFamily $SessionKey $selection)) {
                New-ContinuitySelection BLOCKED 'Task selection changed during recovery. Run bootstrap again.'
                exit 0
            }
            # Keep revision identity and classified drift without both full inventories.
            $report.Git = @{
                Matches = $report.Git.Matches
                Changes = $report.Git.Changes
                Worktree = $checkout.root
                Recorded = @{ head = $checkpoint.State.git.head; branch = $checkpoint.State.git.branch }
                Current = if ($git) { @{ head = $git.head; branch = $git.branch } }
            }
            Add-Member -InputObject $report -NotePropertyName SelectedBy -NotePropertyValue $selection.Result.SelectedBy
            if ($Compact) {
                $report.Checks = @($report.Checks | Select-Object Id, Kind, Status, Phase, ExitCode, Reason)
                $report.Acceptance = @($report.Acceptance | Select-Object Id, Criterion, Checks, Status, ReviewStatus, Reason)
                $report.CompletedWork = $report.CompletedWork | Select-Object Status, ReviewOrigin, Reason, NotesOrigin
            }
        }
        $canContinue = $ConfirmContinuation -and $report.Ready -and $report.Lifecycle -ceq 'open'
        if ($report.Lifecycle -ceq 'closed') { $report.Blockers += 'The task was recorded as closed.' }
        elseif (-not $ConfirmContinuation) { $report.Blockers += 'Current implementation authorization has not been confirmed.' }
        Add-Member -InputObject $report -NotePropertyMembers @{
            Task = $checkpoint.Task
            Progress = $checkpoint.Progress
            ContextRecovered = $true
            RecoveredPrevious = $usingPrevious
            CanContinue = $canContinue
            CanCorrect = $ConfirmContinuation -and $report.CorrectiveReady -and $report.Lifecycle -ceq 'open'
            RecoveryOnly = -not $canContinue
        }
        if ($usingPrevious) {
            $report.Status = 'BLOCKED'
            $report.CompletionStatus = 'NOT_RUN'
            $report.Ready = $report.CorrectiveReady = $report.AcceptanceReady = $false
            $report.Blockers += 'Read-only recovery. Reconcile the confirmed pointer and any intervening work before continuing.'
        }
    }
    $report
    if ($Action -ceq 'verify' -and $report.Status -cne 'PASS') { exit 1 }
    exit 0
}

$lock = Enter-ContinuityLock $store -Create:($Action -ceq 'save')
try {
    $checkpoint = $null
    if ($Action -ceq 'save' -and -not [IO.File]::Exists((Join-Path $store 'current.json')) -and
        ([IO.Directory]::Exists((Join-Path $store 'snapshots')) -or [IO.File]::Exists((Join-Path $store 'previous.json')))) {
        throw 'No current confirmed checkpoint. Preserve the existing task history and recover the previously published context.'
    }
    if ($Action -cne 'save' -or [IO.File]::Exists((Join-Path $store 'current.json'))) {
        $checkpoint = Read-ContinuityCheckpoint $store $TaskId
        Assert-AgentFamily $checkpoint
        Assert-OriginalWorktree $checkpoint
        if ($ExpectedCheckpointId -and $ExpectedCheckpointId -cne $checkpoint.State.checkpointId) {
            throw 'The checkpoint changed since review. Refresh status before writing.'
        }
    }
    if ($null -ne $checkpoint -and $checkpoint.State.lifecycle -ceq 'closed' -and $Action -cne 'export') {
        throw 'The task is already closed. Use status or verify to inspect its evidence.'
    }
    $git = Get-ContinuityGitSnapshot $checkout

    if ($Action -ceq 'save') {
        if ($initializeBinding -and $null -ne $checkpoint) {
            throw 'Use -SessionKey with save only at initialization. Use bind for an existing task.'
        }
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
            if ($AcceptanceFile -and $state.schemaVersion -ge 2) { throw 'The acceptance declaration is already fixed.' }
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
            $state.git = $git
            $state.gitFingerprint = Get-ContinuityGitFingerprint $git
        }
        if ($AcceptanceFile) { Set-AcceptanceDeclaration $state $task }
        Enable-TraceableReviews $state
        if ($null -ne $checkpoint) { $progress = Get-UpdatedProgress $checkpoint $state }
        Resolve-InterruptedMeasurements $state
        if ($initializeBinding) {
            $saved = Publish-ContinuityBinding $checkout ([IO.Path]::GetDirectoryName($store)) $SessionKey $state `
                -InitialStore $store -Task $task -Progress $progress
            return Get-SavedResult $saved
        }
        return Get-SavedResult (Save-ContinuityCheckpoint $store $state $task $progress)
    }

    $state = $checkpoint.State
    if ($Action -ceq 'run') {
        Assert-ContinuityId $EvidenceId
        if (-not ($Kind -and $Executable)) { throw 'Run requires -EvidenceId, -Kind and -Executable.' }
        $beforeRun = Get-ContinuityStatus $checkout $store $checkpoint $git $RelatedRoot
        $referenceProblems = $beforeRun.ReferenceProblems
        if (-not $beforeRun.Git.Matches -or $beforeRun.GitConflicted -or $referenceProblems.Count) {
            throw 'Reconcile Git drift, unmerged paths and pinned references before a controlled milestone.'
        }
        if (-not $ConfirmStopped -and @($beforeRun.Checks | Where-Object Phase -CEQ 'running').Count) {
            throw 'Confirm that interrupted processes stopped with -ConfirmStopped before running another milestone.'
        }
        if ($Kind -ceq 'edit' -and -not $Corrective -and @($beforeRun.Checks | Where-Object {
            $_.Status -cin @('NOT_RUN', 'BLOCKED')
        }).Count) {
            throw 'Resolve outstanding observations or use -Corrective for an authorized repair.'
        }
        if ($EvidenceId -cin $state.requiredChecks -and $Kind -cnotin @('build', 'test')) {
            throw 'A required acceptance check must be a build or test.'
        }
        if ($state.checks.Contains($EvidenceId) -and $state.checks[$EvidenceId].kind -cne $Kind) {
            throw 'Use a distinct evidence ID for each kind of milestone.'
        }
        Enable-TraceableReviews $state
        $progress = Get-UpdatedProgress $checkpoint $state
        Resolve-InterruptedMeasurements $state
        $measurement = New-ContinuityMeasurement $checkout $store $state $Kind $Executable $Arguments $InputPath $EnvironmentName
        if ($ClaimOutputPath) {
            $verifier = Join-Path $PSScriptRoot 'verify-claim.ps1'
            if ($measurement.phase -cne 'running' -or [IO.Path]::GetFileName($measurement.toolPath) -inotmatch '^pwsh(?:\.exe)?\z' -or
                $Arguments.Count -lt 4 -or $Arguments[0] -ine '-NoProfile' -or $Arguments[1] -ine '-NonInteractive' -or
                $Arguments[2] -ine '-File' -or $Arguments[3] -cne $verifier) {
                throw 'Capture requires direct execution of the approved claim verifier.'
            }
            $destination = Resolve-ContinuityFile $checkout.root $ClaimOutputPath
            Assert-NoCredential $ClaimOutputPath
            Assert-ContinuityDestination $checkout $destination -OwningWorktree
            $parent = Split-Path -Parent $destination
            if ((Test-ContinuityInside (Split-Path -Parent $store) $destination) -or
                (Test-Path -LiteralPath $destination) -or ![IO.Directory]::Exists($parent)) {
                throw 'Choose a new claim output in an existing ignored owner directory outside checkpoint storage.'
            }
        }
        $state.checks[$EvidenceId] = $measurement
        $confirmed = Save-ContinuityCheckpoint $store $state $checkpoint.Task $progress
        if ($measurement.phase -ceq 'running') {
            if ($ClaimOutputPath) {
                Invoke-ContinuityMeasurement $checkout $measurement $TimeoutSeconds -ClaimOutputPath $ClaimOutputPath
            }
            else { Invoke-ContinuityMeasurement $checkout $measurement $TimeoutSeconds }
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
            if (-not $ReviewFile -or -not $ConfirmHumanReview -or -not $ExpectedCheckpointId) {
                throw 'Supply -ReviewFile, -ExpectedCheckpointId and -ConfirmHumanReview for an authorized human review.'
            }
            if (-not $report.Git.Matches -or $report.GitConflicted -or $report.ReferenceProblems.Count) {
                throw 'Reconcile Git drift, unmerged paths and pinned references before recording an acceptance review.'
            }
            Enable-TraceableReviews $state
            $progress = Get-UpdatedProgress $checkpoint $state
            Set-ContinuityReviews $checkout $store $state $progress $ReviewFile
            return Get-SavedResult (Save-ContinuityCheckpoint $store $state $checkpoint.Task $progress)
        }
        'close' {
            if (-not $report.CorrectiveReady) { Assert-Ready $report }
            if (-not $ConfirmAcceptance) { throw 'Review the acceptance criteria and supply -ConfirmAcceptance.' }
            if ($state.schemaVersion -eq 1) { throw 'Declare the original criteria with save -AcceptanceFile before closure.' }
            if (-not $report.AcceptanceReady) {
                $pending = @($report.Acceptance | Where-Object Status -CNE 'PASS' | ForEach-Object { "$($_.Id)=$($_.Status)" })
                throw ('Closure requires current evidence for completed work and each criterion. ' +
                    ((@("completed-work=$($report.CompletedWork.Status)") + $pending) -join ', '))
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
