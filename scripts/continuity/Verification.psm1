#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../shared/Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot 'Repository.psm1')
Import-Module (Join-Path $PSScriptRoot 'Checkpoint.psm1')

function Get-ReferenceRoots($Repository, $References, [hashtable]$RelatedRoot) {
    $roots = @{ primary = $Repository }
    foreach ($related in $References.relatedRepositories) {
        if (-not $RelatedRoot.ContainsKey($related.id) -or $RelatedRoot[$related.id] -isnot [string]) {
            throw "Supply the currently authorized -RelatedRoot for '$($related.id)'."
        }
        $checkout = Get-ContinuityRepository (Get-ContinuityInputPath $RelatedRoot[$related.id])
        if ((Get-ContinuityHead $checkout) -cne $related.revision) {
            throw "The related checkout '$($related.id)' differs from its selected revision."
        }
        $roots.Add($related.id, $checkout)
    }
    foreach ($id in $roots.Keys) {
        $paths = @($References.documents | Where-Object repository -CEQ $id | ForEach-Object { $_.path })
        if (Test-ContinuityReferenceConflict $roots[$id] $paths) {
            throw "A pinned document in '$id' has unmerged Git index entries."
        }
    }
    $roots
}

function Get-DocumentDigest($Checkout, [string]$Relative, [string]$Store) {
    $path = Resolve-ContinuityFile $Checkout.root $Relative
    if (Test-ContinuityInside (Split-Path -Parent $Store) $path) {
        throw 'A reference or check input cannot use the checkpoint store as evidence.'
    }
    Get-ContinuityFileDigest $path
}

function New-ContinuityReferences($Repository, [string]$Store, [string]$Path, [hashtable]$RelatedRoot, [string]$PrimaryHead) {
    $config = ConvertFrom-ContinuityJson (Read-ContinuityBytes (Get-ContinuityInputPath $Path) 1048576)
    Assert-ContinuityReferences $config
    $roots = Get-ReferenceRoots $Repository $config $RelatedRoot
    $revisions = @{ primary = $PrimaryHead }
    foreach ($related in $config.relatedRepositories) { $revisions.Add($related.id, $related.revision) }
    $documents = @(
        foreach ($doc in $config.documents) {
            [ordered]@{
                repository = $doc.repository
                kind = $doc.kind
                path = $doc.path
                revision = $revisions[$doc.repository]
                sha256 = Get-DocumentDigest $roots[$doc.repository] $doc.path $Store
            }
        }
    )
    [ordered]@{ documents = $documents; relatedRepositories = $config.relatedRepositories }
}

function Get-ContinuityReferenceProblems($Repository, [string]$Store, $References, [hashtable]$RelatedRoot) {
    try { $roots = Get-ReferenceRoots $Repository $References $RelatedRoot }
    catch { return $_.Exception.Message }
    foreach ($doc in $References.documents) {
        try {
            $sha = Get-DocumentDigest $roots[$doc.repository] $doc.path $Store
            if ($sha -cne $doc.sha256) { "$($doc.kind) reference changed in '$($doc.repository)'." }
        }
        catch { "$($doc.kind) reference is unavailable in '$($doc.repository)'." }
    }
}

function Find-ContinuityExecutable([string]$Name) {
    $command = Get-Command -Name ([WildcardPattern]::Escape($Name)) -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -eq $command) { return '' }
    if ($IsWindows -and [IO.Path]::GetExtension($command.Path) -notin @('.exe', '.com')) { return '' }
    $command.Path
}

function Get-ContinuityCheckInputs($Repository, [string]$Store, [string[]]$Paths) {
    $inputs = [Collections.Specialized.OrderedDictionary]::new([StringComparer]::Ordinal)
    foreach ($path in $Paths) {
        if ($inputs.Contains($path)) { throw 'Duplicate check input path.' }
        Assert-NoCredential $path
        $inputs.Add($path, (Get-DocumentDigest $Repository $path $Store))
    }
    $inputs
}

function Get-EnvironmentFingerprint([string]$Name) {
    Assert-ContinuityEnvironmentName $Name
    $value = [Environment]::GetEnvironmentVariable($Name)
    Assert-NoCredential $value
    Get-ContinuityValueHash ([ordered]@{ value = $value })
}

