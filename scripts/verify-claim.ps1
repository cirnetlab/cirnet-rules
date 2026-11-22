#requires -Version 7.2
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Repository,
    [Parameter(Mandatory)][string]$ExpectationPath,
    [Parameter(Mandatory)][ValidatePattern('(?-i:^[a-f0-9]{64}\z)')][string]$ExpectationSha256,
    [Parameter(Mandatory)][string]$ObservationPath,
    [Parameter(Mandatory)][ValidatePattern('(?-i:^[a-f0-9]{64}\z)')][string]$ObservationSha256,
    [switch]$Strict,
    [string]$TrustedReviewPath,
    [string]$TrustedReviewSha256,
    [string]$CheckpointRoot = '.cirnet/tasks'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot 'continuity/Checkpoint.psm1')
Import-Module (Join-Path $PSScriptRoot 'continuity/Verification.psm1')
Import-Module (Join-Path $PSScriptRoot 'shared/Bundle.psm1')

$digestPattern = '^[a-f0-9]{64}\z'
$reasonCodes = @('tool-unavailable','input-unavailable','authorization-required','owner-policy',
    'not-selected','platform-unavailable','timeout','interrupted','not-executed')
$blockedReasons = @('tool-unavailable','input-unavailable','authorization-required','platform-unavailable')

function Read-PinnedBytes([string]$Root, [string]$Path, [int]$MaxBytes, [ref]$Budget) {
    if ($Path -match '(?:^|/)\.git(?:/|$)') { throw 'Git administrative path.' }
    $resolved = Resolve-CirnetPath -Root $Root -Relative $Path
    $file = Get-Item -LiteralPath $resolved -Force
    if ($file -isnot [IO.FileInfo] -or !$file.Exists -or $file.LinkType -eq 'HardLink' -or
        ($file.Attributes -band ([IO.FileAttributes]::ReparsePoint -bor [IO.FileAttributes]::Device))) {
        throw 'Expected a bounded regular file.'
    }
    if (!$IsWindows) {
        $stat = $file.PSObject.Properties['UnixStat']
        if (!$stat -or !$stat.Value -or [string]$stat.Value.ItemType -cne 'File' -or $stat.Value.HardlinkCount -ne 1) {
            throw 'Expected one regular Unix file.'
        }
    }
    return ,(Read-ContinuityBytes $file.FullName $MaxBytes $Budget)
}

function Read-EvidenceRecord([string]$Root, [string]$Path, [string]$Digest, [ref]$Budget) {
    $bytes = Read-PinnedBytes $Root $Path 1MB $Budget
    if ((Get-CirnetPayloadHash $bytes) -cne $Digest) { throw 'Evidence hash mismatch.' }
    ConvertFrom-ContinuityJson $bytes -UniqueProperties
}

function Assert-Expectation($Plan) {
    Assert-Record $Plan @('schemaVersion','revision','inputs','stages')
    if (!(Test-Integer $Plan.schemaVersion) -or $Plan.schemaVersion -ne 1 -or $Plan.revision -isnot [string] -or
        $Plan.revision -cnotmatch '^(?:[a-f0-9]{40}|[a-f0-9]{64})\z' -or
        $Plan.inputs -isnot [Collections.IDictionary] -or $Plan.inputs.psbase.Count -notin (1..32) -or
        $Plan.stages -isnot [array] -or $Plan.stages.Count -notin (1..64)) { throw 'Invalid expectation.' }
    foreach ($pin in $Plan.inputs.psbase.Values) {
        if ($pin -isnot [string] -or $pin -cnotmatch $digestPattern) { throw 'Invalid input pin.' }
    }
    $ids = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($stage in $Plan.stages) {
        Assert-Record $stage @('id','required','minAssertions')
        if ($stage.id -isnot [string] -or !$ids.Add($stage.id) -or $stage.required -isnot [bool] -or
            !(Test-Integer $stage.minAssertions) -or $stage.minAssertions -lt 1) { throw 'Invalid stage declaration.' }
        Assert-ContinuityId $stage.id
    }
    if (!($Plan.stages | Where-Object required)) { throw 'A claim needs a required stage.' }
}

