#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$runner = Join-Path $repo 'scripts/benchmark/run.ps1'
$role = Join-Path $repo 'scripts/benchmark/role.ps1'
$reporter = Join-Path $repo 'scripts/benchmark/report.mjs'
$output = (New-Item -ItemType Directory -Path (Join-Path ([IO.Path]::GetTempPath()) "cirnet-resume-$([guid]::NewGuid().ToString('N'))")).FullName
$shim = Join-Path $output 'client.ps1'
@'
if ($args -contains '--version') { 'codex local resume shim'; exit 0 }
if ($args -contains '--help') {
    '--ask-for-approval never --sandbox read-only workspace-write --json --ephemeral --output-last-message --cd --config --disable --model --ignore-user-config --ignore-rules'
    exit 0
}
if ($args -contains 'login') { exit 0 }
$target = $args[[array]::IndexOf($args, '--output-last-message') + 1]
$prompt = [Console]::In.ReadToEnd()
if ($prompt -match 'Start the handoff with') {
    if ($PWD.Path -match 'original-work') { throw 'Resume invoked Author' }
    $answer = @"
PROMPT_READY
Original task: Fix the loading state that never clears when saving fails.
Write scope: ["save-panel.ts","tests/","README.md"]
## Requirements and constraints
Fix failed-save loading and preserve the component contract and graph preview.
## Acceptance and verification
Run the TypeScript model tests for rejection and original success.
Report: deliverables, changed paths, actual verification, failures, and unresolved limitations.
"@
} else {
    if ($PWD.Path -notmatch 'original-work') { throw 'Unexpected Work call' }
    $file = Join-Path $PWD 'save-panel.ts'
    $source = [IO.File]::ReadAllText($file).Replace('    state.loading = false;', '')
    $source = $source -replace '(?m)^  }(\r?\n)}', '  } finally {$1    state.loading = false;$1  }$1}'
    [IO.File]::WriteAllText($file, $source)
    Add-Content -LiteralPath 'tests/save-panel.test.mjs' -Value @"

test('rejected save emits failed and clears loading', async () => {
  const state = { loading: false };
  const events = [];
  await savePanel(state, 'g-01', async () => { throw new Error('offline'); }, (...e) => events.push(e));
  assert.deepEqual(events, [['failed', 'offline']]);
  assert.equal(state.loading, false);
});
"@
    $test = node --test 'tests/*.test.mjs' | Out-String
    if ($LASTEXITCODE -ne 0) { throw 'Fixture model tests failed' }
    $answer = 'The two model checks passed in this offline fixture.'
}
'{"type":"thread.started","thread_id":"resume-shim"}'
'{"type":"turn.started"}'
if ($test) {
    @{ type='item.completed'; item=@{ type='command_execution'; id='shim-test'; command='node --test tests/*.test.mjs'; exit_code=0; aggregated_output=$test } } | ConvertTo-Json -Depth 5 -Compress
}
[IO.File]::WriteAllText($target, $answer)
@{ type='item.completed'; item=@{ type='agent_message'; id='answer'; text=$answer } } | ConvertTo-Json -Depth 5 -Compress
'{"type":"turn.completed"}'
'@ | Set-Content -LiteralPath $shim -Encoding utf8NoBOM

function Read-Json([string]$Path) { Get-Content -Raw -LiteralPath $Path | ConvertFrom-Json -AsHashtable }
function Save-Json([string]$Path, $Value) { ConvertTo-Json -Depth 25 -InputObject $Value | Set-Content -LiteralPath $Path -Encoding utf8NoBOM }
function Digest([string]$Path) { (Get-FileHash -LiteralPath $Path).Hash.ToLowerInvariant() }

& $runner -Session original -OutputRoot $output -CaseId E03
if ($LASTEXITCODE -ne 0) { throw 'Could not freeze E03' }
$original = Join-Path $output 'original'
$manifestPath = Join-Path $original 'manifest.json'
$manifest = Read-Json $manifestPath
$manifest.mode = 'TEST_SHIM'
Save-Json $manifestPath $manifest
$trial = (New-Item -ItemType Directory -Path (Join-Path $original 'trials/E03')).FullName
& $role -Session Author -Role Author -Execute -CodexPath $shim -CaseId E03 -OutputRoot $trial `
    -RulesRoot (Join-Path $original 'rules') -SuiteRoot (Join-Path $original 'evaluator') -BenchmarkRoot $original `
    -Model $manifest.modelRequested -Effort $manifest.effortRequested -SourceRevision $manifest.sourceRevision -TimeoutSeconds $manifest.timeoutSecondsPerRole
$author = Read-Json (Join-Path $trial 'Author/run.json')
if ($author.execution -cne 'COMPLETED') { throw 'Shim Author did not complete' }
$summary = @{schema=2; mode='TEST_SHIM'; cohort='Candidate'; maximumRoleCalls=2; cases=@(@{ id='E03'; status='BLOCKED';
    handoffShaMatches=$false; author=@{record='trials/E03/Author/run.json'; invoked=$true; execution='COMPLETED'; source='TEST_SHIM'};
    work=@{execution='NOT_RUN'; invoked=$false} })}
Save-Json (Join-Path $original 'compatibility-summary.json') $summary
& node $reporter seal $original
if ($LASTEXITCODE -ne 0) { throw 'Could not seal E03 Author evidence' }
$indexPath = Join-Path $original 'evidence-index.json'
$handoff = Join-Path $trial 'Author/final.md'
$argsForResume = @{Session='original';OutputRoot=$output;ResumeWork=$true;Execute=$true;MaxNativeCalls=1;CodexPath=$shim;
    ReviewedHandoffSha256=(Digest $handoff);OriginalEvidenceSha256=(Digest $indexPath)}
