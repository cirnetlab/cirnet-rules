#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../shared/Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot 'Repository.psm1')

$utf8 = [Text.UTF8Encoding]::new($false, $true)
$payloadNames = @('task.md', 'progress.md', 'state.json')
$taskHeadings = @('Original Task', 'Objectives', 'Acceptance Criteria', 'Architecture & Rules', 'Scope & Authorization', 'Plans & Specs')
$progressHeadings = @('Completed Work', 'Pending Work', 'Decisions & Blockers', 'Next Action', 'Re-verification')
$idPattern = '^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?\z'
$guidPattern = '^[a-f0-9]{32}\z'
$shaPattern = '^[a-f0-9]{64}\z'
$commitPattern = '^(?:[a-f0-9]{40}|[a-f0-9]{64})\z'

function Assert-ContinuityId([string]$Id) {
    if ($Id -cnotmatch $idPattern) { throw 'Use a lowercase task, repository or check ID with up to 64 letters, digits and hyphens.' }
    Assert-NoCredential $Id
}

function Assert-Record($Value, [string[]]$Keys) {
    if ($Value -isnot [Collections.IDictionary] -or $Value.psbase.Count -ne $Keys.Count) {
        throw 'Invalid checkpoint record fields.'
    }
    foreach ($key in $Keys) {
        if ($key -cnotin $Value.psbase.Keys) { throw 'Invalid checkpoint record fields.' }
    }
}

function Test-Integer($Value) {
    $Value -is [int] -or $Value -is [long]
}

function Assert-NoCredential([string]$Text) {
    $patterns = @(
        '(?i)\b(?:authorization\s*:\s*)?(?:bearer|basic)\s+[a-z0-9+/_.=-]{12,}',
        '(?i)\b(?:api[ _-]?key|(?:access[ _-]?|refresh[ _-]?|session[ _-]?)?token|password|passwd|secret(?:[ _-]?key)?|(?:session[ _-]?)?cookie)["'']?\s*[:=]\s*["'']?[^\s"'']{3,}',
        '(?i)(?:^|\s)--?(?:api[-_]?key|(?:access[-_]?)?token|password|secret|cookie)\s+[^\s]{3,}',
        '-----BEGIN [A-Z ]*PRIVATE KEY-----',
        '\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\b',
        '\b(?:gh[opusr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|sk-(?:proj-)?[A-Za-z0-9_-]{20,}|xox[baprs]-[A-Za-z0-9-]{12,}|AKIA[A-Z0-9]{16})\b',
        '(?i)\b[a-z][a-z0-9+.-]*://[^\s/@]+:[^\s/@]+@'
    )
    foreach ($pattern in $patterns) {
        if ($Text -match $pattern) { throw 'Checkpoint input appears to contain a credential. Supply redacted local input.' }
    }
}

function Get-ContinuityNoteSections([string]$Content, [ValidateSet('task', 'progress')][string]$Kind) {
    Assert-NoCredential $Content
    $headings = switch ($Kind) {
        'task' { $taskHeadings }
        'progress' { $progressHeadings }
    }
    $sections = [regex]::Matches($Content, '(?m)^## +([^\r\n]+)\r?$')
    $result = [ordered]@{}
    $bounds = @($sections | ForEach-Object { $_.Index }) + $Content.Length
    foreach ($heading in $Headings) {
        $indices = @(for ($i = 0; $i -lt $sections.Count; $i++) {
            if ($sections[$i].Groups[1].Value.TrimEnd() -ceq $heading) { $i }
        })
        if ($indices.Count -ne 1) { throw "Supply exactly one '## $heading' section." }
        $index = $indices[0]
        $start = $sections[$index].Index + $sections[$index].Length
        $text = $Content.Substring($start, $bounds[$index + 1] - $start).Trim()
        if (-not $text) { throw "Supply content in the '## $heading' section." }
        $result.Add($heading, $text)
    }
    $result
}