function Read-StageObservations($Record, $Plan) {
    Assert-Record $Record @('schemaVersion','expectationSha256','stages')
    if (!(Test-Integer $Record.schemaVersion) -or $Record.schemaVersion -ne 1 -or
        $Record.expectationSha256 -isnot [string] -or $Record.expectationSha256 -cne $ExpectationSha256 -or
        $Record.stages -isnot [array] -or $Record.stages.Count -gt $Plan.stages.Count) { throw 'Invalid observation.' }
    $rows = @{}
    foreach ($row in $Record.stages) {
        Assert-Record $row @('id','status','exitCode','assertions','failedAssertions','reason')
        if ($row.id -isnot [string] -or $row.id -cnotin $Plan.stages.id -or $rows.ContainsKey($row.id) -or
            $row.status -isnot [string] -or $row.status -cnotin @('PASS','FAIL','BLOCKED','NOT_RUN','SKIPPED') -or
            $row.reason -isnot [string] -or !(Test-Integer $row.assertions) -or !(Test-Integer $row.failedAssertions) -or
            $row.failedAssertions -lt 0 -or $row.assertions -lt $row.failedAssertions) { throw 'Invalid stage observation.' }
        if ($row.status -cin @('PASS','FAIL')) {
            if (!(Test-Integer $row.exitCode) -or $row.exitCode -lt [int]::MinValue -or
                $row.exitCode -gt [int]::MaxValue -or $row.reason) { throw 'Invalid completed stage.' }
            if ($row.status -ceq 'PASS' -and ($row.exitCode -ne 0 -or $row.failedAssertions -ne 0 -or $row.assertions -eq 0)) {
                throw 'Invalid passing stage.'
            }
            if ($row.status -ceq 'FAIL' -and $row.exitCode -eq 0 -and $row.failedAssertions -eq 0) { throw 'Missing failure observation.' }
        }
        elseif ($null -ne $row.exitCode -or $row.assertions -ne 0 -or $row.reason -cnotin $reasonCodes) {
            throw 'Invalid incomplete stage.'
        }
        $rows[$row.id] = $row
    }
    $rows
}

function Set-ClaimStatus($Result) {
    $required = @($Result.stages | Where-Object required)
    $Result.status = if ('FAIL' -cin $required.status) { 'FAIL' }
        elseif ($required | Where-Object { $_.status -ceq 'BLOCKED' -or $_.reason -cin $blockedReasons }) { 'BLOCKED' }
        elseif ($required | Where-Object { $_.status -cne 'PASS' }) { 'NOT_RUN' }
        else { 'PASS' }
    $Result.reason = 'required-stage-assessment'
}

function Read-ReviewStages($Review, $Plan) {
    Assert-Record $Review @('schemaVersion','expectationSha256','observationSha256','taskId','checkpointSeal','stages')
    if (!(Test-Integer $Review.schemaVersion) -or $Review.schemaVersion -ne 1 -or
        $Review.expectationSha256 -isnot [string] -or $Review.observationSha256 -isnot [string] -or
        $Review.expectationSha256 -cne $ExpectationSha256 -or $Review.observationSha256 -cne $ObservationSha256 -or
        $Review.taskId -isnot [string] -or $Review.checkpointSeal -isnot [string] -or
        $Review.checkpointSeal -cnotmatch $digestPattern -or $Review.stages -isnot [array] -or
        $Review.stages.Count -gt $Plan.stages.Count) { throw 'Invalid execution review.' }
    Assert-ContinuityId $Review.taskId
    $rows = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::Ordinal)
    $checks = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $artifacts = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($row in $Review.stages) {
        Assert-Record $row @('id','checkId','artifactPath','artifactSha256')
        if ($row.id -isnot [string] -or $row.id -cnotin $Plan.stages.id -or $row.checkId -isnot [string] -or
            $row.artifactPath -isnot [string] -or $row.artifactPath -in @($ExpectationPath,$ObservationPath,$TrustedReviewPath) -or
            $row.artifactSha256 -isnot [string] -or $row.artifactSha256 -cnotmatch $digestPattern -or
            !$checks.Add($row.checkId) -or !$artifacts.Add($row.artifactPath)) { throw 'Invalid reviewed stage.' }
        Assert-ContinuityId $row.checkId
        Assert-NoCredential $row.artifactPath
        $rows.Add($row.id, $row)
    }
    return ,$rows
}

