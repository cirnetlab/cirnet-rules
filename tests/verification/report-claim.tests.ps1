#requires -Version 7.2
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$reportEntry = Join-Path $repo 'scripts/report-claim.ps1'
if (![IO.File]::Exists($reportEntry)) { throw 'The claim report entry point is missing.' }
$target = [IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetFullPath([IO.Path]::GetTempPath()))
$fixture = Join-Path $target ('claim-report-test-' + [guid]::NewGuid().ToString('N'))
$worktree = Join-Path $fixture 'worktree'
$pwsh = (Get-Command pwsh -CommandType Application | Select-Object -First 1).Source
$common = @{ Repository = $worktree; TaskId = 'claim'; AgentFamily = 'codex' }
$manager = Join-Path $worktree 'scripts/task-continuity.ps1'
$verifier = Join-Path $worktree 'scripts/verify-claim.ps1'
$report = Join-Path $worktree 'scripts/report-claim.ps1'
$resultRelative = '.local/result.json'
$inputRelative = '.local/source.txt'
$substitutionFailures = [Collections.Generic.List[string]]::new()
$reportRelatedRoot = @{}
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

function Invoke-FixtureGit([string[]]$Arguments, [string]$Checkout = $worktree) {
    & git -C $Checkout -c core.hooksPath=../absent-hooks -c core.autocrlf=false -c core.fsmonitor=false @Arguments
    if ($LASTEXITCODE -ne 0) { throw 'Fixture Git operation failed.' }
}

function Invoke-ClaimCheck($Rows, [int]$ExpectedExit, $Review, [switch]$NoOutput, [switch]$Unbound,
        [string]$VerifierRepository = $worktree, [string]$VerifierPath = $verifier) {
    $planPin = Write-Json '.local/expectation.json' $plan
    $observationPin = Write-Json '.local/observation.json' @{
        schemaVersion = 1; expectationSha256 = $planPin; stages = $Rows
    }
    $nativeArguments = @('-NoProfile', '-NonInteractive', '-File', $VerifierPath, '-Repository', $VerifierRepository,
        '-ExpectationPath', '.local/expectation.json', '-ExpectationSha256', $planPin,
        '-ObservationPath', '.local/observation.json', '-ObservationSha256', $observationPin)
    $inputs = @('scripts/verify-claim.ps1', '.local/expectation.json', '.local/observation.json', $inputRelative)
    if ($Review) {
        $Review.expectationSha256 = $planPin
        $Review.observationSha256 = $observationPin
        $nativeArguments += @('-Strict', '-TrustedReviewPath', '.local/review.json', '-TrustedReviewSha256',
            (Write-Json '.local/review.json' $Review))
        $inputs += '.local/review.json'
        if ($Review.ContainsKey('relatedRoots')) { $nativeArguments += '-ConfirmRelatedRead' }
    }
    $script:resultRelative = '.local/result-' + [guid]::NewGuid().ToString('N') + '.json'
    $capture = if ($Unbound) { @{} } else { @{ ClaimOutputPath = $resultRelative } }
    $measured = & $manager run @common -ConfirmScope -EvidenceId claim-check -Kind test -Executable $pwsh `
        -Arguments $nativeArguments -InputPath $inputs @capture
    if ($LASTEXITCODE -ne $ExpectedExit -or $measured.ExitCode -ne $ExpectedExit) { throw 'Unexpected measured process exit.' }
    if ($NoOutput -or $Unbound) {
        if ([IO.File]::Exists((Join-Path $worktree $resultRelative)) -or $measured.Evidence.Contains('stdoutSha256')) {
            throw 'An incomplete or uncaptured output acquired a digest.'
        }
    }
    elseif (![IO.File]::Exists((Join-Path $worktree $resultRelative)) -or
        $measured.Evidence.stdoutSha256 -cne (Pin-File $resultRelative)) {
        throw "Expected directly captured verifier result and measured exit $ExpectedExit."
    }
    $measured.CheckpointId
}

function Read-ClaimReport([string]$CheckpointId, [int]$ExpectedExit, [string]$ExpectedStatus,
        [string]$Pin = (Pin-File $resultRelative), [string]$Path = $resultRelative) {
    $raw = & $report -Repository $worktree -TaskId claim -EvidenceId claim-check -ExpectedCheckpointId $CheckpointId `
        -ResultPath $Path -ResultSha256 $Pin -RelatedRoot $reportRelatedRoot
    $code = $LASTEXITCODE
    $result = ConvertFrom-Json -InputObject $raw
    if ($code -ne $ExpectedExit -or $result.reportStatus -cne $ExpectedStatus -or
        ($ExpectedStatus -ceq 'VALIDATED') -ne ($result.outputOrigin -ceq 'captured-stdout')) {
        throw "Expected report $ExpectedStatus/$ExpectedExit, got $($result.reportStatus)/$code."
    }
    if ($result.task.acceptanceReady -or $result.task.completionStatus -cne 'NOT_RUN') {
        throw 'The report treated a check as task acceptance.'
    }
    $result
}