function Get-ContinuityCriterionText([string]$Task) {
    $text = (Get-ContinuityNoteSections $Task task)['Acceptance Criteria']
    $lines = @($text -split '\r?\n' | Where-Object { $_.Trim() })
    if ($lines[0] -match '^\s*(?:[-*+]|\d+[.)])\s+') {
        foreach ($line in $lines) {
            $criterion = [regex]::Match($line, '^\s*(?:[-*+]|\d+[.)])\s+(\S.*)$')
            if (-not $criterion.Success) {
                throw 'Declare each acceptance criterion as a separate one-line list item.'
            }
            $criterion.Groups[1].Value.TrimEnd()
        }
        return
    }
    if ($text -match '\n\s*\n' -or @($lines | Where-Object { $_ -match '^\s*(?:[-*+]|\d+[.)])\s+' }).Count) {
        throw 'Use an acceptance list with one criterion per line, or a single paragraph for one criterion.'
    }
    $text
}

function Assert-ContinuityCriteria($Criteria, [string[]]$Text, [string[]]$RequiredChecks) {
    if ($Criteria -isnot [array] -or $Criteria.Count -eq 0 -or $Criteria.Count -ne $Text.Count) {
        throw 'Declare one acceptance record for each criterion in the original task, in the same order.'
    }
    $ids = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $checkIds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($item in $Criteria) {
        Assert-Record $item @('id', 'checks')
        if ($item.id -isnot [string] -or $item.checks -isnot [array]) { throw 'Invalid acceptance criterion record.' }
        Assert-ContinuityId $item.id
        if (-not $ids.Add($item.id)) { throw 'Duplicate acceptance criterion ID.' }
        $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($id in $item.checks) {
            if ($id -isnot [string]) { throw 'Invalid acceptance check ID.' }
            Assert-ContinuityId $id
            if (-not $seen.Add($id)) { throw 'Duplicate check ID for an acceptance criterion.' }
            [void]$checkIds.Add($id)
        }
    }
    if ($null -ne $RequiredChecks -and
        ($RequiredChecks.Count -ne $checkIds.Count -or -not $checkIds.SetEquals($RequiredChecks))) {
        throw 'Acceptance criteria disagree with the required check IDs.'
    }
}

function Read-ContinuityBytes([string]$Path, [int]$MaxBytes = 67108864, [ref]$RemainingBytes) {
    $share = [IO.FileShare]::Read -bor [IO.FileShare]::Delete
    $file = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, $share)
    try {
        $length = $file.Length
        if ($length -gt $MaxBytes) { throw 'A continuity file exceeds its size limit.' }
        if ($null -ne $RemainingBytes) {
            if ($length -gt $RemainingBytes.Value) { throw 'A continuity file exceeds the remaining scan payload budget.' }
            $RemainingBytes.Value -= $length
        }
        $bytes = [byte[]]::new([int]$length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $file.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -eq 0) { throw 'A continuity file changed while being read.' }
            $offset += $read
        }
        if ($file.ReadByte() -ne -1) { throw 'A continuity file changed while being read.' }
        return ,$bytes
    }
    finally { $file.Dispose() }
}

function ConvertFrom-ContinuityJson([byte[]]$Bytes, [switch]$UniqueProperties) {
    try {
        $json = $utf8.GetString($Bytes)
        if ($UniqueProperties) {
            $settings = [Newtonsoft.Json.Linq.JsonLoadSettings]::new()
            $settings.DuplicatePropertyNameHandling = 'Error'
            $null = [Newtonsoft.Json.Linq.JObject]::Parse($json, $settings)
        }
        ConvertFrom-Json -InputObject $json -AsHashtable -Depth 24 -NoEnumerate
    }
    catch { throw 'Invalid continuity JSON.' }
}

function ConvertTo-ContinuityBytes($Data) {
    $json = ConvertTo-Json -InputObject $Data -Depth 24 -Compress
    return ,$utf8.GetBytes($json + [char]10)
}