function Test-StageEvidence($Root, [string]$StorageRoot, $Stage, $Reviewed, $Check, $Plan, [ref]$Budget) {
    if ($null -eq $Check -or $Check.Phase -cne 'finished' -or $Check.Kind -cnotin @('build','test') -or
        $Check.Status -cnotin @('PASS','FAIL') -or $Check.EvidenceOrigin -cne 'machine-observation') { return $false }
    foreach ($path in $Plan.inputs.psbase.Keys) {
        if ($path -cnotin $Check.Evidence.inputs.psbase.Keys -or
            $Check.Evidence.inputs[$path] -cne $Plan.inputs[$path]) { return $false }
    }
    $resolved = Resolve-CirnetPath -Root $Root.root -Relative $Reviewed.artifactPath
    if (Test-ContinuityInside $StorageRoot $resolved) { return $false }
    $artifact = Read-EvidenceRecord $Root.root $Reviewed.artifactPath $Reviewed.artifactSha256 $Budget
    Assert-Record $artifact @('schemaVersion','stageId','exitCode','assertions','failedAssertions')
    if (!(Test-Integer $artifact.schemaVersion) -or $artifact.schemaVersion -ne 1 -or
        $artifact.stageId -isnot [string] -or $artifact.stageId -cne $Stage.id -or
        !(Test-Integer $artifact.exitCode) -or $artifact.exitCode -ne $Stage.exitCode -or
        !(Test-Integer $artifact.assertions) -or $artifact.assertions -ne $Stage.assertions -or
        !(Test-Integer $artifact.failedAssertions) -or $artifact.failedAssertions -ne $Stage.failedAssertions) { return $false }
    # A reviewer may bind an observed child failure when a wrapper discarded that exit.
    return $Check.ExitCode -eq $artifact.exitCode -or ($Check.ExitCode -eq 0 -and $Stage.recordStatus -ceq 'FAIL')
}

function Measure-StrictEvidence($Root, $Plan, $Result) {
    $completed = @($Result.stages | Where-Object { $_.status -cin @('PASS','FAIL') })
    foreach ($stage in $Result.stages) {
        $stage.recordStatus = $stage.status
        $stage.evidenceStatus = 'NOT_RUN'
    }
    foreach ($stage in $completed) {
        $stage.status = 'BLOCKED'
        $stage.reason = 'trusted-execution-evidence-required'
    }
    Set-ClaimStatus $Result
    if (!$TrustedReviewPath -and !$TrustedReviewSha256) { return }
    if (!$TrustedReviewPath -or $TrustedReviewSha256 -cnotmatch $digestPattern) { throw 'Missing trusted review pin.' }
    $budget = 8MB
    $review = Read-EvidenceRecord $Root.root $TrustedReviewPath $TrustedReviewSha256 ([ref]$budget)
    $reviewedStages = Read-ReviewStages $review $Plan
    $store = Get-ContinuityStore $Root $CheckpointRoot $review.taskId
    $storageRoot = [IO.Path]::GetDirectoryName($store)
    $checkpoint = Read-ContinuityCheckpoint $store $review.taskId -RemainingBytes ([ref]$budget)
    if ($checkpoint.Seal -cne $review.checkpointSeal) { throw 'Reviewed checkpoint is stale.' }
    $git = Get-ContinuityGitSnapshot $Root
    $status = Get-ContinuityStatus $Root $store $checkpoint $git @{}
    if (!$status.Git.Matches -or $status.GitConflicted -or $status.ReferenceProblems.Count -or
        (Get-ContinuityHead $Root) -cne $Plan.revision) { throw 'Reviewed checkout is stale.' }
    $matches = @{}
    foreach ($stage in $completed) {
        if (!$reviewedStages.ContainsKey($stage.id)) { continue }
        $reviewed = $reviewedStages[$stage.id]
        $check = $status.Checks | Where-Object Id -CEQ $reviewed.checkId
        try {
            if (Test-StageEvidence $Root $storageRoot $stage $reviewed $check $Plan ([ref]$budget)) { $matches[$stage.id] = $check }
        }
        catch { continue }
    }
    $current = Read-ContinuityCheckpoint $store $review.taskId -RemainingBytes ([ref]$budget)
    if ($current.Seal -cne $checkpoint.Seal) { throw 'Reviewed checkpoint changed during assessment.' }
    $Result.checkpointSeal = $checkpoint.Seal
    foreach ($stage in $completed) {
        if (!$matches.ContainsKey($stage.id)) { continue }
        $reviewed = $reviewedStages[$stage.id]
        $stage.status = $stage.recordStatus
        $stage.reason = ''
        $stage.evidenceStatus = 'PASS'
        $stage.checkId = $reviewed.checkId
        $stage.artifactPath = $reviewed.artifactPath
        $stage.artifactSha256 = $reviewed.artifactSha256
        $stage.measurementExitCode = $matches[$stage.id].ExitCode
    }
    Set-ClaimStatus $Result
    $Result.executionVerified = $Result.status -ceq 'PASS'
}

