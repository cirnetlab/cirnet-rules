#requires -Version 7.2
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$manager = Join-Path $repo 'scripts/task-continuity.ps1'
$verify = Join-Path $repo 'scripts/verify-claim.ps1'
Import-Module (Join-Path $repo 'scripts/continuity/Repository.psm1')
Import-Module (Join-Path $repo 'scripts/continuity/Checkpoint.psm1')
$target = Join-Path $repo 'target'
$root = Join-Path $target ('strict-claim-test-' + [guid]::NewGuid().ToString('N'))
$taskId = 'strict-claim'
$scope = @{ Repository = $root; TaskId = $taskId; AgentFamily = 'codex'; ConfirmScope = $true }

function Write-Fixture($Path, $Content) { [IO.File]::WriteAllText((Join-Path $root $Path), $Content) }
function Get-FixtureHash($Path) { (Get-FileHash -LiteralPath (Join-Path $root $Path)).Hash.ToLowerInvariant() }
function Write-Record($Path, $Record) {
    Write-Fixture $Path (ConvertTo-Json -InputObject $Record -Depth 8)
    Get-FixtureHash $Path
}
function Invoke-FixtureGit($Arguments) {
    & git -C $root -c core.autocrlf=false -c "core.hooksPath=$(Join-Path $root '.local/no-hooks')" `
        -c user.name=Fixture -c user.email=fixture@example.invalid -c commit.gpgsign=false @Arguments
    if ($LASTEXITCODE) { throw 'Fixture Git failed.' }
}
function Assert-Strict($Row, $Proofs, $Expected, $Exit, $StageStatus, [switch]$BadReviewPin) {
    $observationHash = Write-Record .local/observation.json @{
        schemaVersion = 1; expectationSha256 = $planHash; stages = @($Row)
    }
    $reviewHash = Write-Record .local/review.json @{
        schemaVersion = 1; expectationSha256 = $planHash; observationSha256 = $observationHash
        taskId = $taskId; checkpointSeal = $checkpoint.Seal; stages = @($Proofs)
    }
    if ($BadReviewPin) { $reviewHash = '0' * 64 }
    $json = & $verify -Repository $root -ExpectationPath .local/expectation.json -ExpectationSha256 $planHash `
        -ObservationPath .local/observation.json -ObservationSha256 $observationHash -Strict `
        -TrustedReviewPath .local/review.json -TrustedReviewSha256 $reviewHash
    $actual = $LASTEXITCODE
    $result = $json | ConvertFrom-Json
    if ($actual -ne $Exit -or $result.status -cne $Expected -or $result.mode -cne 'strict' -or
        $result.executionVerified -ne ($Expected -ceq 'PASS') -or
        ($StageStatus -and ($result.stages.Count -ne 1 -or $result.stages[0].status -cne $StageStatus))) {
        throw "Expected $Expected/$Exit ($StageStatus), got $actual $json"
    }
    $result
}

try {
    [void][IO.Directory]::CreateDirectory((Join-Path $root '.local'))
    Invoke-FixtureGit @('init','--quiet','--template=','-b','main')
    Write-Fixture '.gitignore' "/.local/`n/.cirnet/tasks/`n"
    Write-Fixture 'AGENTS.md' '# Fixture instructions'
    Write-Fixture 'architecture.md' '# Fixture architecture'
    Write-Fixture 'stage.mjs' @'
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import test from 'node:test';

const [mode, scenario] = process.argv.slice(2);
if (mode === 'child') {
  test('checks the approved input', () => assert.equal(scenario, 'pass'));
} else {
  const child = spawnSync(process.execPath, ['--test-reporter=tap', 'stage.mjs', 'child', scenario], { encoding: 'utf8' });
  const assertions = Number(child.stdout.match(/^# tests (\d+)$/m)?.[1]);
  const failedAssertions = Number(child.stdout.match(/^# fail (\d+)$/m)?.[1]);
  assert.ok(Number.isInteger(assertions) && Number.isInteger(failedAssertions));
  assert.ok(Number.isInteger(child.status));
  writeFileSync('.local/' + scenario + '.json', JSON.stringify({
    schemaVersion: 1, stageId: 'tests', exitCode: child.status, assertions, failedAssertions,
  }));
  if (mode !== 'wrapper') process.exitCode = child.status;
}
'@
    Invoke-FixtureGit @('add','--all')
    Invoke-FixtureGit @('commit','--quiet','-m','fixture')
    $head = Invoke-FixtureGit @('rev-parse','HEAD')
    Write-Fixture '.local/task.md' @'
## Original Task
Verify the fixture stage.
## Objectives
Bind measured child evidence.
## Acceptance Criteria
- The approved stage passes.
## Architecture & Rules
Use the pinned rules.
## Scope & Authorization
Verify only this disposable checkout.
## Plans & Specs
Use the declared expectation.
'@
    Write-Fixture '.local/progress.md' @'
## Completed Work
Prepared the fixture.
## Pending Work
Review the stage.
## Decisions & Blockers
Only the fixture is in scope.
## Next Action
Measure the stage.
## Re-verification
Inspect structured child output.
'@
    Write-Fixture '.local/refs.json' @'
{"documents":[
  {"kind":"rules","repository":"primary","path":"AGENTS.md"},
  {"kind":"architecture","repository":"primary","path":"architecture.md"}
],"relatedRepositories":[]}
'@
    Write-Fixture '.local/acceptance.json' '[{"id":"fixture","checks":["stage-pass"]}]'
    $null = & $manager save @scope -TaskFile (Join-Path $root '.local/task.md') -ProgressFile (Join-Path $root '.local/progress.md') `
        -ReferencesFile (Join-Path $root '.local/refs.json') -AcceptanceFile (Join-Path $root '.local/acceptance.json')
    $planHash = Write-Record .local/expectation.json @{
        schemaVersion = 1; revision = $head; inputs = @{ 'stage.mjs' = Get-FixtureHash stage.mjs }
        stages = @(@{ id = 'tests'; required = $true; minAssertions = 1 })
    }
    $measurement = & $manager run @scope -EvidenceId stage-pass -Kind test -Executable node `
        -Arguments @('stage.mjs','direct','pass') -InputPath 'stage.mjs'
    if ($LASTEXITCODE -ne 0 -or $measurement.ExitCode -ne 0 -or $measurement.RecordedStatus -cne 'PASS') {
        throw 'Fixture success was not observed.'
    }
    $store = Get-ContinuityStore (Get-ContinuityRepository $root) '.cirnet/tasks' $taskId
    $checkpoint = Read-ContinuityCheckpoint $store $taskId
    $pass = @{ id = 'tests'; status = 'PASS'; exitCode = 0; assertions = 1; failedAssertions = 0; reason = '' }
    $passProof = @{
        id = 'tests'; checkId = 'stage-pass'; artifactPath = '.local/pass.json'
        artifactSha256 = Get-FixtureHash .local/pass.json
    }
    $good = Assert-Strict $pass @($passProof) PASS 0 PASS
    if ($good.stages[0].evidenceStatus -cne 'PASS' -or $good.stages[0].measurementExitCode -ne 0 -or
        $good.checkpointSeal -cne $checkpoint.Seal) { throw 'Strict PASS was missing reviewed evidence.' }
    $null = Assert-Strict $pass @() BLOCKED 3 BLOCKED
    $null = Assert-Strict $pass @($passProof) BLOCKED 3 '' -BadReviewPin
    $outsideProof = $passProof.Clone()
    $outsideProof.artifactPath = '../unapproved.json'
    $null = Assert-Strict $pass @($outsideProof) BLOCKED 3 BLOCKED
    $artifact = Join-Path $root '.local/pass.json'
    $artifactBytes = [IO.File]::ReadAllBytes($artifact)
    [IO.File]::AppendAllText($artifact, ' ')
    $null = Assert-Strict $pass @($passProof) BLOCKED 3 BLOCKED
    [IO.File]::WriteAllBytes($artifact, $artifactBytes)

    $measurement = & $manager run @scope -EvidenceId wrapper -Kind test -Executable node `
        -Arguments @('stage.mjs','wrapper','fail') -InputPath 'stage.mjs'
    if ($LASTEXITCODE -ne 0 -or $measurement.ExitCode -ne 0) { throw 'Fixture wrapper did not reproduce exit zero.' }
    $null = Assert-Strict $pass @($passProof) BLOCKED 3 ''
    $checkpoint = Read-ContinuityCheckpoint $store $taskId
    $fail = @{ id = 'tests'; status = 'FAIL'; exitCode = 1; assertions = 1; failedAssertions = 1; reason = '' }
    $failProof = @{
        id = 'tests'; checkId = 'wrapper'; artifactPath = '.local/fail.json'
        artifactSha256 = Get-FixtureHash .local/fail.json
    }
    $bad = Assert-Strict $fail @($failProof) FAIL 1 FAIL
    if ($bad.stages[0].measurementExitCode -ne 0 -or $bad.stages[0].exitCode -ne 1) { throw 'Lost child exit or wrapper exit.' }
    $null = Assert-Strict $pass @($failProof) BLOCKED 3 BLOCKED
    $skipped = @{ id = 'tests'; status = 'SKIPPED'; exitCode = $null; assertions = 0; failedAssertions = 0; reason = 'owner-policy' }
    $null = Assert-Strict $skipped @() NOT_RUN 2 SKIPPED
    $skipped.reason = 'tool-unavailable'
    $null = Assert-Strict $skipped @() BLOCKED 3 SKIPPED
    $skipped.status = 'NOT_RUN'
    $skipped.reason = 'timeout'
    $null = Assert-Strict $skipped @() NOT_RUN 2 NOT_RUN
    $missing = $pass.Clone()
    $missing.exitCode = $null
    $null = Assert-Strict $missing @($passProof) BLOCKED 3 ''
    [IO.File]::AppendAllText((Join-Path $root 'stage.mjs'), '// changed input')
    $null = Assert-Strict $pass @($passProof) NOT_RUN 2 ''
    Write-Host 'PASS: strict evidence verified observed execution, preserved child failure and skips, and rejected stale and mismatched proof.'
}
finally {
    $resolved = (Resolve-Path -LiteralPath $root).Path
    if ([IO.Path]::GetDirectoryName($resolved) -cne (Resolve-Path -LiteralPath $target).Path -or
        [IO.Path]::GetFileName($resolved) -cnotmatch '^strict-claim-test-[a-f0-9]{32}$') { throw 'Unsafe strict fixture cleanup path.' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
exit 0
