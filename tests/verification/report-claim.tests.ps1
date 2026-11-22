#requires -Version 7.2
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$reportEntry = Join-Path $repo 'scripts/report-claim.ps1'
if (![IO.File]::Exists($reportEntry)) { throw 'The claim report entry point is missing.' }
$target = (New-Item -ItemType Directory -Force -Path (Join-Path $repo 'target')).FullName
$fixture = Join-Path $target ('claim-report-test-' + [guid]::NewGuid().ToString('N'))
$worktree = Join-Path $fixture 'worktree'
$pwsh = (Get-Command pwsh -CommandType Application | Select-Object -First 1).Source
$common = @{ Repository = $worktree; TaskId = 'claim'; AgentFamily = 'codex' }
$manager = Join-Path $worktree 'scripts/task-continuity.ps1'
$verifier = Join-Path $worktree 'scripts/verify-claim.ps1'
$report = Join-Path $worktree 'scripts/report-claim.ps1'
$resultRelative = '.local/result.json'
$inputRelative = '.local/source.txt'
Import-Module (Join-Path $repo 'scripts/continuity/Repository.psm1')
Import-Module (Join-Path $repo 'scripts/continuity/Checkpoint.psm1')

function Write-Fixture([string]$Root, [string]$Relative, [string]$Content) {
    [IO.File]::WriteAllText((Join-Path $Root $Relative), $Content)
}

function Pin-File([string]$Relative) {
    (Get-FileHash -LiteralPath (Join-Path $worktree $Relative)).Hash.ToLowerInvariant()
}

function Write-Json([string]$Relative, $Data) {
    Write-Fixture $worktree $Relative (ConvertTo-Json -InputObject $Data -Depth 12)
    Pin-File $Relative
}

function Invoke-FixtureGit([string[]]$Arguments) {
    & git -C $worktree -c core.hooksPath=../absent-hooks -c core.autocrlf=false -c core.fsmonitor=false @Arguments
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git operation failed.' }
}