function Get-ProcessInputsMatch($Repository, [string]$Store, $Measurement) {
    try {
        $tool = Find-ContinuityExecutable $Measurement.executable
        if (-not $tool -or $tool -cne $Measurement.toolPath -or (Get-CirnetHash -Path $tool) -cne $Measurement.toolSha256) { return $false }
        foreach ($path in $Measurement.inputs.psbase.Keys) {
            if ((Get-DocumentDigest $Repository $path $Store) -cne $Measurement.inputs[$path]) { return $false }
        }
        if ($Measurement.Contains('environment')) {
            foreach ($name in $Measurement.environment.psbase.Keys) {
                if ((Get-EnvironmentFingerprint $name) -cne $Measurement.environment[$name]) { return $false }
            }
        }
        return $true
    }
    catch { return $false }
}

function Get-EffectiveCheck($Repository, [string]$Store, [string]$Id, $Measurement,
        [string]$GitFingerprint, [string]$ReferenceFingerprint, [bool]$ReferenceValid) {
    $result = $Measurement.result
    $reason = $Measurement.cause
    if ($Measurement.phase -ceq 'running') { $reason = 'exit-not-observed' }
    if ($Measurement.phase -ceq 'finished' -and ($reason -ceq 'changed-during-run' -or
        ($Measurement.kind -cin @('build', 'test') -and
        ($Measurement.gitFingerprint -cne $GitFingerprint -or
            $Measurement.afterGitFingerprint -cne $GitFingerprint -or -not $ReferenceValid -or
            $Measurement.referenceFingerprint -cne $ReferenceFingerprint -or
            -not (Get-ProcessInputsMatch $Repository $Store $Measurement))))) {
        $result = 'NOT_RUN'
        $reason = 're-verification-required'
    }
    [pscustomobject]@{
        Id = $Id
        Kind = $Measurement.kind
        Status = $result
        RecordedStatus = $Measurement.result
        EvidenceOrigin = 'machine-observation'
        Phase = $Measurement.phase
        ExitCode = $Measurement.exitCode
        Reason = $reason
        Evidence = $Measurement
    }
}

function Get-CurrentReview($Repository, [string]$Store, $Review, [string]$Basis, [bool]$Current) {
    if ($null -eq $Review) { return @{ Status = 'NOT_RUN'; Origin = 'none'; Reason = 'review-required' } }
    if (-not $Review.Contains('origin')) {
        return @{ Status = 'NOT_RUN'; Origin = 'legacy-statement'; Reason = 'traceable-review-required' }
    }
    if (-not $Current -or $Review.basis -cne $Basis) {
        return @{ Status = 'NOT_RUN'; Origin = $Review.origin; Reason = 'review-outdated' }
    }
    try {
        foreach ($path in $Review.evidence.psbase.Keys) {
            if ((Get-DocumentDigest $Repository $path $Store) -cne $Review.evidence[$path]) {
                return @{ Status = 'NOT_RUN'; Origin = $Review.origin; Reason = 'review-artifact-changed' }
            }
        }
    }
    catch { return @{ Status = 'BLOCKED'; Origin = $Review.origin; Reason = 'review-artifact-unavailable' } }
    @{ Status = $Review.status; Origin = $Review.origin; Reason = '' }
}

function Get-ContinuityCombinedStatus([string[]]$Statuses) {
    if ('FAIL' -cin $Statuses) { return 'FAIL' }
    if ('BLOCKED' -cin $Statuses) { return 'BLOCKED' }
    if ($Statuses.Count -eq 0 -or 'NOT_RUN' -cin $Statuses) { return 'NOT_RUN' }
    'PASS'
}