$continued = Join-Path $output 'original-work'
function Assert-Rejected([hashtable]$Parameters, [string]$Expected) {
    $rejected = $false
    try { & $runner @Parameters 2>&1 | Out-Null }
    catch { if ($_.Exception.Message -notmatch $Expected) { throw }; $rejected = $true }
    if (-not $rejected) { throw "Expected resume rejection: $Expected" }
}
$noApproval = $argsForResume.Clone()
$noApproval.Remove('ReviewedHandoffSha256')
Assert-Rejected $noApproval 'reviewed handoff'
$wrongApproval = $argsForResume.Clone()
$wrongApproval.ReviewedHandoffSha256 = '1' * 64
Assert-Rejected $wrongApproval 'approved handoff digest'
$wrongBudget = $argsForResume.Clone()
$wrongBudget.MaxNativeCalls = 2
Assert-Rejected $wrongBudget '1 role call'
$invalidSeal = $argsForResume.Clone()
$invalidSeal.OriginalEvidenceSha256 = '0' * 64
Assert-Rejected $invalidSeal 'original.*evidence'
$bytes = [IO.File]::ReadAllBytes($handoff)
try {
    Add-Content -LiteralPath $handoff -Value 'Tampered.'
    Assert-Rejected $argsForResume 'original.*evidence'
} finally { [IO.File]::WriteAllBytes($handoff, $bytes) }
$work = (New-Item -ItemType Directory -Path (Join-Path $trial 'Work')).FullName
try {
    Save-Json (Join-Path $work 'run.json') @{startUtc='2026-01-01T00:00:00Z'}
    Assert-Rejected $argsForResume 'original.*evidence'
} finally { [IO.File]::Delete((Join-Path $work 'run.json')) }
$setup = Join-Path $work 'session.json'
Save-Json $setup @{role='Work'; startUtc='2026-01-01T00:00:00Z'; finishUtc=$null; cases=@()}
$setupDigest = Digest $setup
$originalSeal = [IO.File]::ReadAllBytes($indexPath)
$changedRecords = @{}
try {
    $manifest.sourceRevision = 'f' * 40
    $index = Read-Json $indexPath
    foreach ($name in 'manifest.json', 'trials/E03/Author/run.json', 'trials/E03/Author/session.json') {
        $path = Join-Path $original $name
        $changedRecords[$path] = [IO.File]::ReadAllBytes($path)
        $record = Read-Json $path
        $record.sourceRevision = $manifest.sourceRevision
        Save-Json $path $record
        $index.files[$name] = @{sha256=(Digest $path);bytes=(Get-Item -LiteralPath $path).Length}
    }
    Save-Json $indexPath $index
    $mismatch = $argsForResume.Clone()
    $mismatch.OriginalEvidenceSha256 = Digest $indexPath
    Assert-Rejected $mismatch 'frozen source revision at HEAD'
} finally {
    foreach ($path in $changedRecords.Keys) { [IO.File]::WriteAllBytes($path, $changedRecords[$path]) }
    [IO.File]::WriteAllBytes($indexPath, $originalSeal)
}
$preflight = $argsForResume.Clone()
foreach ($key in 'Execute','MaxNativeCalls','CodexPath') { $preflight.Remove($key) }
& $runner @preflight
if ($LASTEXITCODE -ne 0) { throw 'Work resume preflight failed' }
if (Test-Path -LiteralPath $continued) { throw 'Invalid resume reserved a continuation' }
& $runner @argsForResume
if ($LASTEXITCODE -ne 0) { throw 'Offline Work-only resume failed' }
$combined = Read-Json (Join-Path $continued 'compatibility-summary.json')
if ($combined.cases[0].status -cne 'TEST_SHIM_OBJECTIVE_PASS' -or -not $combined.cases[0].handoffShaMatches -or
    (Digest (Join-Path $continued 'trials/E03/Author/final.md')) -cne $argsForResume.ReviewedHandoffSha256 -or
    (Digest $indexPath) -cne $argsForResume.OriginalEvidenceSha256 -or @(Get-ChildItem -LiteralPath $work).Count -ne 1 -or
    (Digest $setup) -cne $setupDigest) {
    throw 'Resume failed to preserve original evidence or Work-only scope'
}
Assert-Rejected $argsForResume 'continuation.*exists'
$reviewPath = Join-Path $continued 'review.json'
$review = Read-Json (Join-Path $continued 'review-template.json')
$review.reviewer = 'Offline review'
foreach ($group in $review.cases.E03.Values) {
    foreach ($item in $group) { $item.status='PASS'; $item.reason='Offline fixture'; $item.evidence=@('Work/shim-test') }
}
Save-Json $reviewPath $review
try {
    Add-Content -LiteralPath $handoff -Value 'Tampered after continuation sealing.'
    & $runner -Session original-work -OutputRoot $output -ReviewFile $reviewPath 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 1 -or (Test-Path -LiteralPath (Join-Path $continued 'quality-report.json'))) {
        throw 'Final review must revalidate the live original evidence against the link.'
    }
} finally { [IO.File]::WriteAllBytes($handoff, $bytes) }
& $runner -Session original-work -OutputRoot $output -ReviewFile $reviewPath
if ($LASTEXITCODE -ne 2 -or (Read-Json (Join-Path $continued 'quality-report.json')).status -cne 'NOT_RUN') {
    throw 'Offline continuation must review successfully without Native PASS'
}
Write-Host 'PASS Work-only continuation and rejecting unsupported resume attempts (zero model calls).'
exit 0
