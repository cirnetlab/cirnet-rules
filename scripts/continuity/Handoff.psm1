#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../shared/Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot 'Repository.psm1')
Import-Module (Join-Path $PSScriptRoot 'Checkpoint.psm1')

$shaPattern = '^[a-f0-9]{64}\z'
$commitPattern = '^(?:[a-f0-9]{40}|[a-f0-9]{64})?\z'
$statuses = @('PASS', 'FAIL', 'NOT_RUN', 'BLOCKED')
$kinds = @('edit', 'build', 'test', 'handoff')
$phases = @('running', 'finished', 'blocked', 'interrupted')
$blockerCodes = @('git-unavailable', 'git-drift', 'references-unavailable', 'evidence-requires-observation', 'closure-requires-review')
$maxBytes = 4194304

function Assert-PortableStrings($Record, [string[]]$Fields) {
    foreach ($field in $Fields) {
        if ($Record[$field] -isnot [string]) { throw 'Invalid portable field type.' }
    }
}

function Assert-PortableText($Text) {
    if ($Text -isnot [string] -or -not $Text.Trim()) { throw 'Invalid portable handoff text.' }
    Assert-NoCredential $Text
    if ($Text -match '(?m)^\s*(?:```|~~~|diff --git |@@ |--- [ab]/|\+\+\+ [ab]/)|^[\t ]{4}\S' -or
        $Text -match '(?i)(?:\b[A-Z]:[\\/]|\\\\[^\\\s]|(?:^|[\s"''(<])/(?:home|Users|root|tmp)/)' -or
        $Text -match '(?i)(?:\.codex[/\\](?:auth\.json|sessions|archived_sessions)|\.claude[/\\](?:projects|\.credentials\.json)|session(?:s|[-_ ](?:id|cookie|jsonl))\s*[:=]\s*\S+|\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\b|(?:token|password|secret|cookie|credential)\s*["'']?\s*[:=]\s*["'']?[^\s"'']+)') {
        throw 'Portable context contains a private path, credential, session identifier or source excerpt. Review the disclosure and supply shareable notes.'
    }
}

function Assert-PortableReferences($References) {
    Assert-ContinuityReferences $References -Pinned
    foreach ($related in $References.relatedRepositories) {
        Assert-PortableStrings $related @('id', 'revision', 'access')
        Assert-PortableText $related.id
    }
    foreach ($doc in $References.documents) {
        Assert-PortableStrings $doc @('repository', 'kind', 'path', 'revision', 'sha256')
        Assert-PortableText $doc.path
        if ($doc.path -match '[\\:\p{Cc}]' -or
            @($doc.path.Split('/') | Where-Object { $_ -in @('', '.', '..', '.git', '.codex', '.claude') -or
                $_ -match '^(?:\.env(?:\.|$)|credentials?(?:\.|$)|secrets?(?:\.|$))' }).Count) {
            throw 'A portable document reference must use a shareable repository-relative path.'
        }
    }
}

function Assert-PortableCheck($Check) {
    Assert-Record $Check @('id', 'kind', 'phase', 'status', 'recordedStatus', 'exitCode',
        'startedAtUnixMs', 'finishedAtUnixMs',
        'gitFingerprint', 'afterGitFingerprint', 'referenceFingerprint', 'commandSha256', 'inputsSha256', 'measurementSha256')
    Assert-PortableStrings $Check @('id', 'kind', 'phase', 'status', 'recordedStatus',
        'gitFingerprint', 'afterGitFingerprint', 'referenceFingerprint', 'commandSha256', 'inputsSha256', 'measurementSha256')
    Assert-ContinuityId $Check.id
    Assert-PortableText $Check.id
    if ($Check.kind -cnotin $kinds -or $Check.phase -cnotin $phases -or $Check.status -cnotin $statuses -or
        $Check.afterGitFingerprint -cnotmatch '^(?:[a-f0-9]{64})?\z') { throw 'Invalid portable check.' }
    foreach ($field in @('gitFingerprint', 'referenceFingerprint', 'commandSha256', 'inputsSha256', 'measurementSha256')) {
        if ($Check[$field] -cnotmatch $shaPattern) { throw 'Invalid portable check fingerprint.' }
    }
    if (-not (Test-Integer $Check.startedAtUnixMs) -or $Check.startedAtUnixMs -le 0 -or
        ($Check.phase -ceq 'running' -and $null -ne $Check.finishedAtUnixMs) -or
        ($Check.phase -cne 'running' -and (-not (Test-Integer $Check.finishedAtUnixMs) -or $Check.finishedAtUnixMs -le 0))) {
        throw 'Invalid portable check timestamps.'
    }
    if ($Check.phase -ceq 'finished') {
        if (-not (Test-Integer $Check.exitCode) -or $Check.exitCode -lt [int]::MinValue -or
            $Check.exitCode -gt [int]::MaxValue -or -not $Check.afterGitFingerprint) { throw 'Missing portable process exit evidence.' }
        $recorded = if ($Check.exitCode -eq 0) { 'PASS' } else { 'FAIL' }
    }
    else {
        if ($null -ne $Check.exitCode) { throw 'Incomplete check has an unexpected exit code.' }
        $recorded = if ($Check.phase -ceq 'blocked') { 'BLOCKED' } else { 'NOT_RUN' }
    }
    if ($Check.recordedStatus -cne $recorded -or $Check.status -cnotin @($recorded, 'NOT_RUN')) {
        throw 'Portable check status disagrees with the recorded observation.'
    }
}