function Invoke-ClaimCheck($Rows, [int]$ExpectedExit, $Review) {
    $planPin = Write-Json '.local/expectation.json' $plan
    $observationPin = Write-Json '.local/observation.json' @{
        schemaVersion = 1; expectationSha256 = $planPin; stages = $Rows
    }
    $nativeArguments = @('-NoProfile', '-NonInteractive', '-File', (Join-Path $fixture 'capture.ps1'),
        $worktree, $pwsh, $planPin, $observationPin)
    if ($Review) {
        $Review.expectationSha256 = $planPin
        $Review.observationSha256 = $observationPin
        $nativeArguments += Write-Json '.local/review.json' $Review
    }
    $capture = [Diagnostics.ProcessStartInfo]::new($pwsh)
    $capture.UseShellExecute = $false
    $capture.RedirectStandardOutput = $true
    foreach ($argument in $nativeArguments) { $capture.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::Start($capture)
    try {
        $raw = $process.StandardOutput.ReadToEnd()
        $process.WaitForExit()
        if ($process.ExitCode -ne $ExpectedExit) { throw "Expected measured exit $ExpectedExit, got $($process.ExitCode)." }
    }
    finally { $process.Dispose() }
    Write-Fixture $worktree $resultRelative $raw
    if (!$raw.Trim()) { throw 'The actual verifier process emitted no structured result.' }
    (& $manager status @common).CheckpointId
}

function Read-ClaimReport([string]$CheckpointId, [int]$ExpectedExit, [string]$ExpectedStatus,
        [string]$Pin = (Pin-File $resultRelative), [string]$Path = $resultRelative) {
    $raw = & $report -Repository $worktree -TaskId claim -EvidenceId claim-check -ExpectedCheckpointId $CheckpointId `
        -ResultPath $Path -ResultSha256 $Pin
    $code = $LASTEXITCODE
    $result = ConvertFrom-Json -InputObject $raw
    if ($code -ne $ExpectedExit -or $result.reportStatus -cne $ExpectedStatus) {
        throw "Expected report $ExpectedStatus/$ExpectedExit, got $($result.reportStatus)/$code."
    }
    if ($result.task.acceptanceReady -or $result.task.completionStatus -cne 'NOT_RUN') {
        throw 'The report treated a check as task acceptance.'
    }
    $result
}

try {
    [void][IO.Directory]::CreateDirectory((Join-Path $worktree '.local'))
    $fixtureScripts = (New-Item -ItemType Directory -Path (Join-Path $worktree 'scripts')).FullName
    $supportingFiles = @('task-continuity.ps1', 'verify-claim.ps1', 'report-claim.ps1', 'continuity', 'shared')
    Copy-Item -LiteralPath @($supportingFiles | ForEach-Object { Join-Path $repo "scripts/$_" }) -Destination $fixtureScripts -Recurse
    Invoke-FixtureGit @('init', '--quiet', '--template=', '-b', 'main')
    Write-Fixture $worktree '.gitignore' "/.cirnet/tasks/`n/.local/`n/scripts/`n"
    Write-Fixture $worktree 'AGENTS.md' "# Fixture instructions`n"
    Write-Fixture $worktree 'architecture.md' "# Fixture architecture`n"
    Invoke-FixtureGit @('add', '--', '.gitignore', 'AGENTS.md', 'architecture.md')
    Invoke-FixtureGit @('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
        '-c', 'commit.gpgsign=false', 'commit', '--quiet', '-m', 'Synthetic reporting fixture')

    Write-Fixture $fixture 'task.md' @'
## Original Task
Exercise claim reporting.
## Objectives
Preserve both observed process and claim outcomes.
## Acceptance Criteria
- The required verification succeeds.
## Architecture & Rules
Use the fixture rule.
## Scope & Authorization
The disposable fixture checkout only.
## Plans & Specs
See the fixture rule.
'@
    Write-Fixture $fixture 'progress.md' @'
## Completed Work
Prepared synthetic input.
## Pending Work
Measure verification.
## Decisions & Blockers
Use the synthetic worktree.
## Next Action
Run the verifier.
## Re-verification
Check the process observation and verifier output together.
'@
    Write-Fixture $fixture 'refs.json' @'
{"documents":[{"kind":"rules","repository":"primary","path":"AGENTS.md"},
              {"kind":"architecture","repository":"primary","path":"architecture.md"}],"relatedRepositories":[]}
'@
    Write-Fixture $fixture 'acceptance.json' '[{"id":"verification","checks":["claim-check"]}]'
    Write-Fixture $fixture 'capture.ps1' @'
param($Root, $Tool, $PlanPin, $ObservationPin, $ReviewPin)
$ErrorActionPreference = 'Stop'
$verifier = Join-Path $Root 'scripts/verify-claim.ps1'
$arguments = @('-NoProfile', '-NonInteractive', '-File', $verifier, '-Repository', $Root,
    '-ExpectationPath', '.local/expectation.json', '-ExpectationSha256', $PlanPin,
    '-ObservationPath', '.local/observation.json', '-ObservationSha256', $ObservationPin)
$inputs = @('scripts/verify-claim.ps1', '.local/expectation.json', '.local/observation.json', '.local/source.txt')
if ($ReviewPin) {
    $arguments += @('-Strict', '-TrustedReviewPath', '.local/review.json', '-TrustedReviewSha256', $ReviewPin)
    $inputs += '.local/review.json'
}
& (Join-Path $Root 'scripts/task-continuity.ps1') run -Repository $Root -TaskId claim -AgentFamily codex -ConfirmScope `
    -EvidenceId claim-check -Kind test -Executable $Tool -Arguments $arguments `
    -InputPath $inputs | Out-Null
exit $LASTEXITCODE
'@
    Write-Fixture $worktree $inputRelative 'fixture source'
    $null = & $manager save @common -ConfirmScope -TaskFile (Join-Path $fixture 'task.md') `
        -ProgressFile (Join-Path $fixture 'progress.md') -ReferencesFile (Join-Path $fixture 'refs.json') `
        -AcceptanceFile (Join-Path $fixture 'acceptance.json')
    $plan = @{
        schemaVersion = 1; revision = (Invoke-FixtureGit @('rev-parse', 'HEAD'))
        inputs = @{ $inputRelative = (Pin-File $inputRelative) }
        stages = @(@{ id = 'required'; required = $true; minAssertions = 1 },
            @{ id = 'optional'; required = $false; minAssertions = 1 })
    }
    $pass = @{ id = 'required'; status = 'PASS'; exitCode = 0; assertions = 1; failedAssertions = 0; reason = '' }
    $failed = @{ id = 'required'; status = 'FAIL'; exitCode = 7; assertions = 1; failedAssertions = 1; reason = '' }
    $requiredSkip = @{ id = 'required'; status = 'SKIPPED'; exitCode = $null; assertions = 0; failedAssertions = 0; reason = 'owner-policy' }
    $unavailable = $requiredSkip.Clone()
    $unavailable.reason = 'platform-unavailable'
    $optionalSkip = $requiredSkip.Clone()
    $optionalSkip.id = 'optional'
    foreach ($case in @(
        @{ rows = @($pass, $optionalSkip); status = 'PASS'; exit = 0; stage = 'PASS' },
        @{ rows = @($failed, $optionalSkip); status = 'FAIL'; exit = 1; stage = 'FAIL' },
        @{ rows = @($requiredSkip, $optionalSkip); status = 'NOT_RUN'; exit = 2; stage = 'SKIPPED' },
        @{ rows = @($unavailable, $optionalSkip); status = 'BLOCKED'; exit = 3; stage = 'SKIPPED' }
    )) {
        $id = Invoke-ClaimCheck $case.rows $case.exit
        $result = Read-ClaimReport $id $case.exit VALIDATED
        $recorded = if ($case.exit -eq 0) { 'PASS' } else { 'FAIL' }
        if ($result.process.recordedStatus -cne $recorded -or $result.process.exitCode -ne $case.exit -or
            $result.claim.status -cne $case.status -or $result.claim.stages[0].status -cne $case.stage -or
            $result.claim.stages[1].status -cne 'SKIPPED' -or $result.claim.executionVerified) {
            throw 'The report did not preserve the measured exit and scoped claim outcome.'
        }
    }

    $id = Invoke-ClaimCheck @($pass, $optionalSkip) 0
    $actual = Get-Content -LiteralPath (Join-Path $worktree $resultRelative) -Raw
    $wrong = ConvertFrom-Json -InputObject $actual -AsHashtable
    $wrong.status = 'FAIL'
    $wrong.stages[0].status = 'FAIL'
    $wrong.stages[0].exitCode = 7
    $null = Write-Json $resultRelative $wrong
    $result = Read-ClaimReport $id 3 BLOCKED
    if ($result.process.exitCode -ne 0 -or $null -ne $result.claim) {
        throw 'A wrapper zero and failed claim were silently accepted as a consistent pair.'
    }

    $wrong = ConvertFrom-Json -InputObject $actual -AsHashtable
    $wrong.stages[0].status = 'SKIPPED'
    $null = Write-Json $resultRelative $wrong
    $null = Read-ClaimReport $id 3 BLOCKED
    Write-Fixture $worktree $resultRelative '{'
    $null = Read-ClaimReport $id 3 BLOCKED
    Write-Fixture $worktree $resultRelative $actual
    $null = Read-ClaimReport $id 3 BLOCKED ('0' * 64)
    $null = Read-ClaimReport $id 3 BLOCKED (Pin-File $resultRelative) '../outside.json'

    Write-Fixture $worktree $inputRelative 'changed fixture source'
    $result = Read-ClaimReport $id 3 BLOCKED
    if ($result.process.recordedStatus -cne 'PASS' -or $result.process.effectiveStatus -cne 'NOT_RUN') {
        throw 'Stale inputs should invalidate claim pairing and preserve the raw recorded observation.'
    }
    $id = Invoke-ClaimCheck @($pass, $optionalSkip) 2
    $result = Read-ClaimReport $id 2 VALIDATED
    if ($result.claim.status -cne 'NOT_RUN' -or @($result.claim.stages).Count -ne 0) {
        throw 'A fresh verifier run on a stale claim should preserve NOT_RUN.'
    }
    Write-Fixture $worktree $inputRelative 'fixture source'
    Write-Fixture $worktree '.local/owner-extra.txt' 'owner prerequisite'
    Write-Fixture $worktree '.local/owner-test.ps1' @'
$ErrorActionPreference = 'Stop'
if ((Get-Content -LiteralPath '.local/source.txt' -Raw) -cne 'fixture source' -or
    (Get-Content -LiteralPath '.local/owner-extra.txt' -Raw) -cne 'owner prerequisite') { throw 'Fixture assertion failed.' }
[IO.File]::WriteAllText((Join-Path $PWD '.local/owner-result.json'),
    '{"schemaVersion":1,"stageId":"required","exitCode":0,"assertions":1,"failedAssertions":0}')
'@
    $owner = $common.Clone()
    $owner.TaskId = 'owner-source'
    Write-Fixture $fixture 'owner-acceptance.json' '[{"id":"verification","checks":["owner-check"]}]'
    $null = & $manager save @owner -ConfirmScope -TaskFile (Join-Path $fixture 'task.md') `
        -ProgressFile (Join-Path $fixture 'progress.md') -ReferencesFile (Join-Path $fixture 'refs.json') `
        -AcceptanceFile (Join-Path $fixture 'owner-acceptance.json')
    $observed = & $manager run @owner -ConfirmScope -EvidenceId owner-check -Kind test -Executable $pwsh `
        -Arguments @('-NoProfile', '-File', '.local/owner-test.ps1') `
        -InputPath @($inputRelative, '.local/owner-test.ps1', '.local/owner-extra.txt')
    if ($LASTEXITCODE -or $observed.ExitCode -ne 0) { throw 'The owner fixture did not record a successful execution.' }
    $ownerStore = Get-ContinuityStore (Get-ContinuityRepository $worktree) '.cirnet/tasks' $owner.TaskId
    $ownerCheckpoint = Read-ContinuityCheckpoint $ownerStore $owner.TaskId
    $review = @{
        schemaVersion = 1; taskId = $owner.TaskId; checkpointSeal = $ownerCheckpoint.Seal
        stages = @(@{ id = 'required'; checkId = 'owner-check'; artifactPath = '.local/owner-result.json';
            artifactSha256 = (Pin-File '.local/owner-result.json') })
    }
    $id = Invoke-ClaimCheck @($pass, $optionalSkip) 0 $review
    $result = Read-ClaimReport $id 0 VALIDATED
    if ($result.claim.mode -cne 'strict' -or !$result.claim.executionVerified) {
        throw 'Strict PASS did not compose with a separate source checkpoint.'
    }
    $artifact = Get-Content -LiteralPath (Join-Path $worktree '.local/owner-result.json') -Raw
    Write-Fixture $worktree '.local/owner-result.json' ($artifact + ' ')
    $result = Read-ClaimReport $id 3 BLOCKED
    if ($result.process.effectiveStatus -cne 'PASS') { throw 'An artifact change rewrote the measured verifier result.' }
    Write-Fixture $worktree '.local/owner-result.json' $artifact
    Write-Fixture $worktree '.local/owner-extra.txt' 'changed prerequisite'
    $null = Read-ClaimReport $id 3 BLOCKED
    Write-Fixture $worktree '.local/owner-extra.txt' 'owner prerequisite'
    $null = & $manager save @owner
    $null = Read-ClaimReport $id 3 BLOCKED

    $check = & $manager run @common -ConfirmScope -EvidenceId claim-check -Kind test `
        -Executable $pwsh -Arguments @('-NoProfile', '-NonInteractive', '-Command', 'exit 4')
    $result = Read-ClaimReport $check.CheckpointId 3 BLOCKED
    if ($result.process.exitCode -ne 4 -or $result.process.recordedStatus -cne 'FAIL') {
        throw 'Unexpected exit must be preserved and must not validate as a claim.'
    }
    $check = & $manager run @common -ConfirmScope -EvidenceId claim-check -Kind test -Executable 'fixture-command-unavailable'
    $result = Read-ClaimReport $check.CheckpointId 3 BLOCKED
    if ($null -ne $result.process.exitCode -or $result.process.recordedStatus -cne 'BLOCKED') {
        throw 'A tool resolution failure must remain a blocked process without an observed exit.'
    }
    Write-Host 'PASS: measured claim exits, skipped stages, strict source freshness, disagreement and tool errors.'
}
finally {
    if ([IO.Directory]::Exists($fixture)) {
        $resolved = (Resolve-Path -LiteralPath $fixture).Path
        if ([IO.Path]::GetDirectoryName($resolved) -cne $target -or
            [IO.Path]::GetFileName($resolved) -cnotmatch '^claim-report-test-[0-9a-f]{32}$') { throw 'Unsafe report fixture cleanup path.' }
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
}
exit 0