function Get-ContinuityAcceptance($Repository, [string]$Store, $Checkpoint, $Checks, [bool]$Current) {
    $state = $Checkpoint.State
    if ($state.schemaVersion -eq 1) {
        return [pscustomobject]@{
            Id = $null; Criterion = (Get-ContinuityNoteSections $Checkpoint.Task task)['Acceptance Criteria']
            Checks = $state.requiredChecks; Status = 'NOT_RUN'; ReviewStatus = 'NOT_RUN'
            ReviewOrigin = 'none'; Evidence = $null; RecordedReview = $null; Reason = 'criteria-not-declared'
        }
    }
    $observed = @{}
    foreach ($check in $Checks) { $observed.Add($check.Id, $check) }
    $text = @(Get-ContinuityCriterionText $Checkpoint.Task)
    for ($i = 0; $i -lt $state.criteria.Count; $i++) {
        $item = $state.criteria[$i]
        $review = if ($state.reviews.Contains($item.id)) { $state.reviews[$item.id] } else { $null }
        $staleObservation = @($item.checks | Where-Object {
            $observed.ContainsKey($_) -and $observed[$_].Reason -ceq 're-verification-required'
        }).Count -gt 0
        $assessment = Get-CurrentReview $Repository $Store $review (Get-ContinuityReviewBasis $state $item) ($Current -and -not $staleObservation)
        $results = @(foreach ($id in $item.checks) {
            if ($observed.ContainsKey($id) -and $observed[$id].Kind -cin @('build', 'test')) { $observed[$id].Status }
            else { 'NOT_RUN' }
        })
        $results += $assessment.Status
        $status = if ($Current) { Get-ContinuityCombinedStatus $results } else { 'BLOCKED' }
        [pscustomobject]@{
            Id = $item.id; Criterion = $text[$i]; Checks = $item.checks; Status = $status
            ReviewStatus = $assessment.Status
            ReviewOrigin = $assessment.Origin
            Evidence = if ($null -ne $review -and -not $assessment.Reason) { $review.evidence } else { $null }
            RecordedReview = $review
            Reason = if (-not $Current) { 'state-not-current' }
                elseif ($assessment.Reason) { $assessment.Reason }
                elseif ($status -cne 'PASS') { 'review-or-check-incomplete' }
                else { '' }
        }
    }
}

function New-ContinuityReview($Repository, [string]$Store, $Review, [string]$Basis, [string]$RequestPath) {
    if ($Review.status -cnotin @('PASS', 'FAIL', 'NOT_RUN', 'BLOCKED') -or $Review.evidence -isnot [array] -or
        ($Review.status -cin @('PASS', 'FAIL') -and $Review.evidence.Count -eq 0)) {
        throw 'PASS and FAIL reviews require one or more evidence artifact paths.'
    }
    $artifacts = [Collections.Specialized.OrderedDictionary]::new([StringComparer]::Ordinal)
    foreach ($artifact in $Review.evidence) {
        Assert-Record $artifact @('path', 'sha256')
        if ($artifact.path -isnot [string] -or -not $artifact.path.Trim() -or
            $artifact.sha256 -isnot [string] -or $artifact.sha256 -cnotmatch '^[a-f0-9]{64}\z' -or
            $artifacts.Contains($artifact.path)) { throw 'Supply a unique artifact path and its reviewed SHA-256.' }
        Assert-NoCredential $artifact.path
        $artifactPath = Resolve-ContinuityFile $Repository.root $artifact.path
        if ($artifactPath -ceq $RequestPath -or ($IsWindows -and $artifactPath -ieq $RequestPath)) {
            throw 'Supply evidence separate from the review request.'
        }
        $digest = Get-DocumentDigest $Repository $artifact.path $Store
        if ($digest -cne $artifact.sha256) { throw 'Review artifact differs from the approved SHA-256.' }
        $artifacts.Add($artifact.path, $digest)
    }
    [ordered]@{
        status = $Review.status
        origin = 'declared-human-review'
        evidence = $artifacts
        basis = $Basis
        reviewedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    }
}