function Assert-PortableHandoff($Data, [string]$AgentFamily) {
    if ($Data -isnot [Collections.IDictionary] -or -not (Test-Integer $Data.schemaVersion) -or
        $Data.schemaVersion -notin @(1, 2)) { throw 'Unsupported portable handoff schema.' }
    $fields = @('schemaVersion', 'format', 'taskKey', 'checkpointId', 'checkpointSeal', 'agentFamily',
        'lifecycle', 'savedAtUnixMs', 'checks')
    if ($Data.schemaVersion -eq 1) { $fields += 'gitHead' }
    else { $fields += @('taskId', 'task', 'progress', 'git', 'references', 'acceptance', 'blockersAtExport', 'readyAtExport', 'exportedAtUnixMs') }
    Assert-Record $Data $fields
    Assert-PortableStrings $Data @('format', 'taskKey', 'checkpointId', 'checkpointSeal', 'agentFamily', 'lifecycle')
    if ($Data.format -cne 'cirnet-task-handoff' -or $Data.taskKey -cnotmatch '^[a-f0-9]{32}\z' -or
        $Data.checkpointId -cnotmatch '^[a-f0-9]{32}\z' -or $Data.checkpointSeal -cnotmatch $shaPattern -or
        $Data.agentFamily -cne $AgentFamily -or $Data.lifecycle -cnotin @('open', 'closed') -or
        -not (Test-Integer $Data.savedAtUnixMs) -or $Data.savedAtUnixMs -le 0 -or $Data.checks -isnot [array]) {
        throw 'Invalid portable handoff metadata or incompatible agent family.'
    }
    if ($Data.schemaVersion -eq 1) {
        Assert-PortableStrings $Data @('gitHead')
        if ($Data.gitHead -cnotmatch $commitPattern) { throw 'Invalid portable Git revision.' }
        foreach ($check in $Data.checks) {
            Assert-Record $check @('kind', 'status')
            Assert-PortableStrings $check @('kind', 'status')
            if ($check.kind -cnotin $kinds -or $check.status -cnotin $statuses) { throw 'Invalid portable check.' }
        }
        return
    }
    if (-not (Test-Integer $Data.exportedAtUnixMs) -or $Data.exportedAtUnixMs -le 0 -or
        $Data.readyAtExport -isnot [bool] -or $Data.acceptance -isnot [array] -or
        $Data.blockersAtExport -isnot [array] -or $Data.readyAtExport -ne ($Data.blockersAtExport.Count -eq 0)) {
        throw 'Invalid portable handoff state.'
    }
    foreach ($code in $Data.blockersAtExport) {
        if ($code -isnot [string] -or $code -cnotin $blockerCodes) { throw 'Invalid portable blocker.' }
    }
    Assert-PortableStrings $Data @('taskId')
    Assert-ContinuityId $Data.taskId
    Assert-PortableText $Data.taskId
    Assert-Record $Data.task @('Original Task', 'Objectives', 'Acceptance Criteria', 'Architecture & Rules', 'Scope & Authorization', 'Plans & Specs')
    Assert-Record $Data.progress @('Completed Work', 'Pending Work', 'Decisions & Blockers', 'Next Action', 'Re-verification')
    foreach ($value in @($Data.task.psbase.Values) + @($Data.progress.psbase.Values)) { Assert-PortableText $value }
    Assert-PortableReferences $Data.references
    Assert-Record $Data.git @('repositoryFingerprint', 'worktreeFingerprint', 'branch', 'head', 'fingerprint',
        'indexSha256', 'entriesSha256', 'currentFingerprint', 'changedFields', 'changedFileCount')
    Assert-PortableStrings $Data.git @('repositoryFingerprint', 'worktreeFingerprint', 'branch', 'head', 'fingerprint',
        'indexSha256', 'entriesSha256', 'currentFingerprint')
    foreach ($field in @('repositoryFingerprint', 'worktreeFingerprint', 'fingerprint', 'entriesSha256')) {
        if ($Data.git[$field] -cnotmatch $shaPattern) { throw 'Invalid portable Git fingerprint.' }
    }
    if ($Data.git.head -cnotmatch $commitPattern -or
        $Data.git.indexSha256 -cnotmatch '^(?:[a-f0-9]{64})?\z' -or $Data.git.currentFingerprint -cnotmatch '^(?:[a-f0-9]{64})?\z' -or
        $Data.git.changedFields -isnot [array] -or -not (Test-Integer $Data.git.changedFileCount) -or $Data.git.changedFileCount -lt 0) {
        throw 'Invalid portable Git state.'
    }
    if ($Data.git.branch) { Assert-PortableText $Data.git.branch }
    foreach ($field in $Data.git.changedFields) {
        if ($field -isnot [string] -or $field -cnotin @('worktree', 'gitDirectory', 'repository', 'branch', 'head', 'indexSha256', 'entriesSha256')) {
            throw 'Invalid portable Git difference.'
        }
    }
    $checks = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::Ordinal)
    foreach ($check in $Data.checks) {
        Assert-PortableCheck $check
        if (-not $checks.TryAdd($check.id, $check)) { throw 'Duplicate portable check.' }
    }
    $criterionIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($item in $Data.acceptance) {
        Assert-Record $item @('id', 'checks', 'status', 'reviewStatus')
        Assert-PortableStrings $item @('id', 'status', 'reviewStatus')
        Assert-ContinuityId $item.id
        Assert-PortableText $item.id
        if (-not $criterionIds.Add($item.id) -or $item.checks -isnot [array] -or
            $item.status -cnotin $statuses -or $item.reviewStatus -cnotin $statuses) { throw 'Invalid portable acceptance record.' }
        if ($item.status -ceq 'PASS' -and $item.reviewStatus -cne 'PASS') { throw 'Portable acceptance lacks a PASS review.' }
        foreach ($id in $item.checks) {
            if ($id -isnot [string]) { throw 'Invalid portable acceptance check ID.' }
            Assert-ContinuityId $id
            Assert-PortableText $id
            if ($item.status -ceq 'PASS' -and (-not $checks.ContainsKey($id) -or
                $checks[$id].status -cne 'PASS' -or $checks[$id].kind -cnotin @('build', 'test'))) {
                throw 'Portable acceptance lacks a linked PASS observation.'
            }
        }
    }
}