function Get-ContinuityInputPath([string]$Path) {
    Resolve-CirnetRoot -Path $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path)
}

function Read-ContinuityInput([string]$Path) {
    $utf8.GetString((Read-ContinuityBytes (Get-ContinuityInputPath $Path) 1048576))
}

function Write-ContinuityNewFile([string]$Path, [byte[]]$Bytes) {
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        try {
            $stream.Write($Bytes, 0, $Bytes.Length)
            $stream.Flush($true)
        }
        finally { $stream.Dispose() }
    }
    catch {
        [IO.File]::Delete($Path)
        throw
    }
}

function Assert-ContinuityReferences($References, [switch]$Pinned) {
    Assert-Record $References @('documents', 'relatedRepositories')
    if ($References.documents -isnot [array] -or $References.relatedRepositories -isnot [array]) {
        throw 'References must contain documents and relatedRepositories arrays.'
    }
    $repositories = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    [void]$repositories.Add('primary')
    foreach ($related in $References.relatedRepositories) {
        Assert-Record $related @('id', 'revision', 'access')
        Assert-ContinuityId $related.id
        if (-not $repositories.Add($related.id) -or $related.revision -cnotmatch $commitPattern -or $related.access -cnotin @('read', 'edit')) {
            throw 'Invalid related repository, selected revision or recorded access.'
        }
    }
    $kinds = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($document in $References.documents) {
        $keys = @('repository', 'kind', 'path')
        if ($Pinned) { $keys += @('revision', 'sha256') }
        Assert-Record $document $keys
        if ($document.repository -isnot [string] -or -not $repositories.Contains($document.repository) -or
            $document.kind -cnotin @('plan', 'spec', 'architecture', 'rules') -or
            $document.path -isnot [string] -or [string]::IsNullOrWhiteSpace($document.path) -or
            -not $seen.Add(($document.repository, $document.kind, $document.path -join [char]0))) {
            throw 'Invalid or duplicate document reference.'
        }
        if ($Pinned -and ($document.sha256 -cnotmatch $shaPattern -or
                ($document.revision -cnotmatch $commitPattern -and -not ($document.repository -ceq 'primary' -and $document.revision -ceq '')))) {
            throw 'Invalid pinned document checksum or revision.'
        }
        [void]$kinds.Add($document.kind)
    }
    if (-not $kinds.Contains('architecture') -or -not $kinds.Contains('rules')) {
        throw 'Pin the applicable architecture and rules. Include the owning plans and specs when they exist.'
    }
}