function Measure-Claim($Result) {
    $root = Get-ContinuityRepository $Repository
    $plan = Read-EvidenceRecord $root.root $ExpectationPath $ExpectationSha256
    Assert-Expectation $plan
    $record = Read-EvidenceRecord $root.root $ObservationPath $ObservationSha256
    $rows = Read-StageObservations $record $plan
    $Result.revision = $plan.revision
    $budget = 32MB
    foreach ($path in $plan.inputs.psbase.Keys) {
        $bytes = Read-PinnedBytes $root.root $path 16MB ([ref]$budget)
        if ((Get-CirnetPayloadHash $bytes) -cne $plan.inputs[$path]) {
            $Result.status = 'NOT_RUN'
            $Result.reason = 'declared-input-changed'
            return
        }
    }
    $head = Get-ContinuityHead $root
    if (!$head) { throw 'Current revision is unavailable.' }
    if ($head -cne $plan.revision) {
        $Result.status = 'NOT_RUN'
        $Result.reason = 'revision-changed'
        return
    }
    $Result.stages = @(foreach ($stage in $plan.stages) {
        $row = $rows[$stage.id]
        $status = if ($row) { $row.status } else { 'NOT_RUN' }
        $reason = if ($row) { $row.reason } else { 'missing-stage' }
        if ($status -ceq 'PASS' -and $row.assertions -lt $stage.minAssertions) {
            $status = 'NOT_RUN'
            $reason = 'assertion-coverage-incomplete'
        }
        [ordered]@{
            id = $stage.id; required = $stage.required; status = $status; reason = $reason
            exitCode = if ($row) { $row.exitCode } else { $null }
            assertions = if ($row) { $row.assertions } else { 0 }
            failedAssertions = if ($row) { $row.failedAssertions } else { 0 }
        }
    })
    Set-ClaimStatus $Result
    $Result.recordStatus = $Result.status
    if ($Strict) { Measure-StrictEvidence $root $plan $Result }
    elseif ($TrustedReviewPath -or $TrustedReviewSha256) { throw 'Use -Strict with a trusted review.' }
}

$result = [ordered]@{
    schemaVersion = 1; status = 'BLOCKED'; reason = 'evidence-unavailable-or-invalid'
    scope = 'declared-stages-and-inputs'; revision = $null
    mode = if ($Strict) { 'strict' } else { 'record-consistency' }
    recordStatus = $null; executionVerified = $false; checkpointSeal = $null
    expectationSha256 = $ExpectationSha256; observationSha256 = $ObservationSha256; stages = @()
}
try { Measure-Claim $result }
catch { $result.status = 'BLOCKED'; $result.reason = 'evidence-unavailable-or-invalid'; $result.stages = @() }
if ($null -eq $result.recordStatus) { $result.recordStatus = $result.status }
ConvertTo-Json -InputObject $result -Depth 8 -Compress
exit (@{ PASS = 0; FAIL = 1; NOT_RUN = 2; BLOCKED = 3 }[$result.status])