function Test-Substitution([string]$CheckpointId, [string]$Mode) {
    $actualBytes = [IO.File]::ReadAllBytes((Join-Path $worktree $resultRelative))
    $path = '.local/substituted.json'
    Write-Fixture $worktree $path ([Text.Encoding]::UTF8.GetString($actualBytes) + ' ')
    $raw = & $report -Repository $worktree -TaskId claim -EvidenceId claim-check -ExpectedCheckpointId $CheckpointId `
        -ResultPath $path -ResultSha256 (Pin-File $path) -RelatedRoot $reportRelatedRoot
    $code = $LASTEXITCODE
    $result = $raw | ConvertFrom-Json
    if ($code -ne 3 -or $result.reportStatus -cne 'BLOCKED') {
        $substitutionFailures.Add("$Mode accepted non-origin bytes with status $($result.reportStatus)/$code")
    }
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

    $repositoryWithSeparator = $worktree + [IO.Path]::DirectorySeparatorChar
    $id = Invoke-ClaimCheck @($pass, $optionalSkip) 0 -VerifierRepository $repositoryWithSeparator
    $null = Read-ClaimReport $id 0 VALIDATED
    if ($IsWindows) {
        $verifierWithSlashes = $verifier.Replace([IO.Path]::DirectorySeparatorChar, '/')
        $id = Invoke-ClaimCheck @($pass, $optionalSkip) 0 -VerifierPath $verifierWithSlashes
        $null = Read-ClaimReport $id 0 VALIDATED
    }

    $id = Invoke-ClaimCheck @($pass, $optionalSkip) 0
    Test-Substitution $id 'record-consistency'
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
    $related = Join-Path $fixture 'related'
    [void][IO.Directory]::CreateDirectory($related)
    Invoke-FixtureGit @('init','--quiet','--template=','-b','main') $related
    Write-Fixture $related 'contract.md' '# Disposable owner contract'
    Invoke-FixtureGit @('add','--','contract.md') $related
    Invoke-FixtureGit @('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','commit.gpgsign=false',
        'commit','--quiet','-m','Related fixture') $related
    $refs = Get-Content -LiteralPath (Join-Path $fixture 'refs.json') -Raw | ConvertFrom-Json -AsHashtable
    $refs.documents += @{ kind = 'architecture'; repository = 'related'; path = 'contract.md' }
    $refs.relatedRepositories = @(@{ id = 'related'; revision = (Invoke-FixtureGit @('rev-parse','HEAD') $related); access = 'read' })
    Write-Fixture $fixture 'owner-refs.json' (ConvertTo-Json $refs -Depth 8)
    $owner.RelatedRoot = @{ related = $related }
    Write-Fixture $fixture 'owner-acceptance.json' '[{"id":"verification","checks":["owner-check"]}]'
    $null = & $manager save @owner -ConfirmScope -TaskFile (Join-Path $fixture 'task.md') `
        -ProgressFile (Join-Path $fixture 'progress.md') -ReferencesFile (Join-Path $fixture 'owner-refs.json') `
        -AcceptanceFile (Join-Path $fixture 'owner-acceptance.json')
    $observed = & $manager run @owner -ConfirmScope -EvidenceId owner-check -Kind test -Executable $pwsh `
        -Arguments @('-NoProfile', '-File', '.local/owner-test.ps1') `
        -InputPath @($inputRelative, '.local/owner-test.ps1', '.local/owner-extra.txt')
    if ($LASTEXITCODE -or $observed.ExitCode -ne 0) { throw 'The owner fixture did not record a successful execution.' }
    $ownerStore = Get-ContinuityStore (Get-ContinuityRepository $worktree) '.cirnet/tasks' $owner.TaskId
    $ownerCheckpoint = Read-ContinuityCheckpoint $ownerStore $owner.TaskId
    $review = @{
        schemaVersion = 1; taskId = $owner.TaskId; checkpointSeal = $ownerCheckpoint.Seal
        relatedRoots = $owner.RelatedRoot
        stages = @(@{ id = 'required'; checkId = 'owner-check'; artifactPath = '.local/owner-result.json';
            artifactSha256 = (Pin-File '.local/owner-result.json') })
    }
    $id = Invoke-ClaimCheck @($pass, $optionalSkip) 0 $review
    $null = Read-ClaimReport $id 3 BLOCKED
    $reportRelatedRoot = $owner.RelatedRoot
    $result = Read-ClaimReport $id 0 VALIDATED
    if ($result.claim.mode -cne 'strict' -or !$result.claim.executionVerified) {
        throw 'Strict PASS did not compose with a separate source checkpoint.'
    }
    Test-Substitution $id 'strict'
    if ($substitutionFailures.Count) { throw ($substitutionFailures -join [Environment]::NewLine) }
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

    $savedResult = $resultRelative
    $savedPin = Pin-File $savedResult
    $originalVerifier = [IO.File]::ReadAllBytes($verifier)
    foreach ($output in @('exit 0', "Write-Output ('x' * (1MB + 1)); exit 0")) {
        [IO.File]::WriteAllText($verifier, $output)
        $id = Invoke-ClaimCheck @($pass, $optionalSkip) 0 -NoOutput
        $null = Read-ClaimReport $id 3 BLOCKED $savedPin $savedResult
    }
    [IO.File]::WriteAllBytes($verifier, $originalVerifier)
    $id = Invoke-ClaimCheck @($pass, $optionalSkip) 0 -Unbound
    $unbound = Read-ClaimReport $id 3 BLOCKED $savedPin $savedResult
    if ($unbound.reason -cne 'process-output-provenance-unverified') { throw 'Expected an unbound historical result.' }
    $resultRelative = $savedResult

    $check = & $manager run @common -ConfirmScope -EvidenceId claim-check -Kind test `
        -Executable $pwsh -Arguments @('-NoProfile', '-NonInteractive', '-Command', 'exit 0')
    $result = Read-ClaimReport $check.CheckpointId 3 BLOCKED
    if ($result.process.exitCode -ne 0 -or $result.reason -cne 'verifier-invocation-unconfirmed') {
        throw 'A different successful command cannot establish the verifier result.'
    }

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