function Assert-Measurement($Check) {
    $fields = @('kind', 'phase', 'executable', 'toolPath', 'toolSha256', 'arguments', 'inputs',
        'gitFingerprint', 'afterGitFingerprint', 'referenceFingerprint', 'startedAtUnixMs', 'finishedAtUnixMs',
        'exitCode', 'result', 'cause')
    if ($Check -is [Collections.IDictionary] -and $Check.Contains('environment')) {
        $fields += 'environment'
        if ($Check.environment -isnot [Collections.IDictionary]) { throw 'Invalid environment fingerprints.' }
        foreach ($name in $Check.environment.psbase.Keys) {
            Assert-ContinuityEnvironmentName $name
            if ($Check.environment[$name] -isnot [string] -or $Check.environment[$name] -cnotmatch $shaPattern) {
                throw 'Invalid environment fingerprint.'
            }
        }
    }
    if ($Check.Contains('stdoutSha256')) {
        $fields += 'stdoutSha256'
        if ($Check.phase -cne 'finished' -or $Check.stdoutSha256 -isnot [string] -or
            $Check.stdoutSha256 -cnotmatch $shaPattern) { throw 'Invalid measured stdout digest.' }
    }
    Assert-Record $Check $fields
    if ($Check.kind -cnotin @('edit', 'build', 'test', 'handoff') -or
        $Check.phase -cnotin @('running', 'finished', 'blocked', 'interrupted') -or
        $Check.executable -isnot [string] -or -not $Check.executable -or $Check.toolPath -isnot [string] -or
        $Check.toolSha256 -isnot [string] -or ($Check.toolSha256 -and $Check.toolSha256 -cnotmatch $shaPattern) -or
        $Check.arguments -isnot [array] -or $Check.inputs -isnot [Collections.IDictionary] -or
        $Check.gitFingerprint -cnotmatch $shaPattern -or $Check.referenceFingerprint -cnotmatch $shaPattern -or
        $Check.afterGitFingerprint -cnotmatch '^(?:[a-f0-9]{64})?\z' -or
        -not (Test-Integer $Check.startedAtUnixMs) -or $Check.startedAtUnixMs -le 0) {
        throw 'Invalid observed process evidence.'
    }
    foreach ($argument in $Check.arguments) {
        if ($argument -isnot [string]) { throw 'Invalid observed process argument.' }
    }
    foreach ($path in $Check.inputs.psbase.Keys) {
        if ($Check.inputs[$path] -cnotmatch $shaPattern) { throw 'Invalid process input fingerprint.' }
    }
    $expected = switch -CaseSensitive ($Check.phase) {
        'finished' {
            if (-not (Test-Integer $Check.exitCode) -or $Check.exitCode -lt [int]::MinValue -or $Check.exitCode -gt [int]::MaxValue -or
                -not $Check.toolPath -or -not $Check.toolSha256 -or -not $Check.afterGitFingerprint -or
                $Check.cause -cnotin @('', 'changed-during-run')) {
                throw 'Finished evidence requires an observed process exit and tool fingerprint.'
            }
            if ($Check.exitCode -eq 0) { 'PASS' } else { 'FAIL' }
        }
        'blocked' { 'BLOCKED' }
        default { 'NOT_RUN' }
    }
    if ($Check.result -cne $expected -or ($Check.phase -cne 'finished' -and $null -ne $Check.exitCode) -or
        ($Check.phase -ceq 'running' -and $null -ne $Check.finishedAtUnixMs) -or
        ($Check.phase -cne 'running' -and (-not (Test-Integer $Check.finishedAtUnixMs) -or $Check.finishedAtUnixMs -le 0))) {
        throw 'Process evidence status disagrees with its recorded observation.'
    }
}

function Assert-ContinuityEnvironmentName([string]$Name) {
    if ($Name -cnotmatch '^[A-Z_][A-Z0-9_]{0,127}\z' -or $Name -match 'TOKEN|SECRET|KEY|PASS|COOKIE|CREDENTIAL|AUTH|SESSION') {
        throw 'Declare only uppercase environment input names that do not identify a secret.'
    }
}

function Assert-ContinuityReview($Review, [switch]$LegacyAllowed) {
    if ($Review -isnot [Collections.IDictionary]) { throw 'Invalid review record.' }
    $fields = @('status', 'evidence', 'basis', 'reviewedAtUnixMs')
    $legacy = -not $Review.Contains('origin')
    if ($legacy -and -not $LegacyAllowed) { throw 'A traceable human review is required.' }
    if (-not $legacy) { $fields += 'origin' }
    Assert-Record $Review $fields
    if ($Review.status -cnotin @('PASS', 'FAIL', 'NOT_RUN', 'BLOCKED') -or
        $Review.basis -cnotmatch $shaPattern -or -not (Test-Integer $Review.reviewedAtUnixMs) -or
        $Review.reviewedAtUnixMs -le 0) { throw 'Invalid review status or scope.' }
    if ($legacy) {
        if ($Review.evidence -isnot [string] -or -not $Review.evidence.Trim()) { throw 'Invalid legacy review.' }
        Assert-NoCredential $Review.evidence
        return
    }
    if ($Review.origin -cne 'declared-human-review' -or $Review.evidence -isnot [Collections.IDictionary] -or
        ($Review.status -cin @('PASS', 'FAIL') -and $Review.evidence.psbase.Count -eq 0)) {
        throw 'A traceable human review must identify its evidence.'
    }
    foreach ($path in $Review.evidence.psbase.Keys) {
        if ($path -isnot [string] -or -not $path.Trim() -or
            $Review.evidence[$path] -isnot [string] -or $Review.evidence[$path] -cnotmatch $shaPattern) {
            throw 'Invalid review artifact fingerprint.'
        }
        Assert-NoCredential $path
    }
}

