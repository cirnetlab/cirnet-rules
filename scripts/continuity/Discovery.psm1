#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../shared/Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot 'Repository.psm1')
Import-Module (Join-Path $PSScriptRoot 'Checkpoint.psm1')

$maxDiscoveryEntries = 128
$maxDiscoveryPayloadBytes = 16MB

function Get-BindingId($Repository, [string]$AgentFamily, [string]$SessionKey) {
    Get-ContinuityValueHash ([ordered]@{
        repository = $Repository.commonDirectory
        worktree = $Repository.root
        gitDirectory = $Repository.gitDirectory
        agentFamily = $AgentFamily
        sessionKey = $SessionKey
    })
}

function Read-Binding([string]$Root, [string]$BindingId) {
    $path = Resolve-CirnetPath $Root ".bindings/$BindingId/binding.json"
    $file = [IO.FileInfo]::new($path)
    if ($file.LinkType -eq 'HardLink') { throw 'Invalid binding file.' }
    try { $bytes = Read-ContinuityBytes $path 4096 }
    catch [IO.FileNotFoundException], [IO.DirectoryNotFoundException] { return }
    $binding = ConvertFrom-ContinuityJson $bytes -UniqueProperties
    Assert-Record $binding @('schemaVersion', 'bindingId', 'taskId', 'taskKey')
    if (-not (Test-Integer $binding.schemaVersion) -or $binding.schemaVersion -ne 1 -or
        $binding.bindingId -isnot [string] -or $binding.bindingId -cne $BindingId -or
        $binding.taskId -isnot [string] -or $binding.taskKey -isnot [string] -or
        $binding.taskKey -cnotmatch '^[a-f0-9]{32}\z') { throw 'Invalid binding identity.' }
    Assert-ContinuityId $binding.taskId
    $binding
}

function New-ContinuitySelection([string]$Status, [string]$Reason, [object[]]$Candidates = @(),
        [string]$SelectedBy = '', [int]$Rejected = 0) {
    [pscustomobject]@{
        Status = $Status
        Blockers = @(if ($Reason) { $Reason })
        TaskId = if ($Status -ceq 'SELECTED') { $Candidates[0].TaskId }
        CheckpointId = if ($Status -ceq 'SELECTED') { $Candidates[0].CheckpointId }
        SelectedBy = $SelectedBy
        Candidates = $Candidates
        RejectedCheckpoints = $Rejected
        ContextRecovered = $false
        CanContinue = $false
        CanCorrect = $false
    }
}

function Get-CandidateSummary($Checkpoint) {
    [pscustomobject]@{ TaskId = $Checkpoint.State.taskId; CheckpointId = $Checkpoint.State.checkpointId }
}

function Test-EligibleCheckpoint($Repository, $Checkpoint, [string]$AgentFamily) {
    $Checkpoint.State.lifecycle -ceq 'open' -and $Checkpoint.State.agentFamily -ceq $AgentFamily -and
        (Test-ContinuityWorktree $Repository $Checkpoint.State.git)
}

function Test-CheckpointArtifacts([string]$Store) {
    foreach ($name in @('current.json', 'previous.json', 'snapshots')) {
        try {
            $null = [IO.File]::GetAttributes((Join-Path $Store $name))
            return $true
        }
        catch [IO.FileNotFoundException], [IO.DirectoryNotFoundException] { }
    }
    $false
}