function Set-ContinuityReviews($Repository, [string]$Store, $State, [string]$Progress, [string]$Path) {
    if ($State.schemaVersion -ne 3) { throw 'Declare the acceptance criteria before recording reviews.' }
    $request = Get-ContinuityInputPath $Path
    $reviews = ConvertFrom-ContinuityJson (Read-ContinuityBytes $request 1048576)
    Assert-Record $reviews @('criteria', 'completedWork')
    if ($reviews.criteria -isnot [array] -or ($reviews.criteria.Count -eq 0 -and $null -eq $reviews.completedWork)) {
        throw 'Supply reviews for criteria, completed work, or both.'
    }
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($review in $reviews.criteria) {
        Assert-Record $review @('id', 'status', 'evidence')
        if ($review.id -isnot [string] -or $review.id -cnotin $State.criteria.id -or -not $seen.Add($review.id)) {
            throw 'Supply a unique declared ID for each criterion being reviewed.'
        }
        $item = $State.criteria | Where-Object { $_.id -ceq $review.id }
        $State.reviews[$review.id] = New-ContinuityReview $Repository $Store $review (Get-ContinuityReviewBasis $State $item) $request
    }
    if ($null -ne $reviews.completedWork) {
        Assert-Record $reviews.completedWork @('status', 'evidence')
        $State.completedReview = New-ContinuityReview $Repository $Store $reviews.completedWork (Get-ContinuityCompletedBasis $State $Progress) $request
    }
}

function Get-ContinuityStatus($Repository, [string]$Store, $Checkpoint, $Git, [hashtable]$RelatedRoot) {
    $state = $Checkpoint.State
    $comparison = if ($null -ne $Git) { Compare-ContinuityGit $state.git $Git }
        else { @{ Matches = $false; Changes = $null; Recorded = $state.git; Current = $null } }
    $referenceProblems = @(Get-ContinuityReferenceProblems $Repository $Store $state.references $RelatedRoot)
    $gitFingerprint = if ($null -ne $Git) { Get-ContinuityGitFingerprint $Git } else { '' }
    $referenceFingerprint = Get-ContinuityValueHash $state.references
    $hasConflicts = $null -ne $Git -and $Git.Contains('unmerged') -and $Git.unmerged.Count -gt 0
    $referencesUsable = -not $hasConflicts -and $referenceProblems.Count -eq 0
    $checks = @(
        foreach ($id in $state.checks.psbase.Keys) {
            Get-EffectiveCheck $Repository $Store $id $state.checks[$id] $gitFingerprint $referenceFingerprint $referencesUsable
        }
    )
    $current = $comparison.Matches -and $referencesUsable
    $acceptance = @(Get-ContinuityAcceptance $Repository $Store $Checkpoint $checks $current)
    $completedReview = if ($state.schemaVersion -eq 3) { $state.completedReview } else { $null }
    $completed = Get-CurrentReview $Repository $Store $completedReview (Get-ContinuityCompletedBasis $state $Checkpoint.Progress) $current
    $requiredStatuses = @($checks | Where-Object { $_.Id -cin $state.requiredChecks } | ForEach-Object { $_.Status })
    if ($requiredStatuses.Count -ne $state.requiredChecks.Count) { $requiredStatuses += 'NOT_RUN' }
    $evidenceStatus = Get-ContinuityCombinedStatus (@($acceptance | ForEach-Object { $_.Status }) + $requiredStatuses + $completed.Status)
    $verificationStatus = if ($current) { $evidenceStatus } else { 'BLOCKED' }
    $acceptanceReady = $verificationStatus -ceq 'PASS'
    $blockers = @(
        if ($null -eq $Git) { 'Current Git state could not be captured. Retry with a stable, accessible worktree.' }
        elseif (-not $comparison.Matches) { 'Git differs from the saved checkpoint. Review the classified changes.' }
        if ($hasConflicts) { 'The Git index contains unmerged paths. Resolve them under repository authorization.' }
        $referenceProblems
        foreach ($check in $checks) {
            if ($check.Status -cin @('NOT_RUN', 'BLOCKED')) {
                "Check '$($check.Id)' requires a new observation."
            }
        }
    )
    [pscustomobject]@{
        TaskId = $state.taskId
        CheckpointId = $state.checkpointId
        AgentFamily = $state.agentFamily
        AgentIdentityVerified = $false
        AgentIdentitySource = 'caller-declaration'
        Lifecycle = $state.lifecycle
        SavedAtUnixMs = $state.savedAtUnixMs
        IntegrityStatus = 'PASS'
        Status = $verificationStatus
        EvidenceStatus = $evidenceStatus
        CompletionStatus = if ($state.lifecycle -ceq 'closed') { $verificationStatus } else { 'NOT_RUN' }
        Ready = $blockers.Count -eq 0
        CorrectiveReady = $current -and -not @($checks | Where-Object Phase -CEQ 'running').Count
        Blockers = $blockers
        Git = $comparison
        GitConflicted = $hasConflicts
        References = $state.references
        ReferenceProblems = $referenceProblems
        RequiredChecks = $state.requiredChecks
        Checks = $checks
        Acceptance = $acceptance
        AcceptanceReady = $acceptanceReady
        CompletedWork = [pscustomobject]@{
            Status = $completed.Status; ReviewOrigin = $completed.Origin; Reason = $completed.Reason
            NotesOrigin = 'agent-or-operator-statement'; RecordedReview = $completedReview
        }
        ProgressReconciliation = if ($state.schemaVersion -eq 3) { $state.progressReconciliation } else { $null }
        CheckpointDirectory = $Checkpoint.Directory
    }
}

