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
    $roots
}

function Get-DocumentDigest($Checkout, [string]$Relative, [string]$Store) {
    $path = Resolve-ContinuityFile $Checkout.root $Relative
    if (Test-ContinuityInside $Store $path) { throw 'A reference or check input cannot use the checkpoint store as evidence.' }
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
        $inputs.Add($path, (Get-DocumentDigest $Repository $path $Store))
    }
    $inputs
}

function Get-ProcessInputsMatch($Repository, [string]$Store, $Measurement) {
    try {
        $tool = Find-ContinuityExecutable $Measurement.executable
        if (-not $tool -or $tool -cne $Measurement.toolPath -or (Get-CirnetHash -Path $tool) -cne $Measurement.toolSha256) { return $false }
        foreach ($path in $Measurement.inputs.psbase.Keys) {
            if ((Get-DocumentDigest $Repository $path $Store) -cne $Measurement.inputs[$path]) { return $false }
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
        Phase = $Measurement.phase
        ExitCode = $Measurement.exitCode
        Reason = $reason
        Evidence = $Measurement
    }
}

function Get-ContinuityAcceptance($Checkpoint, $Checks, [bool]$Current) {
    $state = $Checkpoint.State
    if ($state.schemaVersion -eq 1) {
        return [pscustomobject]@{
            Id = $null; Criterion = (Get-ContinuityNoteSections $Checkpoint.Task task)['Acceptance Criteria']
            Checks = $state.requiredChecks; Status = 'NOT_RUN'; ReviewStatus = 'NOT_RUN'
            Evidence = $null; Reason = 'criteria-not-declared'
        }
    }
    $observed = @{}
    foreach ($check in $Checks) { $observed.Add($check.Id, $check) }
    $text = @(Get-ContinuityCriterionText $Checkpoint.Task)
    for ($i = 0; $i -lt $state.criteria.Count; $i++) {
        $item = $state.criteria[$i]
        $review = $state.reviews[$item.id]
        $staleObservation = @($item.checks | Where-Object {
            $observed.ContainsKey($_) -and $observed[$_].Reason -ceq 're-verification-required'
        }).Count -gt 0
        $freshReview = $Current -and -not $staleObservation -and $null -ne $review -and
            $review.basis -ceq (Get-ContinuityReviewBasis $state $item)
        $reviewStatus = if ($freshReview) { $review.status } else { 'NOT_RUN' }
        $results = @(foreach ($id in $item.checks) {
            if ($observed.ContainsKey($id) -and $observed[$id].Kind -cin @('build', 'test')) { $observed[$id].Status }
            else { 'NOT_RUN' }
        })
        $results += $reviewStatus
        $status = if (-not $Current -or 'BLOCKED' -cin $results) { 'BLOCKED' }
            elseif ('FAIL' -cin $results) { 'FAIL' }
            elseif ('NOT_RUN' -cin $results) { 'NOT_RUN' }
            else { 'PASS' }
        [pscustomobject]@{
            Id = $item.id; Criterion = $text[$i]; Checks = $item.checks; Status = $status
            ReviewStatus = $reviewStatus
            Evidence = if ($freshReview) { $review.evidence } else { $null }
            Reason = if (-not $Current) { 'state-not-current' }
                elseif (-not $freshReview) { 'review-required' }
                elseif ($status -cne 'PASS') { 'review-or-check-incomplete' }
                else { '' }
        }
    }
}

function Set-ContinuityReviews($State, [string]$Path) {
    if ($State.schemaVersion -ne 2) { throw 'Declare the acceptance criteria before recording reviews.' }
    $reviews = ConvertFrom-ContinuityJson (Read-ContinuityBytes (Get-ContinuityInputPath $Path) 1048576)
    if ($reviews -isnot [array] -or $reviews.Count -eq 0) { throw 'Supply a nonempty array of acceptance reviews.' }
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($review in $reviews) {
        Assert-Record $review @('id', 'status', 'evidence')
        if ($review.id -isnot [string] -or $review.id -cnotin $State.criteria.id -or -not $seen.Add($review.id) -or
            $review.status -cnotin @('PASS', 'FAIL', 'NOT_RUN', 'BLOCKED') -or
            $review.evidence -isnot [string] -or -not $review.evidence.Trim()) {
            throw 'Supply one valid status and evidence reference for each criterion being reviewed.'
        }
        Assert-NoCredential $review.evidence
        $item = $State.criteria | Where-Object { $_.id -ceq $review.id }
        $State.reviews[$review.id] = [ordered]@{
            status = $review.status
            evidence = $review.evidence
            basis = Get-ContinuityReviewBasis $State $item
            reviewedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        }
    }
}

function Get-ContinuityStatus($Repository, [string]$Store, $Checkpoint, $Git, [hashtable]$RelatedRoot) {
    $state = $Checkpoint.State
    $comparison = if ($null -ne $Git) { Compare-ContinuityGit $state.git $Git }
        else { @{ Matches = $false; Changes = $null; Recorded = $state.git; Current = $null } }
    $referenceProblems = @(Get-ContinuityReferenceProblems $Repository $Store $state.references $RelatedRoot)
    $gitFingerprint = if ($null -ne $Git) { Get-ContinuityGitFingerprint $Git } else { '' }
    $referenceFingerprint = Get-ContinuityValueHash $state.references
    $checks = @(
        foreach ($id in $state.checks.psbase.Keys) {
            Get-EffectiveCheck $Repository $Store $id $state.checks[$id] $gitFingerprint $referenceFingerprint ($referenceProblems.Count -eq 0)
        }
    )
    $acceptance = @(Get-ContinuityAcceptance $Checkpoint $checks ($comparison.Matches -and $referenceProblems.Count -eq 0))
    $acceptanceReady = @($acceptance | Where-Object Status -CNE 'PASS').Count -eq 0
    $blockers = @(
        if ($null -eq $Git) { 'Current Git state could not be captured. Retry with a stable, accessible worktree.' }
        elseif (-not $comparison.Matches) { 'Git differs from the saved checkpoint. Review the classified changes.' }
        $referenceProblems
        foreach ($check in $checks) {
            if ($check.Status -cin @('NOT_RUN', 'BLOCKED')) {
                "Check '$($check.Id)' requires a new observation."
            }
        }
        if ($state.lifecycle -ceq 'closed' -and -not $acceptanceReady) {
            'The recorded closure lacks current evidence for every criterion.'
        }
    )
    [pscustomobject]@{
        TaskId = $state.taskId
        CheckpointId = $state.checkpointId
        AgentFamily = $state.agentFamily
        Lifecycle = $state.lifecycle
        SavedAtUnixMs = $state.savedAtUnixMs
        Status = if ($blockers.Count -eq 0) { 'PASS' } else { 'BLOCKED' }
        Ready = $blockers.Count -eq 0
        Blockers = $blockers
        Git = $comparison
        References = $state.references
        ReferenceProblems = $referenceProblems
        RequiredChecks = $state.requiredChecks
        Checks = $checks
        Acceptance = $acceptance
        AcceptanceReady = $acceptanceReady
        CheckpointDirectory = $Checkpoint.Directory
    }
}

function New-ContinuityMeasurement($Repository, [string]$Store, $State, [string]$Kind, [string]$Executable,
        [string[]]$Arguments, [string[]]$InputPath) {
    Assert-NoCredential (($Executable, ($Arguments -join ' '), ($InputPath -join ' ')) -join ' ')
    $measurement = [ordered]@{
        kind = $Kind
        phase = 'running'
        executable = $Executable
        toolPath = ''
        toolSha256 = ''
        arguments = $Arguments
        inputs = [ordered]@{}
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
