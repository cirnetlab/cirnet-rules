#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../shared/Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot 'Repository.psm1')
Import-Module (Join-Path $PSScriptRoot 'Checkpoint.psm1')

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
    if (-not [IO.File]::Exists($path)) {
        if ([IO.Directory]::Exists($path)) { throw 'Invalid binding file.' }
        return
    }
    $file = [IO.FileInfo]::new($path)
    if ($file.LinkType -eq 'HardLink') { throw 'Invalid binding file.' }
    $bytes = Read-ContinuityBytes $path 4096
    $settings = [Newtonsoft.Json.Linq.JsonLoadSettings]::new()
    $settings.DuplicatePropertyNameHandling = 'Error'
    $null = [Newtonsoft.Json.Linq.JObject]::Parse([Text.UTF8Encoding]::new($false, $true).GetString($bytes), $settings)
    $binding = ConvertFrom-ContinuityJson $bytes
    Assert-Record $binding @('schemaVersion', 'bindingId', 'taskId', 'taskKey')
    if (-not (Test-Integer $binding.schemaVersion) -or $binding.schemaVersion -ne 1 -or
        $binding.bindingId -isnot [string] -or $binding.bindingId -cne $BindingId -or
        $binding.taskId -isnot [string] -or $binding.taskKey -isnot [string] -or
        $binding.taskKey -cnotmatch '^[a-f0-9]{32}\z') { throw 'Invalid binding identity.' }
    Assert-ContinuityId $binding.taskId
    $binding
}

function Get-ApplicableBinding($Repository, [string]$Root, [string]$AgentFamily, [string]$SessionKey) {
    if ($SessionKey) {
        $session = Read-Binding $Root (Get-BindingId $Repository $AgentFamily $SessionKey)
        if ($session) { return $session }
    }
    Read-Binding $Root (Get-BindingId $Repository $AgentFamily '')
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

function Get-ContinuitySelection($Repository, [string]$Root, [string]$AgentFamily, [string]$TaskId, [string]$SessionKey) {
    $binding = $null
    if (-not $TaskId) {
        try { $binding = Get-ApplicableBinding $Repository $Root $AgentFamily $SessionKey }
        catch {
            return @{ Result = New-ContinuitySelection BLOCKED 'Local binding is invalid. Review it or explicitly select an approved Task ID.' }
        }
    }
    if ($TaskId -or $binding) {
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

    $candidates = [Collections.Generic.List[object]]::new()
    $rejected = 0
    if ([IO.Directory]::Exists($Root)) {
        foreach ($entry in [IO.DirectoryInfo]::new($Root).EnumerateDirectories()) {
            if ($entry.Name -cnotmatch '^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?\z') { continue }
            try {
                $candidateStore = Resolve-CirnetPath $Root $entry.Name
                if (-not ([IO.File]::Exists((Join-Path $candidateStore '.lock')) -or
                    [IO.File]::Exists((Join-Path $candidateStore 'current.json')) -or
                    [IO.Directory]::Exists((Join-Path $candidateStore 'snapshots')))) { continue }
                $candidate = Read-ContinuityCheckpoint $candidateStore $entry.Name
            }
            catch { $rejected++; continue }
            if (Test-EligibleCheckpoint $Repository $candidate $AgentFamily) {
                $candidates.Add((Get-CandidateSummary $candidate))
                # Retain a single verified payload. Candidate lists carry IDs only.
                $checkpoint = $candidate
                $store = $candidateStore
            }
        }
    }
    if ($candidates.Count -eq 1) {
        return @{
            Result = New-ContinuitySelection SELECTED '' $candidates.ToArray() 'unique' $rejected
            Checkpoint = $checkpoint; Store = $store; Binding = $null
        }
    }
    if ($candidates.Count -gt 1) {
        return @{ Result = New-ContinuitySelection AMBIGUOUS 'Select one eligible Task ID before recovery.' @($candidates | Sort-Object -Property TaskId -CaseSensitive) '' $rejected }
    }
    $status = if ($rejected) { 'BLOCKED' } else { 'NONE' }
    @{ Result = New-ContinuitySelection $status 'No verified open task for this worktree and agent family. Check the approved worktree and storage root.' @() '' $rejected }
}

function Test-ContinuitySelectionCurrent($Repository, [string]$Root, [string]$AgentFamily, [string]$SessionKey, $Selection) {
    try {
        if ($Selection.Result.SelectedBy -cne 'explicit') {
            $liveBinding = Get-ApplicableBinding $Repository $Root $AgentFamily $SessionKey
            if ((Get-ContinuityValueHash $liveBinding) -cne (Get-ContinuityValueHash $Selection.Binding)) { return $false }
        }
        $path = Resolve-CirnetPath $Selection.Store 'current.json'
        $pointer = ConvertFrom-ContinuityJson (Read-ContinuityBytes $path 4096)
        $Selection.Checkpoint.Seal -ceq (Get-ContinuityValueHash $pointer)
    }
    catch { $false }
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
        $bindingId = Get-BindingId $Repository $AgentFamily $SessionKey
        $bindingStore = Resolve-CirnetPath $Root ".bindings/$bindingId"
        $bindingLock = Enter-ContinuityLock $bindingStore -Create
        try {
            $path = Resolve-CirnetPath $bindingStore 'binding.json'
            $existing = if (-not $Replace) { Read-Binding $Root $bindingId }
            if ($existing -and ($existing.taskId -cne $TaskId -or $existing.taskKey -cne $checkpoint.State.taskKey)) {
                throw 'This scope is already bound. Review the intended task before using -ReplaceBinding.'
            }
            if (-not $existing) {
                $record = [ordered]@{
                    schemaVersion = 1; bindingId = $bindingId; taskId = $TaskId; taskKey = $checkpoint.State.taskKey
                }
                $candidate = Join-Path $bindingStore ('.pending-' + [guid]::NewGuid().ToString('N'))
                Write-ContinuityNewFile $candidate (ConvertTo-ContinuityBytes $record)
                try { [IO.File]::Move($candidate, $path, [bool]$Replace) }
                finally { [IO.File]::Delete($candidate) }
            }
            [pscustomobject]@{
                TaskId = $TaskId; CheckpointId = $checkpoint.State.checkpointId; BindingPath = $path
            }
        }
        finally { $bindingLock.Dispose() }
    }
    finally { $taskLock.Dispose() }
}

Export-ModuleMember -Function Get-ContinuitySelection, New-ContinuitySelection, Test-ContinuitySelectionCurrent, Set-ContinuityBinding