function New-ContinuityMeasurement($Repository, [string]$Store, $State, [string]$Kind, [string]$Executable,
        [string[]]$Arguments, [string[]]$InputPath, [string[]]$EnvironmentName) {
    Assert-NoCredential (($Executable, ($Arguments -join ' '), ($InputPath -join ' ')) -join ' ')
    $measurement = [ordered]@{
        kind = $Kind
        phase = 'running'
        executable = $Executable
        toolPath = ''
        toolSha256 = ''
        arguments = $Arguments
        inputs = [ordered]@{}
        environment = [ordered]@{}
        gitFingerprint = $State.gitFingerprint
        afterGitFingerprint = ''
        referenceFingerprint = Get-ContinuityValueHash $State.references
        startedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        finishedAtUnixMs = $null
        exitCode = $null
        result = 'NOT_RUN'
        cause = ''
    }
    try {
        $measurement.toolPath = Find-ContinuityExecutable $Executable
        if (-not $measurement.toolPath) { throw 'Executable is unavailable.' }
        $measurement.toolSha256 = Get-CirnetHash -Path $measurement.toolPath
        $measurement.inputs = Get-ContinuityCheckInputs $Repository $Store $InputPath
        foreach ($name in $EnvironmentName) {
            if ($measurement.environment.Contains($name)) { throw 'Duplicate environment input.' }
            $measurement.environment.Add($name, (Get-EnvironmentFingerprint $name))
        }
    }
    catch {
        $measurement.phase = 'blocked'
        $measurement.result = 'BLOCKED'
        $measurement.cause = 'tool-or-input-unavailable'
        $measurement.finishedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    }
    $measurement
}

function Invoke-ContinuityMeasurement($Repository, $Measurement, [int]$TimeoutSeconds) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Measurement.toolPath
    $info.WorkingDirectory = $Repository.root
    $info.UseShellExecute = $false
    foreach ($arg in $Measurement.arguments) { $info.ArgumentList.Add($arg) }
    try { $process = [Diagnostics.Process]::Start($info) }
    catch {
        $Measurement.phase = 'blocked'
        $Measurement.result = 'BLOCKED'
        $Measurement.cause = 'process-start-failed'
        $Measurement.finishedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        return
    }
    try {
        if ($process.WaitForExit($TimeoutSeconds * 1000)) {
            $Measurement.exitCode = $process.ExitCode
            $Measurement.phase = 'finished'
            $Measurement.result = if ($Measurement.exitCode -eq 0) { 'PASS' } else { 'FAIL' }
        }
        else {
            $Measurement.phase = 'interrupted'
            $Measurement.cause = 'timeout'
        }
    }
    finally {
        try {
            if (-not $process.HasExited) {
                $process.Kill($true)
                if (-not $process.WaitForExit(5000)) { throw 'The controlled process could not be confirmed stopped.' }
            }
        }
        finally { $process.Dispose() }
    }
    $Measurement.finishedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
}

Export-ModuleMember -Function New-ContinuityReferences, Get-ContinuityReferenceProblems, Get-ContinuityStatus,
    New-ContinuityMeasurement, Invoke-ContinuityMeasurement, Get-ProcessInputsMatch, Get-EffectiveCheck,
    Set-ContinuityReviews
