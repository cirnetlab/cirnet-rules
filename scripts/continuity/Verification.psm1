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
    if ($Measurement.phase -ceq 'finished' -and $Measurement.kind -cin @('build', 'test') -and
        ($reason -ceq 'changed-during-run' -or $Measurement.gitFingerprint -cne $GitFingerprint -or
            $Measurement.afterGitFingerprint -cne $GitFingerprint -or -not $ReferenceValid -or
            $Measurement.referenceFingerprint -cne $ReferenceFingerprint -or
            -not (Get-ProcessInputsMatch $Repository $Store $Measurement))) {
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

function Get-ContinuityStatus($Repository, [string]$Store, $Checkpoint, $Git, [hashtable]$RelatedRoot) {
    $state = $Checkpoint.State
    $comparison = Compare-ContinuityGit $state.git $Git
    $referenceProblems = @(Get-ContinuityReferenceProblems $Repository $Store $state.references $RelatedRoot)
    $gitFingerprint = Get-ContinuityGitFingerprint $Git
    $referenceFingerprint = Get-ContinuityValueHash $state.references
    $checks = @(
        foreach ($id in $state.checks.psbase.Keys) {
            Get-EffectiveCheck $Repository $Store $id $state.checks[$id] $gitFingerprint $referenceFingerprint ($referenceProblems.Count -eq 0)
        }
    )
    $blockers = @(
        if (-not $comparison.Matches) { 'Git differs from the saved checkpoint. Review the classified changes.' }
        $referenceProblems
        foreach ($check in $checks) {
            if ($check.Kind -cin @('build', 'test') -and $check.Status -cin @('NOT_RUN', 'BLOCKED')) {
                "Check '$($check.Id)' requires a new observation."
            }
        }
        if ($state.lifecycle -ceq 'closed') {
            foreach ($id in $state.requiredChecks) {
                if (-not ($checks | Where-Object { $_.Id -ceq $id -and $_.Status -ceq 'PASS' -and $_.Kind -cin @('build', 'test') })) {
                    "Closure evidence for '$id' requires re-verification."
                }
            }
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
        RequiredChecks = $state.requiredChecks
        Checks = $checks
        CheckpointDirectory = $Checkpoint.Directory
    }
}

function New-ContinuityMeasurement($Repository, [string]$Store, $State, [string]$Kind, [string]$Executable,
        [string[]]$Arguments, [string[]]$InputPath) {
    Assert-NoCredential (($Executable, ($Arguments -join ' ')) -join ' ')
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

function Assert-PortableHandoff($Data, [string]$AgentFamily) {
    $fields = @('schemaVersion', 'format', 'taskKey', 'checkpointId', 'checkpointSeal', 'agentFamily',
        'lifecycle', 'savedAtUnixMs', 'gitHead', 'checks')
    if ($Data -isnot [Collections.IDictionary] -or $Data.psbase.Count -ne $fields.Count -or
        @($fields | Where-Object { $_ -cnotin $Data.psbase.Keys }).Count -or
        $Data.schemaVersion -ne 1 -or $Data.format -cne 'cirnet-task-handoff' -or
        $Data.taskKey -cnotmatch '^[a-f0-9]{32}\z' -or $Data.checkpointId -cnotmatch '^[a-f0-9]{32}\z' -or
        $Data.checkpointSeal -cnotmatch '^[a-f0-9]{64}\z' -or $Data.agentFamily -cnotin @('codex', 'claude-code') -or
        $Data.lifecycle -cnotin @('open', 'closed') -or ($Data.savedAtUnixMs -isnot [long] -and $Data.savedAtUnixMs -isnot [int]) -or
        $Data.savedAtUnixMs -le 0 -or $Data.gitHead -cnotmatch '^(?:[a-f0-9]{40}|[a-f0-9]{64})?\z' -or
        $Data.checks -isnot [array]) {
        throw 'Invalid portable handoff schema.'
    }
    foreach ($check in $Data.checks) {
        if ($check -isnot [Collections.IDictionary] -or $check.psbase.Count -ne 2 -or
            'kind' -cnotin $check.psbase.Keys -or 'status' -cnotin $check.psbase.Keys -or
            $check.kind -cnotin @('edit', 'build', 'test', 'handoff') -or $check.status -cnotin @('PASS', 'FAIL', 'NOT_RUN', 'BLOCKED')) {
            throw 'Invalid portable handoff check.'
        }
    }
    if ($Data.agentFamily -cne $AgentFamily) { throw 'Agent family does not match the portable handoff.' }
}

function Export-ContinuityHandoff($Repository, [string]$Store, $Checkpoint, $Report, [string]$Path) {
    $output = Get-ContinuityInputPath $Path
    Assert-ContinuityDestination $Repository $output
    if (Test-ContinuityInside $Store $output) { throw 'Choose a new export file outside the checkpoint store.' }
    $state = $Checkpoint.State
    $projection = [ordered]@{
        schemaVersion = 1
        format = 'cirnet-task-handoff'
        taskKey = $state.taskKey
        checkpointId = $state.checkpointId
        checkpointSeal = $Checkpoint.Seal
        agentFamily = $state.agentFamily
        lifecycle = $state.lifecycle
        savedAtUnixMs = $state.savedAtUnixMs
        gitHead = $state.git.head
        checks = @(foreach ($check in $Report.Checks) { [ordered]@{ kind = $check.Kind; status = $check.Status } })
    }
    $bytes = ConvertTo-ContinuityBytes $projection
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
    if ($ExpectedSha256 -cnotmatch '^[a-f0-9]{64}\z') { throw 'Supply the trusted SHA-256 from export using -ExpectedSha256.' }
    $bytes = Read-ContinuityBytes (Get-ContinuityInputPath $Path) 1048576
    if ((Get-CirnetPayloadHash -Bytes $bytes) -cne $ExpectedSha256) { throw 'Portable handoff hash mismatch.' }
    $data = ConvertFrom-ContinuityJson $bytes
    Assert-PortableHandoff $data $AgentFamily
    [pscustomobject]@{ Status = 'PASS'; Sha256 = $ExpectedSha256; Handoff = $data }
}

Export-ModuleMember -Function New-ContinuityReferences, Get-ContinuityReferenceProblems, Get-ContinuityStatus,
    New-ContinuityMeasurement, Invoke-ContinuityMeasurement, Get-ProcessInputsMatch, Get-EffectiveCheck,
    Export-ContinuityHandoff, Read-ContinuityHandoff