function Assert-ContinuityState($State, [string]$TaskId, [string]$CheckpointId, [string]$Task, [string]$Progress) {
    $fields = @('schemaVersion', 'taskId', 'taskKey', 'checkpointId', 'agentFamily', 'savedAtUnixMs',
        'lifecycle', 'git', 'gitFingerprint', 'references', 'requiredChecks', 'checks')
    if ($State -isnot [Collections.IDictionary]) { throw 'Invalid checkpoint state.' }
    if ($State.schemaVersion -in @(2, 3)) { $fields += @('criteria', 'reviews') }
    if ($State.schemaVersion -eq 3) { $fields += @('completedReview', 'progressReconciliation') }
    Assert-Record $State $fields
    if (-not (Test-Integer $State.schemaVersion) -or $State.schemaVersion -notin @(1, 2, 3) -or
        $State.taskId -cne $TaskId -or $State.taskKey -cnotmatch $guidPattern -or
        $State.checkpointId -cne $CheckpointId -or $State.agentFamily -cnotin @('codex', 'claude-code') -or
        -not (Test-Integer $State.savedAtUnixMs) -or $State.savedAtUnixMs -le 0 -or
        $State.lifecycle -cnotin @('open', 'closed') -or $State.requiredChecks -isnot [array] -or
        $State.checks -isnot [Collections.IDictionary]) {
        throw 'Invalid task state or unsupported checkpoint schema.'
    }
    $gitFields = @('worktree', 'gitDirectory', 'repository', 'branch', 'head', 'indexSha256',
        'entriesSha256', 'staged', 'unstaged', 'untracked', 'files')
    $git = $State.git
    if ($git -is [Collections.IDictionary] -and $git.Contains('unmerged')) { $gitFields += 'unmerged' }
    Assert-Record $git $gitFields
    if ($git.files -isnot [Collections.IDictionary] -or $git.entriesSha256 -cnotmatch $shaPattern -or
        $git.indexSha256 -cnotmatch '^(?:[a-f0-9]{64})?\z' -or $git.head -cnotmatch '^(?:[a-f0-9]{40}|[a-f0-9]{64})?\z' -or
        $State.gitFingerprint -cne (Get-ContinuityGitFingerprint $git)) {
        throw 'Invalid recorded Git state or fingerprint.'
    }
    foreach ($key in @('worktree', 'gitDirectory', 'repository', 'branch')) {
        if ($git[$key] -isnot [string]) { throw 'Invalid recorded Git identity.' }
    }
    foreach ($path in $git.files.psbase.Keys) {
        if ($git.files[$path] -cnotmatch '^(?:absent|directory|(?:file|link):[a-f0-9]{64})\z') { throw 'Invalid worktree file fingerprint.' }
    }
    foreach ($key in @('staged', 'unstaged', 'untracked', 'unmerged')) {
        if (-not $git.Contains($key)) { continue }
        if ($git[$key] -isnot [array]) { throw 'Invalid recorded Git path list.' }
        foreach ($path in $git[$key]) {
            if ($path -isnot [string] -or -not $git.files.Contains($path)) { throw 'Invalid recorded Git path.' }
        }
    }
    Assert-ContinuityReferences $State.references -Pinned
    foreach ($id in $State.requiredChecks + @($State.checks.psbase.Keys)) { Assert-ContinuityId $id }
    foreach ($measurement in $State.checks.psbase.Values) { Assert-Measurement $measurement }
    if ($State.schemaVersion -ge 2) {
        Assert-ContinuityCriteria $State.criteria @(Get-ContinuityCriterionText $Task) $State.requiredChecks
        if ($State.reviews -isnot [Collections.IDictionary]) { throw 'Invalid acceptance reviews.' }
        foreach ($id in $State.reviews.psbase.Keys) {
            if ($id -cnotin $State.criteria.id) { throw 'Review references an unknown criterion.' }
            Assert-ContinuityReview $State.reviews[$id] -LegacyAllowed
        }
    }
    if ($State.schemaVersion -eq 3 -and $null -ne $State.completedReview) { Assert-ContinuityReview $State.completedReview }
    if ($State.schemaVersion -eq 3 -and $null -ne $State.progressReconciliation) {
        $audit = $State.progressReconciliation
        Assert-Record $audit @('priorCheckpointId', 'sections', 'reason')
        if ($audit.priorCheckpointId -cnotmatch $guidPattern -or $audit.sections -isnot [array] -or
            $audit.sections.Count -eq 0 -or @($audit.sections | Where-Object { $_ -cnotin $progressHeadings }).Count -or
            $audit.reason -isnot [string] -or -not $audit.reason.Trim()) { throw 'Invalid progress reconciliation.' }
        Assert-NoCredential $audit.reason
    }
    if ($State.lifecycle -ceq 'closed') {
        if ($State.schemaVersion -eq 1 -and $State.requiredChecks.Count -eq 0) { throw 'Closed task has no acceptance evidence.' }
        foreach ($id in $State.requiredChecks) {
            if (-not $State.checks.Contains($id) -or $State.checks[$id].result -cne 'PASS' -or
                ($State.schemaVersion -ge 2 -and $State.checks[$id].kind -cnotin @('build', 'test'))) {
                throw 'Closed task has invalid acceptance evidence.'
            }
        }
        if ($State.schemaVersion -ge 2) {
            foreach ($item in $State.criteria) {
                if (-not $State.reviews.Contains($item.id) -or $State.reviews[$item.id].status -cne 'PASS' -or
                    ($State.schemaVersion -eq 3 -and -not $State.reviews[$item.id].Contains('origin')) -or
                    $State.reviews[$item.id].basis -cne (Get-ContinuityReviewBasis $State $item)) {
                    throw 'Closed task has an invalid criterion review.'
                }
            }
        }
        if ($State.schemaVersion -eq 3 -and ($null -eq $State.completedReview -or
            $State.completedReview.status -cne 'PASS' -or
            $State.completedReview.basis -cne (Get-ContinuityCompletedBasis $State $Progress))) {
            throw 'Closed task has an invalid completed work review.'
        }
    }
}

