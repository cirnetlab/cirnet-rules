#requires -Version 7.2
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Repository,
    [Parameter(Mandatory)][string]$TaskId,
    [Parameter(Mandatory)][string]$EvidenceId,
    [Parameter(Mandatory)][ValidatePattern('(?-i:^[a-f0-9]{32}\z)')][string]$ExpectedCheckpointId,
    [Parameter(Mandatory)][string]$ResultPath,
    [Parameter(Mandatory)][ValidatePattern('(?-i:^[a-f0-9]{64}\z)')][string]$ResultSha256,
    [ValidateNotNullOrEmpty()][string]$CheckpointRoot,
    [hashtable]$RelatedRoot = @{}
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot 'continuity/Checkpoint.psm1')
Import-Module (Join-Path $PSScriptRoot 'continuity/Verification.psm1')
Import-Module (Join-Path $PSScriptRoot 'shared/Bundle.psm1')

$digestPattern = '^[a-f0-9]{64}\z'
$revisionPattern = '^(?:[a-f0-9]{40}|[a-f0-9]{64})\z'
$claimExits = @{ PASS = 0; FAIL = 1; NOT_RUN = 2; BLOCKED = 3 }
$pathComparison = if ($IsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }

function Read-ReportRecord([string]$Relative, [string]$Digest, [ref]$Budget) {
    Assert-NoCredential $Relative
    $path = Resolve-ContinuityFile $checkout.root $Relative
    if (Test-ContinuityInside (Split-Path -Parent $store) $path) { throw 'Checkpoint data is outside the result scope.' }
    $file = Get-Item -LiteralPath $path -Force
    if ($file -isnot [IO.FileInfo] -or $file.LinkType -eq 'HardLink' -or
        ($file.Attributes -band ([IO.FileAttributes]::ReparsePoint -bor [IO.FileAttributes]::Device))) {
        throw 'Expected a bounded regular result file.'
    }
    if (!$IsWindows) {
        $stat = $file.PSObject.Properties['UnixStat']
        if (!$stat -or !$stat.Value -or [string]$stat.Value.ItemType -cne 'File' -or $stat.Value.HardlinkCount -ne 1) {
            throw 'Expected one regular Unix result file.'
        }
    }
    $bytes = Read-ContinuityBytes $path 1MB $Budget
    if ((Get-CirnetPayloadHash $bytes) -cne $Digest) { throw 'Result pin mismatch.' }
    ConvertFrom-ContinuityJson $bytes -UniqueProperties
}