function Export-ContinuityHandoff($Repository, [string]$Store, $Checkpoint, $Report, [string]$Path) {
    $output = Get-ContinuityInputPath $Path
    Assert-ContinuityDestination $Repository $output
    if (Test-ContinuityInside $Store $output) { throw 'Choose a new export file outside the checkpoint store.' }
    $state = $Checkpoint.State
    $changes = $Report.Git.Changes
    $projection = [ordered]@{
        schemaVersion = 2
        format = 'cirnet-task-handoff'
        taskId = $state.taskId
        taskKey = $state.taskKey
        checkpointId = $state.checkpointId
        checkpointSeal = $Checkpoint.Seal
        agentFamily = $state.agentFamily
        lifecycle = $state.lifecycle
        savedAtUnixMs = $state.savedAtUnixMs
        exportedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        readyAtExport = $Report.Ready
        blockersAtExport = @(
            if ($null -eq $Report.Git.Current) { 'git-unavailable' }
            elseif (-not $Report.Git.Matches) { 'git-drift' }
            if ($Report.ReferenceProblems.Count) { 'references-unavailable' }
            if (@($Report.Checks | Where-Object { $_.Status -cin @('NOT_RUN', 'BLOCKED') }).Count) { 'evidence-requires-observation' }
            if ($state.lifecycle -ceq 'closed' -and -not $Report.AcceptanceReady) { 'closure-requires-review' }
        )
        task = Get-ContinuityNoteSections $Checkpoint.Task task
        progress = Get-ContinuityNoteSections $Checkpoint.Progress progress
        git = [ordered]@{
            repositoryFingerprint = Get-ContinuityValueHash $state.git.repository
            worktreeFingerprint = Get-ContinuityValueHash $state.git.worktree
            branch = $state.git.branch
            head = $state.git.head
            fingerprint = $state.gitFingerprint
            indexSha256 = $state.git.indexSha256
            entriesSha256 = $state.git.entriesSha256
            currentFingerprint = if ($null -ne $Report.Git.Current) { Get-ContinuityGitFingerprint $Report.Git.Current } else { '' }
            changedFields = @(if ($null -ne $changes) { $changes.Fields })
            changedFileCount = if ($null -ne $changes) { $changes.Files.Count } else { 0 }
        }
        references = $state.references # Nested fields are checked by the reference schema.
        checks = @(foreach ($check in $Report.Checks) {
            $evidence = $check.Evidence
            [ordered]@{
                id = $check.Id; kind = $check.Kind; phase = $check.Phase
                status = $check.Status; recordedStatus = $check.RecordedStatus; exitCode = $check.ExitCode
                startedAtUnixMs = $evidence.startedAtUnixMs; finishedAtUnixMs = $evidence.finishedAtUnixMs
                gitFingerprint = $evidence.gitFingerprint; afterGitFingerprint = $evidence.afterGitFingerprint
                referenceFingerprint = $evidence.referenceFingerprint
                commandSha256 = Get-ContinuityValueHash @($evidence.executable, $evidence.toolPath, $evidence.toolSha256, $evidence.arguments)
                inputsSha256 = Get-ContinuityValueHash $evidence.inputs
                measurementSha256 = Get-ContinuityValueHash $evidence
            }
        })
        acceptance = @(foreach ($item in $Report.Acceptance) {
            if ($null -ne $item.Id) {
                [ordered]@{ id = $item.Id; checks = $item.Checks; status = $item.Status; reviewStatus = $item.ReviewStatus }
            }
        })
    }
    # Validate all selected data before writing even a temporary file.
    Assert-PortableHandoff $projection $state.agentFamily
    $bytes = ConvertTo-ContinuityBytes $projection
    if ($bytes.Length -gt $maxBytes) { throw 'The portable handoff exceeds its size limit.' }
    $sha = Get-CirnetPayloadHash -Bytes $bytes
    $candidate = $output + '.pending-' + [guid]::NewGuid().ToString('N')
    Write-ContinuityNewFile $candidate $bytes
    try {
        $null = Read-ContinuityHandoff $candidate $sha $state.agentFamily
        [IO.File]::Move($candidate, $output)
    }
    finally { [IO.File]::Delete($candidate) }
    [pscustomobject]@{ Status = 'PASS'; OutputFile = $output; Sha256 = $sha; CheckpointId = $state.checkpointId }
}

function Read-ContinuityHandoff([string]$Path, [string]$ExpectedSha256, [string]$AgentFamily) {
    if ($ExpectedSha256 -cnotmatch $shaPattern) { throw 'Supply the trusted SHA-256 from export using -ExpectedSha256.' }
    $bytes = Read-ContinuityBytes (Get-ContinuityInputPath $Path) $maxBytes
    if ((Get-CirnetPayloadHash -Bytes $bytes) -cne $ExpectedSha256) { throw 'Portable handoff hash mismatch.' }
    $data = ConvertFrom-ContinuityJson $bytes
    Assert-PortableHandoff $data $AgentFamily
    [pscustomobject]@{
        Status = 'PASS'; IntegrityVerified = $true; Sha256 = $ExpectedSha256; Handoff = $data
        ContextRecovered = $data.schemaVersion -eq 2
        CanContinue = $false; RequiresOriginalWorktree = $true; RequiresLocalCheckpoint = $true
        Blockers = @('Recheck current authorization, the local checkpoint, the worktree, pinned documents and evidence before continuing.')
    }
}

Export-ModuleMember -Function Export-ContinuityHandoff, Read-ContinuityHandoff