function Get-ContinuityReviewBasis($State, $Criterion) {
    $observations = [ordered]@{}
    foreach ($id in $Criterion.checks) {
        $observations[$id] = if ($State.checks.Contains($id)) { $State.checks[$id] } else { $null }
    }
    Get-ContinuityValueHash ([ordered]@{
        git = $State.gitFingerprint
        references = $State.references
        criterion = $Criterion
        observations = $observations
    })
}

function Get-ContinuityCompletedBasis($State, [string]$Progress) {
    Get-ContinuityValueHash ([ordered]@{
        git = $State.gitFingerprint
        references = $State.references
        completed = (Get-ContinuityNoteSections $Progress progress)['Completed Work']
    })
}

function Get-ContinuityStorageRoot($Repository, [string]$CheckpointRoot) {
    if (!$CheckpointRoot) {
        $CheckpointRoot = Get-ContinuityLocalStorage $Repository checkpoints
        if (!$CheckpointRoot) { $CheckpointRoot = '.cirnet/tasks' }
    }
    if (-not [IO.Path]::IsPathFullyQualified($CheckpointRoot)) { $CheckpointRoot = Join-Path $Repository.root $CheckpointRoot }
    $root = Resolve-CirnetRoot -Path $CheckpointRoot
    Assert-ContinuityDestination $Repository $root -Directory
    $root
}