function Get-ContinuitySelection($Repository, [string]$Root, [string]$AgentFamily, [string]$TaskId, [string]$SessionKey) {
    $binding = $null
    if (-not $TaskId) {
        try { $binding = Read-Binding $Root (Get-BindingId $Repository $AgentFamily $SessionKey) }
        catch {
            return @{ Result = New-ContinuitySelection BLOCKED 'Local binding is invalid. Review it or explicitly select an approved Task ID.' }
        }
    }
    if ($TaskId -or ($SessionKey -and $binding)) {
        $selectedId = if ($TaskId) { $TaskId } else { $binding.taskId }
        Assert-ContinuityId $selectedId
        try {
            $store = Resolve-CirnetPath $Root $selectedId
            $checkpoint = Read-ContinuityCheckpoint $store $selectedId
            if (-not (Test-EligibleCheckpoint $Repository $checkpoint $AgentFamily) -or
                ($binding -and $binding.taskKey -cne $checkpoint.State.taskKey)) {
                throw 'Checkpoint identity or lifecycle mismatch.'
            }
        }
        catch {
            return @{ Result = New-ContinuitySelection BLOCKED 'Selected task is unavailable, closed, damaged or belongs to a different worktree, task key or agent family. Review the selection.' }
        }
        return @{
            Result = New-ContinuitySelection SELECTED '' @((Get-CandidateSummary $checkpoint)) $(if ($TaskId) { 'explicit' } else { 'binding' })
            Checkpoint = $checkpoint; Store = $store; Binding = $binding
        }
    }

    try {
        $entries = @([IO.DirectoryInfo]::new($Root).EnumerateFileSystemInfos() | Select-Object -First ($maxDiscoveryEntries + 1))
    }
    catch [IO.DirectoryNotFoundException] { $entries = @() }
    catch { return @{ Result = New-ContinuitySelection BLOCKED 'Checkpoint root cannot be enumerated. Review the approved storage location.' } }
    if ($entries.Count -gt $maxDiscoveryEntries) {
        return @{ Result = New-ContinuitySelection BLOCKED "Checkpoint root exceeds $maxDiscoveryEntries direct entries. Supply an approved Task ID or exact session binding." }
    }

    $candidates = [Collections.Generic.List[object]]::new()
    $seals = [ordered]@{}
    $remainingBytes = $maxDiscoveryPayloadBytes
    $rejected = 0
    foreach ($entry in $entries) {
        if ($entry.Name -ceq '.bindings') { continue }
        try {
            if ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Linked storage entry.' }
            if ($entry -isnot [IO.DirectoryInfo] -or -not (Test-CheckpointArtifacts $entry.FullName)) { continue }
            Assert-ContinuityId $entry.Name
            $candidateStore = Resolve-CirnetPath $Root $entry.Name
            $candidate = Read-ContinuityCheckpoint $candidateStore $entry.Name -RemainingBytes ([ref]$remainingBytes)
        }
        catch { $rejected++; continue }
        $seals.Add($entry.Name, $candidate.Seal)
        if (Test-EligibleCheckpoint $Repository $candidate $AgentFamily) {
            $candidates.Add((Get-CandidateSummary $candidate))
            $checkpoint = $candidate
            $store = $candidateStore
        }
    }
    $summaries = @($candidates | Sort-Object -Property TaskId -CaseSensitive)
    if ($rejected) {
        return @{ Result = New-ContinuitySelection BLOCKED 'A candidate is unreadable, damaged or exceeds the 16 MiB scan payload budget. Review the store or supply an approved Task ID.' $summaries '' $rejected }
    }
    if ($SessionKey -and $candidates.Count) {
        return @{ Result = New-ContinuitySelection AMBIGUOUS 'This session key has no binding. Select an eligible Task ID explicitly.' $summaries }
    }
    if ($candidates.Count -gt 1) {
        return @{ Result = New-ContinuitySelection AMBIGUOUS 'Select one eligible Task ID or use its verified session binding.' $summaries }
    }
    if ($binding -and ($candidates.Count -ne 1 -or $binding.taskId -cne $checkpoint.State.taskId -or
        $binding.taskKey -cne $checkpoint.State.taskKey)) {
        return @{ Result = New-ContinuitySelection BLOCKED 'Worktree binding does not match the sole eligible task. Review the binding or select an approved Task ID.' $summaries }
    }
    if ($candidates.Count -eq 0) {
        return @{ Result = New-ContinuitySelection NONE 'No verified open task for this worktree and agent family. Check the approved worktree and storage root.' }
    }
    @{
        Result = New-ContinuitySelection SELECTED '' $summaries $(if ($binding) { 'binding' } else { 'unique' })
        Checkpoint = $checkpoint; Store = $store; Binding = $binding
        ScanFingerprint = Get-ContinuityValueHash $seals
    }
}