function Get-VerifierInvocation($Check, $Status) {
    $measurement = $Check.Evidence
    $argsList = $measurement.arguments
    $expectedScript = Join-Path $PSScriptRoot 'verify-claim.ps1'
    if ([IO.Path]::GetFileName($measurement.toolPath) -inotmatch '^pwsh(?:\.exe)?\z' -or
        $argsList.Count -lt 14 -or $argsList[0] -ine '-NoProfile' -or $argsList[1] -ine '-NonInteractive' -or
        $argsList[2] -ine '-File' -or ![IO.Path]::IsPathFullyQualified($argsList[3]) -or
        ![string]::Equals([IO.Path]::GetFullPath($argsList[3]), $expectedScript, $pathComparison)) {
        throw 'Expected a direct verifier invocation.'
    }

    $scriptPinned = $false
    if (Test-ContinuityInside $checkout.root $expectedScript) {
        $relative = [IO.Path]::GetRelativePath($checkout.root, $expectedScript).Replace([IO.Path]::DirectorySeparatorChar, '/')
        $scriptPinned = $measurement.inputs.Contains($relative)
    }
    if (!$scriptPinned) {
        foreach ($reference in $Status.References.documents) {
            $root = if ($reference.repository -ceq 'primary') { $checkout.root }
                else { (Get-ContinuityRepository $RelatedRoot[$reference.repository]).root }
            $path = Resolve-ContinuityFile $root $reference.path
            if ([string]::Equals($path, $expectedScript, $pathComparison)) { $scriptPinned = $true; break }
        }
    }
    if (!$scriptPinned) { throw 'The verifier script is not pinned.' }

    $options = @{}
    $keys = @('-Repository','-ExpectationPath','-ExpectationSha256','-ObservationPath','-ObservationSha256')
    $allowed = $keys + @('-TrustedReviewPath','-TrustedReviewSha256','-CheckpointRoot','-Strict','-ConfirmRelatedRead')
    for ($index = 4; $index -lt $argsList.Count; $index++) {
        $key = $argsList[$index]
        if ($key -notin $allowed -or $options.ContainsKey($key)) { throw 'Unexpected or duplicate verifier parameter.' }
        if ($key -in @('-Strict','-ConfirmRelatedRead')) { $options[$key] = $true; continue }
        $index++
        if ($index -ge $argsList.Count -or !$argsList[$index]) { throw 'Missing verifier parameter value.' }
        $options[$key] = $argsList[$index]
    }
    foreach ($key in $keys) {
        if (!$options.ContainsKey($key)) { throw 'Missing verifier parameter.' }
    }
    if (![IO.Path]::IsPathFullyQualified($options['-Repository']) -or
        ![string]::Equals((Resolve-CirnetRoot -Path $options['-Repository']), $checkout.root, $pathComparison) -or
        $options['-ExpectationSha256'] -cnotmatch $digestPattern -or $options['-ObservationSha256'] -cnotmatch $digestPattern -or
        $measurement.inputs[$options['-ExpectationPath']] -cne $options['-ExpectationSha256'] -or
        $measurement.inputs[$options['-ObservationPath']] -cne $options['-ObservationSha256']) {
        throw 'The verifier invocation does not match its pinned inputs.'
    }
    if ($options.ContainsKey('-TrustedReviewPath') -or $options.ContainsKey('-TrustedReviewSha256')) {
        if (!$options.ContainsKey('-Strict') -or !$options.ContainsKey('-TrustedReviewPath') -or
            $options['-TrustedReviewSha256'] -cnotmatch $digestPattern -or
            $measurement.inputs[$options['-TrustedReviewPath']] -cne $options['-TrustedReviewSha256']) {
            throw 'The verifier review input is not pinned.'
        }
    }
    $options
}