function Get-ContinuityStore($Repository, [string]$CheckpointRoot, [string]$TaskId) {
    Assert-ContinuityId $TaskId
    Resolve-CirnetPath -Root (Get-ContinuityStorageRoot $Repository $CheckpointRoot) -Relative $TaskId
}

function Enter-ContinuityLock([string]$Store, [switch]$Create) {
    $lock = Resolve-CirnetPath -Root $Store -Relative '.lock'
    if (-not [IO.Directory]::Exists($Store)) {
        if (-not $Create) { throw 'No local checkpoint exists for this task.' }
        [void][IO.Directory]::CreateDirectory($Store)
    }
    if ($Create -and -not [IO.File]::Exists($lock) -and @([IO.Directory]::EnumerateFileSystemEntries($Store)).Count -gt 0) {
        throw 'The task directory contains unrelated files. Choose a different task ID or CheckpointRoot.'
    }
    $mode = if ($Create) { [IO.FileMode]::OpenOrCreate } else { [IO.FileMode]::Open }
    try { [IO.File]::Open($lock, $mode, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None) }
    catch [IO.IOException] { throw 'The checkpoint is busy or its lock is unavailable.' }
}

function Read-CheckpointPayload([string]$Store, $Pointer, [string]$TaskId, [ref]$RemainingBytes) {
    Assert-Record $Pointer @('schemaVersion', 'checkpointId', 'files')
    if (-not (Test-Integer $Pointer.schemaVersion) -or $Pointer.schemaVersion -ne 1 -or $Pointer.checkpointId -cnotmatch $guidPattern) {
        throw 'Invalid checkpoint pointer or unsupported schema.'
    }
    Assert-Record $Pointer.files $payloadNames
    $directory = Resolve-CirnetPath -Root $Store -Relative ('snapshots/' + $Pointer.checkpointId)
    $manifestPath = Resolve-CirnetPath -Root $directory -Relative 'manifest.json'
    $manifest = ConvertFrom-ContinuityJson (Read-ContinuityBytes $manifestPath 4096) -UniqueProperties
    $seal = Get-ContinuityValueHash $Pointer
    if ($seal -cne (Get-ContinuityValueHash $manifest)) { throw 'Checkpoint pointer and manifest disagree.' }
    $contents = @{}
    $budget = @{}
    if ($null -ne $RemainingBytes) { $budget.RemainingBytes = $RemainingBytes }
    foreach ($name in $payloadNames) {
        $path = Resolve-CirnetPath -Root $directory -Relative $name
        $contents[$name] = Read-ContinuityBytes $path @budget
        if ($Pointer.files[$name] -cne (Get-CirnetPayloadHash -Bytes $contents[$name])) {
            throw "Checkpoint hash mismatch for $name."
        }
    }
    $state = ConvertFrom-ContinuityJson $contents['state.json']
    $task = $utf8.GetString($contents['task.md'])
    $progress = $utf8.GetString($contents['progress.md'])
    Assert-ContinuityState $state $TaskId $Pointer.checkpointId $task $progress
    $null = Get-ContinuityNoteSections $task task
    $null = Get-ContinuityNoteSections $progress progress
    @{ State = $state; Task = $task; Progress = $progress; Directory = $directory; Seal = $seal }
}

function Read-ContinuityCheckpoint([string]$Store, [string]$TaskId, [switch]$Previous, [ref]$RemainingBytes) {
    $name = if ($Previous) { 'previous.json' } else { 'current.json' }
    $current = Resolve-CirnetPath -Root $Store -Relative $name
    if (-not [IO.File]::Exists($current)) { throw 'No confirmed checkpoint exists for this task.' }
    $budget = @{}
    if ($null -ne $RemainingBytes) { $budget.RemainingBytes = $RemainingBytes }
    $pointer = ConvertFrom-ContinuityJson (Read-ContinuityBytes $current 4096) -UniqueProperties
    Read-CheckpointPayload $Store $pointer $TaskId @budget
}