function Test-ContinuitySelectionCurrent($Repository, [string]$Root, [string]$AgentFamily, [string]$SessionKey, $Selection) {
    try {
        if ($Selection.ContainsKey('ScanFingerprint')) {
            # Repeat ownership selection after live Git and evidence checks.
            $live = Get-ContinuitySelection $Repository $Root $AgentFamily '' $SessionKey
            return $live.Result.Status -ceq 'SELECTED' -and $live.Result.TaskId -ceq $Selection.Result.TaskId -and
                $live.ScanFingerprint -ceq $Selection.ScanFingerprint -and
                (Get-ContinuityValueHash $live.Binding) -ceq (Get-ContinuityValueHash $Selection.Binding)
        }
        if ($Selection.Result.SelectedBy -cne 'explicit') {
            $liveBinding = Read-Binding $Root (Get-BindingId $Repository $AgentFamily $SessionKey)
            if ((Get-ContinuityValueHash $liveBinding) -cne (Get-ContinuityValueHash $Selection.Binding)) { return $false }
        }
        $path = Resolve-CirnetPath $Selection.Store 'current.json'
        $pointer = ConvertFrom-ContinuityJson (Read-ContinuityBytes $path 4096) -UniqueProperties
        if ($Selection.Checkpoint.Seal -cne (Get-ContinuityValueHash $pointer)) { return $false }
        $path = Resolve-CirnetPath $Selection.Checkpoint.Directory 'manifest.json'
        $manifest = ConvertFrom-ContinuityJson (Read-ContinuityBytes $path 4096) -UniqueProperties
        $Selection.Checkpoint.Seal -ceq (Get-ContinuityValueHash $manifest)
    }
    catch { $false }
}

function Publish-ContinuityBinding($Repository, [string]$Root, [string]$SessionKey, $State,
        [switch]$Replace, [string]$InitialStore, [string]$Task, [string]$Progress) {
    # Both callers hold the task lock, so the lock order stays task then binding.
    $bindingId = Get-BindingId $Repository $State.agentFamily $SessionKey
    $bindingStore = Resolve-CirnetPath $Root ".bindings/$bindingId"
    $bindingLock = Enter-ContinuityLock $bindingStore -Create
    try {
        $path = Resolve-CirnetPath $bindingStore 'binding.json'
        $existing = if (-not $Replace) { Read-Binding $Root $bindingId }
        if ($existing -and ($existing.taskId -cne $State.taskId -or $existing.taskKey -cne $State.taskKey -or $InitialStore)) {
            throw 'This scope is already bound. Review the intended task before using bind -ReplaceBinding.'
        }
        if ($existing) {
            return [pscustomobject]@{ TaskId = $State.taskId; CheckpointId = $State.checkpointId; BindingPath = $path }
        }
        $record = [ordered]@{
            schemaVersion = 1; bindingId = $bindingId; taskId = $State.taskId; taskKey = $State.taskKey
        }
        $candidate = Join-Path $bindingStore ('.pending-' + [guid]::NewGuid().ToString('N'))
        Write-ContinuityNewFile $candidate (ConvertTo-ContinuityBytes $record)
        $saved = $null
        try {
            if ($InitialStore) { $saved = Save-ContinuityCheckpoint $InitialStore $State $Task $Progress }
            [IO.File]::Move($candidate, $path, [bool]$Replace)
        }
        catch {
            if ($saved) { throw "Checkpoint for '$($State.taskId)' was saved. Its session binding could not be published. Recover by Task ID." }
            throw
        }
        finally { [IO.File]::Delete($candidate) }
        if ($InitialStore) { return $saved }
        [pscustomobject]@{ TaskId = $State.taskId; CheckpointId = $State.checkpointId; BindingPath = $path }
    }
    finally { $bindingLock.Dispose() }
}

function Set-ContinuityBinding($Repository, [string]$Root, [string]$AgentFamily, [string]$TaskId, [string]$SessionKey,
        [string]$ExpectedCheckpointId, [switch]$Replace) {
    Assert-ContinuityId $TaskId
    $store = Resolve-CirnetPath $Root $TaskId
    $taskLock = Enter-ContinuityLock $store
    try {
        $checkpoint = Read-ContinuityCheckpoint $store $TaskId
        if (-not (Test-EligibleCheckpoint $Repository $checkpoint $AgentFamily)) {
            throw 'Bind only a verified open task from this worktree and agent family.'
        }
        if ($ExpectedCheckpointId -and $ExpectedCheckpointId -cne $checkpoint.State.checkpointId) {
            throw 'The checkpoint changed since review. Refresh discovery before binding.'
        }
        Publish-ContinuityBinding $Repository $Root $SessionKey $checkpoint.State -Replace:$Replace
    }
    finally { $taskLock.Dispose() }
}

Export-ModuleMember -Function Get-ContinuitySelection, New-ContinuitySelection, Test-ContinuitySelectionCurrent,
    Set-ContinuityBinding, Publish-ContinuityBinding