function Assert-ClaimResult($Result, $Check, $Options) {
    Assert-Record $Result @('schemaVersion','status','reason','scope','revision','mode','recordStatus','executionVerified',
        'checkpointSeal','expectationSha256','observationSha256','stages')
    $mode = if ($Options.ContainsKey('-Strict')) { 'strict' } else { 'record-consistency' }
    if (!(Test-Integer $Result.schemaVersion) -or $Result.schemaVersion -ne 1 -or
        $Result.status -isnot [string] -or $Result.status -cnotin $claimExits.Keys -or
        $Result.recordStatus -isnot [string] -or $Result.recordStatus -cnotin $claimExits.Keys -or
        $Result.mode -isnot [string] -or $Result.mode -cne $mode -or
        $Result.scope -isnot [string] -or $Result.scope -cne 'declared-stages-and-inputs' -or
        $Result.executionVerified -isnot [bool] -or
        $Result.executionVerified -ne ($mode -ceq 'strict' -and $Result.status -ceq 'PASS') -or
        $Result.expectationSha256 -isnot [string] -or $Result.expectationSha256 -cne $Options['-ExpectationSha256'] -or
        $Result.observationSha256 -isnot [string] -or $Result.observationSha256 -cne $Options['-ObservationSha256'] -or
        $Result.reason -isnot [string] -or
        $Result.reason -cnotin @('required-stage-assessment','declared-input-changed','revision-changed','evidence-unavailable-or-invalid') -or
        $claimExits[$Result.status] -ne $Check.ExitCode -or
        $Result.stages -isnot [array] -or $Result.stages.Count -gt 64 -or
        ($null -ne $Result.revision -and ($Result.revision -isnot [string] -or $Result.revision -cnotmatch $revisionPattern))) {
        throw 'Invalid verifier result, scope or process exit.'
    }

    if (($mode -ceq 'record-consistency' -and $Result.recordStatus -cne $Result.status) -or
        ($Result.status -ceq 'PASS' -and $Result.recordStatus -cne 'PASS')) { throw 'Record status mismatch.' }
    $planStages = @()
    if ($Result.stages.Count -eq 0) {
        if (!($Result.status -ceq 'BLOCKED' -and $Result.reason -ceq 'evidence-unavailable-or-invalid') -and
            !($Result.status -ceq 'NOT_RUN' -and $Result.reason -cin @('declared-input-changed','revision-changed'))) {
            throw 'Missing verifier stage summaries.'
        }
    }
    else {
        $plan = Read-ReportRecord $Options['-ExpectationPath'] $Options['-ExpectationSha256']
        if ($Result.revision -cne (Get-ContinuityHead $checkout) -or $Result.revision -cne $plan.revision -or
            $Result.reason -cne 'required-stage-assessment' -or $plan.stages -isnot [array] -or
            $plan.stages.Count -ne $Result.stages.Count -or $plan.inputs -isnot [Collections.IDictionary]) {
            throw 'Verifier stage summaries have a stale or different scope.'
        }
        foreach ($inputPath in $plan.inputs.psbase.Keys) {
            if ($Check.Evidence.inputs[$inputPath] -cne $plan.inputs[$inputPath]) {
                throw 'The reporting check did not pin every declared input.'
            }
        }
        $planStages = $plan.stages
    }

    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $stageStatuses = @('PASS','FAIL','NOT_RUN','BLOCKED','SKIPPED')
    $blockedReasons = @('tool-unavailable','input-unavailable','authorization-required','platform-unavailable')
    $stageReasons = @('', 'owner-policy','not-selected','timeout','interrupted','not-executed','missing-stage',
        'assertion-coverage-incomplete','trusted-execution-evidence-required') + $blockedReasons
    $requiredStatuses = @(foreach ($stage in $Result.stages) {
        if ($stage -isnot [Collections.IDictionary] -or $stage.id -isnot [string] -or !$seen.Add($stage.id) -or
            $stage.required -isnot [bool] -or $stage.status -isnot [string] -or $stage.status -cnotin $stageStatuses -or
            $stage.reason -isnot [string] -or $stage.reason -cnotin $stageReasons -or
            ($null -ne $stage.exitCode -and !(Test-Integer $stage.exitCode)) -or
            !(Test-Integer $stage.assertions) -or !(Test-Integer $stage.failedAssertions) -or
            $stage.failedAssertions -lt 0 -or $stage.assertions -lt $stage.failedAssertions -or
            ($stage.status -ceq 'PASS' -and ($stage.exitCode -ne 0 -or $stage.failedAssertions -ne 0 -or
                $stage.reason)) -or
            ($stage.status -ceq 'FAIL' -and ($null -eq $stage.exitCode -or
                ($stage.exitCode -eq 0 -and $stage.failedAssertions -eq 0) -or $stage.reason)) -or
            ($stage.status -ceq 'SKIPPED' -and ($null -ne $stage.exitCode -or $stage.assertions -ne 0 -or !$stage.reason))) {
            throw 'Invalid stage summary.'
        }
        Assert-ContinuityId $stage.id
        $declaration = @($planStages | Where-Object id -CEQ $stage.id)
        if ($declaration.Count -ne 1 -or $declaration[0].required -cne $stage.required -or
            ($stage.status -ceq 'PASS' -and $stage.assertions -lt $declaration[0].minAssertions)) {
            throw 'The reported stage disagrees with the pinned expectation.'
        }
        if ($mode -ceq 'strict' -and $stage.status -cin @('PASS','FAIL') -and $stage.evidenceStatus -cne 'PASS') {
            throw 'A strict completed stage is missing reviewed evidence.'
        }
        if ($stage.required) {
            if ($stage.status -ceq 'FAIL') { 'FAIL' }
            elseif ($stage.status -ceq 'BLOCKED' -or $stage.reason -cin $blockedReasons) { 'BLOCKED' }
            else { $stage.status }
        }
    })
    if ($Result.stages.Count) {
        $aggregate = if ('FAIL' -cin $requiredStatuses) { 'FAIL' }
            elseif ('BLOCKED' -cin $requiredStatuses) { 'BLOCKED' }
            elseif ($requiredStatuses | Where-Object { $_ -cne 'PASS' }) { 'NOT_RUN' }
            else { 'PASS' }
        if (!$requiredStatuses.Count -or $aggregate -cne $Result.status) { throw 'Required stage outcomes disagree.' }
    }
    if ($null -ne $Result.checkpointSeal) {
        if ($Result.checkpointSeal -isnot [string] -or $Result.checkpointSeal -cnotmatch $digestPattern -or
            $mode -cne 'strict' -or !$Options.ContainsKey('-TrustedReviewPath')) { throw 'Unexpected reviewed checkpoint seal.' }
        $budget = 8MB
        $review = Read-ReportRecord $Options['-TrustedReviewPath'] $Options['-TrustedReviewSha256'] ([ref]$budget)
        $sourceRoot = if ($Options.ContainsKey('-CheckpointRoot')) { $Options['-CheckpointRoot'] } else { '.cirnet/tasks' }
        $sourceStore = Get-ContinuityStore $checkout $sourceRoot $review.taskId
        $source = Read-ContinuityCheckpoint $sourceStore $review.taskId -RemainingBytes ([ref]$budget)
        if (!(Test-ContinuityWorktree $checkout $source.State.git) -or $source.Seal -cne $Result.checkpointSeal -or
            $review.checkpointSeal -cne $Result.checkpointSeal) {
            throw 'The reviewed source checkpoint changed.'
        }
        $sourceRoots = Get-ReviewedRelatedRoots $checkout $source.State.references $review `
            ($Options.ContainsKey('-ConfirmRelatedRead')) $RelatedRoot
        $sourceStatus = Get-ContinuityStatus $checkout $sourceStore $source $git $sourceRoots
        if (!$sourceStatus.Git.Matches -or $sourceStatus.GitConflicted -or $sourceStatus.ReferenceProblems.Count) {
            throw 'The reviewed source checkout changed.'
        }
        foreach ($stage in $Result.stages | Where-Object evidenceStatus -CEQ 'PASS') {
            $proof = $review.stages | Where-Object id -CEQ $stage.id
            $sourceCheck = $sourceStatus.Checks | Where-Object Id -CEQ $stage.checkId
            if ($null -eq $proof -or $proof.checkId -cne $stage.checkId -or $proof.artifactPath -cne $stage.artifactPath -or
                $stage.artifactSha256 -isnot [string] -or $stage.artifactSha256 -cnotmatch $digestPattern -or
                $proof.artifactSha256 -cne $stage.artifactSha256 -or $null -eq $sourceCheck -or
                $sourceCheck.Phase -cne 'finished' -or $sourceCheck.Status -cnotin @('PASS','FAIL') -or
                $sourceCheck.ExitCode -ne $stage.measurementExitCode) {
                throw 'The reviewed source observation is stale or disagrees.'
            }
            $null = Read-ReportRecord $stage.artifactPath $stage.artifactSha256 ([ref]$budget)
        }
        $budget = 8MB
        if ((Read-ContinuityCheckpoint $sourceStore $review.taskId -RemainingBytes ([ref]$budget)).Seal -cne $source.Seal) {
            throw 'The reviewed source checkpoint changed during reporting.'
        }
    }
    elseif ($Result.executionVerified) { throw 'Missing verified checkpoint seal.' }
}

$report = [ordered]@{
    schemaVersion = 1; reportStatus = 'BLOCKED'; reason = 'checkpoint-unavailable'
    outputOrigin = 'unverified'
    checkpointId = $null; evidenceId = $null; process = $null; claim = $null; task = $null
}
try {
    Assert-ContinuityId $EvidenceId
    $checkout = Get-ContinuityRepository (Get-ContinuityInputPath $Repository)
    $store = Get-ContinuityStore $checkout $CheckpointRoot $TaskId
    $budget = 64MB
    $checkpoint = Read-ContinuityCheckpoint $store $TaskId -RemainingBytes ([ref]$budget)
    if (!(Test-ContinuityWorktree $checkout $checkpoint.State.git)) { throw 'The checkpoint belongs to another worktree.' }
    $report.checkpointId = $checkpoint.State.checkpointId
    $report.evidenceId = $EvidenceId
    $git = Get-ContinuityGitSnapshot $checkout
    $status = Get-ContinuityStatus $checkout $store $checkpoint $git $RelatedRoot
    $report.task = [ordered]@{
        status = $status.Status; acceptanceReady = $status.AcceptanceReady; completionStatus = $status.CompletionStatus
    }
    $report.reason = 'claim-process-unavailable-or-stale'
    $check = $status.Checks | Where-Object Id -CEQ $EvidenceId
    if ($null -eq $check) { throw 'Missing measured process.' }
    $report.process = [ordered]@{
        recordedStatus = $check.RecordedStatus; effectiveStatus = $check.Status; phase = $check.Phase; exitCode = $check.ExitCode
    }
    if ($report.checkpointId -cne $ExpectedCheckpointId -or !$status.Git.Matches -or $status.ReferenceProblems.Count -or
        $check.Kind -cnotin @('build','test') -or $check.Phase -cne 'finished' -or $check.EvidenceOrigin -cne 'machine-observation' -or
        $check.Status -cnotin @('PASS','FAIL') -or $check.Status -cne $check.RecordedStatus -or $check.ExitCode -notin 0..3) {
        throw 'The process observation is incomplete, stale or unexpected.'
    }

    $report.reason = 'verifier-invocation-unconfirmed'
    $options = Get-VerifierInvocation $check $status
    $report.reason = 'process-output-provenance-unverified'
    if (!$check.Evidence.Contains('stdoutSha256') -or $check.Evidence.stdoutSha256 -cne $ResultSha256) {
        throw 'The supplied digest is not the stdout digest observed by Task Continuity.'
    }
    $report.reason = 'verifier-result-unavailable-or-disagrees'
    $claim = Read-ReportRecord $ResultPath $ResultSha256
    Assert-ClaimResult $claim $check $options
    $budget = 64MB
    if ((Read-ContinuityCheckpoint $store $TaskId -RemainingBytes ([ref]$budget)).Seal -cne $checkpoint.Seal) {
        throw 'The task checkpoint changed during reporting.'
    }
    # Forward the bounded status fields only. Commands, environment and artifact contents stay in owner storage.
    $report.claim = [ordered]@{
        status = $claim.status; recordStatus = $claim.recordStatus; mode = $claim.mode; executionVerified = $claim.executionVerified
        reason = $claim.reason; scope = $claim.scope; revision = $claim.revision
        stages = @(foreach ($stage in $claim.stages) {
            [ordered]@{ id = $stage.id; required = $stage.required; status = $stage.status; reason = $stage.reason }
        })
    }
    $report.reportStatus = 'VALIDATED'
    $report.outputOrigin = 'captured-stdout'
    $report.reason = ''
}
catch { }
ConvertTo-Json -InputObject $report -Depth 8 -Compress
if ($report.reportStatus -ceq 'BLOCKED') { exit 3 }
exit $claimExits[$report.claim.status]