function Save-ContinuityCheckpoint([string]$Store, $State, [string]$Task, [string]$Progress) {
    $null = Get-ContinuityNoteSections $Task task
    $null = Get-ContinuityNoteSections $Progress progress
    $State.checkpointId = [guid]::NewGuid().ToString('N')
    $State.savedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    Assert-ContinuityState $State $State.taskId $State.checkpointId $Task $Progress
    $current = Resolve-CirnetPath -Root $Store -Relative 'current.json'
    $candidate = Resolve-CirnetPath -Root $Store -Relative ('.pending-' + $State.checkpointId + '.json')
    $previous = Resolve-CirnetPath -Root $Store -Relative 'previous.json'
    $directory = Resolve-CirnetPath -Root $Store -Relative ('snapshots/' + $State.checkpointId)
    if ([IO.Directory]::Exists($directory)) { throw 'Checkpoint ID collision.' }
    $contents = @{
        'task.md' = $utf8.GetBytes($Task)
        'progress.md' = $utf8.GetBytes($Progress)
        'state.json' = ConvertTo-ContinuityBytes $State
    }
    # Inspect decoded metadata. An ordinary file named "token" may have a digest.
    $git = $State.git
    $names = @($git.worktree, $git.gitDirectory, $git.repository, $git.branch) + @($git.files.psbase.Keys)
    $names += @($State.references.documents | ForEach-Object { $_.path })
    $names += @($State.checks.psbase.Values | ForEach-Object { $_.toolPath })
    Assert-NoCredential ($names -join [char]0)
    [void][IO.Directory]::CreateDirectory($directory)
    $hashes = [ordered]@{}
    foreach ($name in $payloadNames) {
        Write-ContinuityNewFile (Join-Path $directory $name) $contents[$name]
        $hashes[$name] = Get-CirnetPayloadHash -Bytes $contents[$name]
    }
    $pointer = [ordered]@{ schemaVersion = 1; checkpointId = $State.checkpointId; files = $hashes }
    $pointerBytes = ConvertTo-ContinuityBytes $pointer
    Write-ContinuityNewFile (Join-Path $directory 'manifest.json') $pointerBytes
    $manifest = ConvertFrom-ContinuityJson (Read-ContinuityBytes (Join-Path $directory 'manifest.json') 4096) -UniqueProperties
    $confirmed = Read-CheckpointPayload $Store $manifest $State.taskId
    Write-ContinuityNewFile $candidate $pointerBytes
    try {
        if ([IO.File]::Exists($current)) {
            $priorBytes = Read-ContinuityBytes $current 4096
            $prior = Read-CheckpointPayload $Store (ConvertFrom-ContinuityJson $priorBytes -UniqueProperties) $State.taskId
            if ($prior.State.taskKey -cne $State.taskKey) { throw 'Task identity changed during publication.' }
            [IO.File]::Replace($candidate, $current, $previous)
        }
        else { [IO.File]::Move($candidate, $current) }
    }
    finally { [IO.File]::Delete($candidate) }
    $confirmed
}

Export-ModuleMember -Function Assert-ContinuityId, Assert-ContinuityReferences, Assert-NoCredential,
    Assert-Record, Test-Integer, Get-ContinuityNoteSections, Get-ContinuityCriterionText,
    Assert-ContinuityCriteria, Assert-ContinuityEnvironmentName, Get-ContinuityReviewBasis, Get-ContinuityCompletedBasis,
    Read-ContinuityBytes, Read-ContinuityInput, ConvertFrom-ContinuityJson, ConvertTo-ContinuityBytes,
    Get-ContinuityInputPath, Write-ContinuityNewFile, Get-ContinuityStorageRoot, Get-ContinuityStore, Enter-ContinuityLock,
    Read-ContinuityCheckpoint, Save-ContinuityCheckpoint
