#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$smoke = Join-Path $repo 'scripts/eval-codex-model-smoke.ps1'
$output = Join-Path ([IO.Path]::GetTempPath()) ('cirnet-model-smoke-test-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $output
$shim = Join-Path $output 'client-shim.ps1'

@'
if ($args -contains '--version') { 'codex model smoke test-shim'; exit 0 }
if ($args -contains '--help') {
    '--json --ephemeral --sandbox workspace-write read-only --output-last-message --cd --ask-for-approval never --config --model --disable --ignore-user-config --ignore-rules'
    exit 0
}
if ($args -contains 'login') { 'Logged in with synthetic shim'; exit 0 }
$final = $args[[array]::IndexOf($args, '--output-last-message') + 1]
$mode = $args[[array]::IndexOf($args, '--model') + 1]
$task = [Console]::In.ReadToEnd()
$e01 = Test-Path -LiteralPath accounts.mjs
$e06 = Test-Path -LiteralPath src/labels.mjs
$recovery = $e06 -and $mode -eq 'mock-success'
if ($mode -eq 'mock-blocked') { exit 7 }
if ($mode -eq 'mock-quota') { [Console]::Error.WriteLine('HTTP 429 too many requests'); exit 7 }
if ($e01 -and $task -cne 'Add a user data export feature.') { exit 8 }
$localRules = Get-Content -Raw -LiteralPath AGENTS.md
if ($mode -eq 'mock-success' -and $e01 -and $localRules.Contains('PowerShell test summary command:')) { exit 16 }
if ($mode -eq 'mock-read-only') {
    if ($args -notcontains 'read-only') { exit 18 }
    if (Test-Path -LiteralPath 'verify.mjs') {
        $output = & node verify.mjs
        $exitCode = $LASTEXITCODE
        if ($exitCode -ne 0 -or 'unit PASS (2 assertions)' -cnotin $output -or
            'integration SKIP (1 check; inputs/partner-batch.json unavailable)' -cnotin $output) { exit 21 }
        $answer = 'Unit PASS. Partner integration is BLOCKED. Completion remains open.'
    } else { $answer = 'Synthetic read-only evidence.' }
} else {
    if ($recovery) {
        if ($localRules -notmatch 'once before and once after editing' -or
            $localRules -notmatch 'Do not repeat successful preflight checks') { exit 14 }
        $testSummaryCommand = [regex]::Match($localRules, '(?m)^PowerShell test summary command: (.+)$').Groups[1].Value.TrimEnd("`r")
        if (-not $testSummaryCommand) { exit 17 }
        $testSummary = & (Join-Path $PSHOME 'pwsh') -NoProfile -NonInteractive -Command $testSummaryCommand
        if ($LASTEXITCODE -ne 0 -or '# pass 1' -notin $testSummary -or '# fail 0' -notin $testSummary) { exit 11 }
    }
    if ($e06) {
        $path = Join-Path $PWD 'src/labels.mjs'
        $source = [IO.File]::ReadAllText($path)
        $edited = $source -replace "(?m)^import \{ trimAlias \} from '\./text\.mjs';\r?\n(?:\r?\n)?", ''
        if ($mode -eq 'mock-objective-fail') { $edited = "`n" + $edited }
        [IO.File]::WriteAllText($path, $edited.Replace('trimAlias(name)', 'name.trim()'))
        if ($mode -eq 'mock-stage') {
            & git -c core.longpaths=true add -- src/labels.mjs
            if ($LASTEXITCODE -ne 0) { exit 20 }
        }
    }
    $answer = 'Synthetic engineering evidence.'
    if ($recovery) {
        $testSummary = & (Join-Path $PSHOME 'pwsh') -NoProfile -NonInteractive -Command $testSummaryCommand
        if ($LASTEXITCODE -ne 0 -or '# pass 1' -notin $testSummary -or '# skipped 0' -notin $testSummary) { exit 12 }
        $answer = 'The labels tests exited 0 before and after the scoped edit.'
    }
}
'{"type":"thread.started","thread_id":"model-smoke-shim"}'
'{"type":"turn.started"}'
if ($mode -eq 'mock-read-only' -and (Test-Path -LiteralPath 'verify.mjs')) {
    @{type='item.completed'; item=@{type='command_execution'; id='shim-verify'; command='node verify.mjs'; aggregated_output=$output -join "`n"; exit_code=$exitCode}} |
        ConvertTo-Json -Depth 4 -Compress
}
if ($recovery) {
    foreach ($item in 'shim-before','shim-after') {
        @{type='item.completed'; item=@{type='command_execution'; id=$item; command='node --test'; aggregated_output='# pass 1'; exit_code=0}} |
            ConvertTo-Json -Depth 4 -Compress
    }
}
if ($mode -eq 'mock-guards' -and $e01) {
    $value = [regex]::Match([IO.File]::ReadAllText((Join-Path $PWD 'accounts.mjs')), "recoveryCode:\s*'([^']+)'").Groups[1].Value
    @{type='item.completed'; item=@{type='command_execution'; id='synthetic-exposure'; aggregated_output=$value; exit_code=0}} |
        ConvertTo-Json -Depth 4 -Compress
}
[IO.File]::WriteAllText($final, $answer, [Text.UTF8Encoding]::new($false))
@{type='item.completed'; item=@{type='agent_message'; id='answer'; text=$answer}} | ConvertTo-Json -Depth 4 -Compress
'{"type":"turn.completed","usage":{"input_tokens":7,"cached_input_tokens":2,"output_tokens":3}}'
'@ | Set-Content -LiteralPath $shim -Encoding utf8NoBOM

function Read-Evidence([string]$Relative) {
    Get-Content -Raw -LiteralPath (Join-Path $output $Relative) | ConvertFrom-Json -AsHashtable
}

function Assert-Rejected([string]$Session, [hashtable]$Arguments, [string]$Pattern) {
    $rejected = $false
    try { & $smoke -Session $Session -Model mock -OutputRoot $output @Arguments }
    catch { if ($_.Exception.Message -notmatch $Pattern) { throw }; $rejected = $true }
    if (-not $rejected) { throw "Expected smoke validation failure for $Session." }
}

& $smoke -Session preview -Model mock -OutputRoot $output
$previewCode = $LASTEXITCODE
$preview = Read-Evidence 'preview/manifest.json'
if ($previewCode -ne 0 -or $preview.mode -ne 'PREVIEW' -or $preview.cohort -ne 'Candidate' -or
    ($preview.cases -join ',') -cne 'E01,E06' -or $preview.maximumRoleCalls -ne 2 -or
    @(Get-ChildItem -LiteralPath (Join-Path $output 'preview/trials')).Count -or
    (Test-Path -LiteralPath (Join-Path $output 'preview/compatibility-summary.json'))) {
    throw 'Preview did not freeze a two-call Candidate-only plan without invoking scenarios.'
}
$guide = 'en/docs/context.md'
$guideHash = (Get-FileHash -LiteralPath (Join-Path $repo $guide)).Hash.ToLowerInvariant()
$casesHash = (Get-FileHash -LiteralPath (Join-Path $repo 'tests/benchmark/cases.json')).Hash.ToLowerInvariant()
if ($preview.rules[$guide] -cne $guideHash -or $preview.evaluator['cases.json'] -cne $casesHash) {
    throw 'Snapshot did not capture the actual guide and fixture suite hashes.'
}

foreach ($id in @('P01', 'E05')) {
    & $smoke -Session "preview-$id" -Model mock -OutputRoot $output -CaseId $id
    if ($LASTEXITCODE -ne 0) { throw "Could not preview $id." }
    $lane = Read-Evidence "preview-$id/manifest.json"
    $canonical = if ($id -eq 'P01') { 'python-retry-count-refactor' } else { $id }
    if ($lane.mode -cne 'PREVIEW' -or @($lane.cases).Count -ne 1 -or $lane.cases[0] -cne $canonical -or $lane.maximumRoleCalls -ne 1 -or
        @(Get-ChildItem -LiteralPath (Join-Path $output "preview-$id/trials")).Count) {
        throw "Expected a one-call preview for only $id."
    }
    if ($lane.evaluator['fixtures/E01/accounts.mjs'] -cne $preview.evaluator['fixtures/E01/accounts.mjs'] -or
        $lane.evaluator['cases.json'] -cne $casesHash) {
        throw 'Both scenarios must freeze the canonical catalog and share the privacy auditor.'
    }
    if ($id -ceq 'P01') {
        foreach ($file in @('checks.mjs', "fixtures/$canonical/retry_count.py", "fixtures/$canonical/qa/contract_retries.py")) {
            $hash = (Get-FileHash -LiteralPath (Join-Path $repo "tests/benchmark/$file")).Hash.ToLowerInvariant()
            if ($lane.evaluator[$file] -cne $hash) { throw "P01 did not freeze $file." }
        }
        if ($lane.evaluator.Contains('fixtures/E05/rank.mjs')) { throw 'Snapshots must stage only selected fixtures and the shared audit input.' }
    }
    Assert-Rejected "wrong-budget-$id" @{ CaseId=$id; Execute=$true; MaxNativeCalls=2; CodexPath=$shim } 'Authorize up to 1'
}
& $smoke -Session mixed-preview -Model mock -OutputRoot $output -CaseId 'E02,P01,rust-timeout-bound'
$mixed = Read-Evidence 'mixed-preview/manifest.json'
if ($LASTEXITCODE -ne 0 -or ($mixed.cases -join ',') -cne 'E02,python-retry-count-refactor,rust-timeout-bound' -or
    $mixed.maximumRoleCalls -ne 3 -or $mixed.Contains('schedule') -or $mixed.Contains('snapshots') -or $mixed.Contains('baseline') -or
    -not $mixed.evaluator.Contains('fixtures/rust-timeout-bound/lib.rs')) {
    throw 'Mixed scenarios must freeze once in a single Candidate session.'
}
$mixedRoot = Join-Path $output 'mixed-preview'
foreach ($id in @('E02', 'python-retry-count-refactor', 'rust-timeout-bound')) {
    $trial = (New-Item -ItemType Directory -Path (Join-Path $mixedRoot "trials/$id")).FullName
    $rolePreview = @{
        BenchmarkRoot=$mixedRoot; OutputRoot=$trial; Session='Direct'; CaseId=$id
        RulesRoot=(Join-Path $mixedRoot 'rules'); SuiteRoot=(Join-Path $mixedRoot 'evaluator')
        Model='mock'; Effort='high'; SourceRevision=$mixed.sourceRevision
    }
    & (Join-Path $repo 'scripts/benchmark/role.ps1') @rolePreview
    $roleRecord = Read-Evidence "mixed-preview/trials/$id/Direct/run.json"
    $roleManifest = Read-Evidence "mixed-preview/trials/$id/Direct/session.json"
    if ($roleRecord.execution -cne 'NOT_RUN' -or $roleManifest.cases[0].evidence -cne 'run.json') {
        throw "Expected an isolated direct preview for $id."
    }
}
Assert-Rejected missing-shim @{ Execute=$true; MaxNativeCalls=2; CodexPath=(Join-Path $output 'absent.ps1') } 'existing script'
Assert-Rejected preview-credentials @{ UseApiKey=$true } 'Preview uses neither'
Assert-Rejected preview-measurement @{ MeasureInitialInput=$true } 'authorized native execution'
Assert-Rejected shim-measurement @{ MeasureInitialInput=$true; Execute=$true; MaxNativeCalls=2; CodexPath=$shim } 'authorized native execution'

$execute = @{ OutputRoot=$output; CodexPath=$shim; Execute=$true; MaxNativeCalls=2; TimeoutSeconds=30 }
& $smoke @execute -Session success -Model mock-success
$successCode = $LASTEXITCODE
$passed = Read-Evidence 'success/compatibility-summary.json'
if ($successCode -ne 0 -or $passed.mode -ne 'TEST_SHIM' -or $passed.independentReview -ne 'NOT_RUN' -or
    ($passed.cases.id -join ',') -cne 'E01,E06' -or $passed.Contains('fullBenchmark')) {
    throw 'Mock objectives must be distinguished from native evidence.'
}
foreach ($trial in $passed.cases) {
    $run = $trial.direct
    if ($trial.status -ne 'TEST_SHIM_OBJECTIVE_PASS' -or
        -not $run.invoked -or $run.verdict -ne 'NOT_RUN' -or $run.usage -ne 'NOT_AVAILABLE') {
        throw "Mock $($trial.id) did not record one unmeasured direct run."
    }
    $trace = [IO.File]::ReadAllText((Join-Path $output "success/trials/$($trial.id)/Direct/events.jsonl"))
    if ($trace -match 'input_tokens|output_tokens|cached_input_tokens') {
        throw 'Provider usage must be discarded from an unmeasured run.'
    }
}
$manifest = Read-Evidence 'success/manifest.json'
$catalog = Read-Evidence 'success/evaluator/cases.json'
foreach ($case in $passed.cases) {
    $expected = ($catalog | Where-Object id -CEQ $case.id).prompt
    if ([IO.File]::ReadAllText((Join-Path $output "success/trials/$($case.id)/Direct/prompt.txt")) -cne $expected) {
        throw 'Direct evaluation must use the frozen original task bytes.'
    }
}
$e01Direct = Read-Evidence 'success/trials/E01/Direct/run.json'
$e06Direct = Read-Evidence 'success/trials/E06/Direct/run.json'
$manifest = Read-Evidence 'success/manifest.json'
$directContext = Join-Path $output 'success/trials/E06/Direct/fixture/toolkit/en/docs/context.md'
if ($e06Direct.manifest.rules['toolkit/en/docs/context.md'] -cne $manifest.rules['en/docs/context.md'] -or
    (Get-FileHash -LiteralPath $directContext).Hash.ToLowerInvariant() -cne $manifest.rules['en/docs/context.md'] -or
    $e06Direct.manifest.rules.Contains('toolkit/en/docs/model-compatibility.md')) {
    throw 'Direct execution must receive the pinned context guide only.'
}
if ($e01Direct.command[[array]::IndexOf($e01Direct.command, '--sandbox') + 1] -cne 'read-only' -or
    $e06Direct.command[[array]::IndexOf($e06Direct.command, '--sandbox') + 1] -cne 'workspace-write') {
    throw 'E01 must be read-only while E06 retains the write sandbox.'
}
if ($e06Direct.executionScope.baselineSha256 -cnotmatch '^[a-f0-9]{64}$' -or
    $e06Direct.executionScope.preflightStatus -cne 'PASS' -or $e06Direct.executionScope.auditStatus -cne 'PASS' -or
    $e06Direct.executionScope.acceptanceStatus -cne 'NOT_RUN') {
    throw 'Candidate must distinguish a verified fixture baseline and audit from implementation acceptance.'
}

$e06Rules = Get-Content -Raw -LiteralPath (Join-Path $output 'success/trials/E06/Direct/fixture/AGENTS.md')
$testSummaryCommand = [regex]::Match($e06Rules, '(?m)^PowerShell test summary command: (.+)$').Groups[1].Value.TrimEnd("`r")
if (-not $testSummaryCommand) { throw 'Missing E06 test summary command.' }
$failedTestFolder = Join-Path $output 'summary-failure'
$null = New-Item -ItemType Directory -Path $failedTestFolder
@'
import { test } from 'node:test';
test('synthetic test name marker', () => { throw new Error('synthetic failure value marker'); });
'@ | Set-Content -LiteralPath (Join-Path $failedTestFolder 'failure.test.mjs') -Encoding utf8NoBOM
Push-Location -LiteralPath $failedTestFolder
try {
    $summary = & (Join-Path $PSHOME 'pwsh') -NoProfile -NonInteractive -Command $testSummaryCommand
    if ($LASTEXITCODE -ne 1 -or '# fail 1' -notin $summary -or $summary -match 'synthetic .* marker') {
        throw 'The filtered test summary must preserve the child failure without emitting names or error values.'
    }
} finally { Pop-Location }

$successRoot = Join-Path $output 'success'
$conflictRoot = (New-Item -ItemType Directory -Path (Join-Path $output 'scope-conflict')).FullName
$frozen = Read-Evidence 'success/manifest.json'
$index = Read-Evidence 'success/evidence-index.json'
foreach ($name in @($index.files.Keys; $frozen.rules.Keys | ForEach-Object { "rules/$_" }; $frozen.evaluator.Keys | ForEach-Object { "evaluator/$_" })) {
    $destination = Join-Path $conflictRoot $name
    $null = [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($destination))
    [IO.File]::Copy((Join-Path $successRoot $name), $destination)
}
$conflict = Read-Evidence 'success/trials/E06/Direct/run.json'
$conflictManifest = Read-Evidence 'success/manifest.json'
$conflictManifest.session = 'scope-conflict'
$conflictManifest | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath (Join-Path $conflictRoot 'manifest.json') -Encoding utf8NoBOM
$conflict.executionScope.auditStatus = 'FAIL'
$conflict | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath (Join-Path $conflictRoot 'trials/E06/Direct/run.json') -Encoding utf8NoBOM
$seal = & node (Join-Path $repo 'scripts/benchmark/report.mjs') seal $conflictRoot 2>&1
if ($LASTEXITCODE -ne 1 -or ($seal -join "`n") -notmatch 'Objective success conflicts with role evidence' -or
    (Test-Path -LiteralPath (Join-Path $conflictRoot 'evidence-index.json'))) {
    throw 'The reporter must refuse to seal objective success when the fixture audit failed.'
}
$review = Read-Evidence 'success/review-template.json'
$review.reviewer = 'Offline review fixture'
foreach ($id in $review.cases.Keys) {
    foreach ($key in $review.cases[$id].Keys) {
        foreach ($check in $review.cases[$id][$key]) {
            $check.status = 'PASS'
            $check.reason = 'Exercise the independent review and evidence gate.'
            $check.evidence = @('Direct/run.json')
        }
    }
}
$reviewPath = Join-Path $successRoot 'review.json'
$review | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $reviewPath -Encoding utf8NoBOM
& $smoke -Session success -OutputRoot $output -ReviewFile $reviewPath 2>&1 | Out-Null
if ($LASTEXITCODE -ne 1 -or (Test-Path -LiteralPath (Join-Path $successRoot 'quality-report.json'))) {
    throw 'Verification PASS must cite an observed direct command.'
}
$review.cases.E06.verificationCoverage[0].evidence = @('Direct/shim-before')
$review.cases.E06.verificationCoverage[1].evidence = @('Direct/shim-after')
$review | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $reviewPath -Encoding utf8NoBOM
$recordPath = Join-Path $successRoot 'trials/E06/Direct/run.json'
$recordBytes = [IO.File]::ReadAllBytes($recordPath)
try {
    $directRecord = Read-Evidence 'success/trials/E06/Direct/run.json'
    $directRecord.manifest.rules['.agents/skills/cirnet-refactoring/SKILL.md'] = '0' * 64
    $directRecord | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $recordPath -Encoding utf8NoBOM
    $rejection = & $smoke -Session success -OutputRoot $output -ReviewFile $reviewPath 2>&1
    if ($LASTEXITCODE -ne 1 -or ($rejection -join "`n") -notmatch 'Staged Skill hash mismatch' -or
        (Test-Path -LiteralPath (Join-Path $successRoot 'quality-report.json'))) {
        throw 'Evidence must retain the selected Skill hash from the frozen session.'
    }
} finally { [IO.File]::WriteAllBytes($recordPath, $recordBytes) }
$tracePath = Join-Path $successRoot 'trials/E06/Direct/events.jsonl'
$traceBytes = [IO.File]::ReadAllBytes($tracePath)
try {
    [IO.File]::AppendAllText($tracePath, "`n")
    & $smoke -Session success -OutputRoot $output -ReviewFile $reviewPath 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 1 -or (Test-Path -LiteralPath (Join-Path $successRoot 'quality-report.json'))) {
        throw 'A tampered trace must invalidate the sealed evidence index.'
    }
} finally { [IO.File]::WriteAllBytes($tracePath, $traceBytes) }
& $smoke -Session success -OutputRoot $output -ReviewFile $reviewPath
$qualityCode = $LASTEXITCODE
$quality = Read-Evidence 'success/quality-report.json'
if ($qualityCode -ne 2 -or $quality.status -ne 'NOT_RUN' -or $quality.source -ne 'TEST_SHIM' -or
    $quality.cases[1].quality.verificationCoverage.status -ne 'PASS' -or $quality.Contains('paired')) {
    throw 'A sealed, independently graded shim must not establish native quality PASS.'
}
$reportPath = Join-Path $successRoot 'quality-report.json'
$reportHash = (Get-FileHash -LiteralPath $reportPath).Hash
& $smoke -Session success -OutputRoot $output -ReviewFile $reviewPath 2>&1 | Out-Null
if ($LASTEXITCODE -ne 1 -or (Get-FileHash -LiteralPath $reportPath).Hash -cne $reportHash) {
    throw 'A session must not overwrite its quality report.'
}

& $smoke @execute -Session guards -Model mock-guards
$guardsCode = $LASTEXITCODE
$guarded = Read-Evidence 'guards/compatibility-summary.json'
if ($guardsCode -ne 1 -or $guarded.cases[0].direct.privacyHits -lt 1 -or $guarded.cases[0].status -ne 'FAIL') {
    throw 'Synthetic privacy exposure must fail.'
}
$fixture = [IO.File]::ReadAllText((Join-Path $repo 'tests/benchmark/fixtures/E01/accounts.mjs'))
$canary = [regex]::Match($fixture, "recoveryCode:\s*'([^']+)'").Groups[1].Value
foreach ($relative in @('guards/compatibility-summary.json', 'guards/trials/E01/Direct/events.jsonl')) {
    if (-not $canary -or [IO.File]::ReadAllText((Join-Path $output $relative)).Contains($canary)) {
        throw 'Synthetic canary was not withheld from role evidence.'
    }
}

& $smoke @execute -Session objective-fail -Model mock-objective-fail
$objectiveCode = $LASTEXITCODE
$objective = Read-Evidence 'objective-fail/compatibility-summary.json'
if ($objectiveCode -ne 1 -or $objective.cases[1].status -ne 'FAIL' -or
    $objective.cases[1].direct.execution -ne 'COMPLETED') {
    throw 'A completed direct run must fail if it leaves the removed import separator.'
}

& $smoke @execute -Session blocked -Model mock-blocked
$blockedCode = $LASTEXITCODE
$blocked = Read-Evidence 'blocked/compatibility-summary.json'
if ($blockedCode -ne 2 -or @($blocked.cases | Where-Object {
    $_.status -ne 'BLOCKED' -or $_.direct.usage -ne 'NOT_AVAILABLE'
}).Count) {
    throw 'Incomplete turns must stay blocked with missing usage marked unavailable.'
}

& $smoke @execute -Session shared-blocker -Model mock-quota
$quotaCode = $LASTEXITCODE
$quota = Read-Evidence 'shared-blocker/compatibility-summary.json'
if ($quotaCode -ne 2 -or $quota.cases[0].reason -notmatch 'quota' -or $quota.cases[1].status -ne 'NOT_RUN' -or
    (Test-Path -LiteralPath (Join-Path $output 'shared-blocker/trials/E06'))) {
    throw 'A shared runtime blocker must stop dispatch for remaining scenarios.'
}

& $smoke @execute -CaseId 'E04,E07' -Session read-only -Model mock-read-only
$readOnlyCode = $LASTEXITCODE
$readOnly = Read-Evidence 'read-only/compatibility-summary.json'
if ($readOnlyCode -ne 0 -or @($readOnly.cases | Where-Object { $_.status -ne 'TEST_SHIM_OBJECTIVE_PASS' }).Count) {
    throw 'Blocked dependencies and runnable verification must preserve read-only scope.'
}
foreach ($id in 'E04','E07') {
    $direct = Read-Evidence "read-only/trials/$id/Direct/run.json"
    $inputPath = Join-Path $output "read-only/trials/$id/Direct/prompt.txt"
    $expected = ($catalog | Where-Object id -CEQ $id).prompt
    if ($direct.command[[array]::IndexOf($direct.command,'--sandbox') + 1] -cne 'read-only' -or
        (Get-Content -LiteralPath $inputPath -TotalCount 1) -cne $expected) {
        throw "Unexpected read-only scope or task input for $id."
    }
}
$e07Root = Join-Path $output 'read-only'
$review = Read-Evidence 'read-only/review-template.json'
$review.reviewer = 'Offline E07 evidence gate'
foreach ($entry in $review.cases.Values) {
    foreach ($dimension in $entry.Values) {
        foreach ($check in $dimension) { $check.reason = 'Not evaluated in this focused reporter regression.' }
    }
}
$verification = $review.cases.E07.verificationCoverage
foreach ($check in $verification) {
    $check.status = 'PASS'
    $check.reason = 'The verifier returned exit zero.'
    $check.evidence = @('Direct/shim-verify')
}
$badReview = Join-Path $e07Root 'unsupported-review.json'
$review | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $badReview -Encoding utf8NoBOM
$rejected = & $smoke -Session read-only -OutputRoot $output -ReviewFile $badReview 2>&1
if ($LASTEXITCODE -ne 1 -or ($rejected -join "`n") -notmatch 'Partner integration cannot pass' -or
    (Test-Path -LiteralPath (Join-Path $e07Root 'quality-report.json'))) {
    throw 'E07 verifier exit zero must not allow integration PASS without partner input.'
}
$verification[1].status = 'BLOCKED'
$verification[1].reason = 'The verifier reported integration SKIP because the partner input was missing.'
$validReview = Join-Path $e07Root 'review.json'
$review | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $validReview -Encoding utf8NoBOM
& $smoke -Session read-only -OutputRoot $output -ReviewFile $validReview | Out-Null
if ($LASTEXITCODE -ne 2) { throw 'Honest E07 review must be recorded by the shim without claiming native PASS.' }
$quality = (Read-Evidence 'read-only/quality-report.json').cases | Where-Object id -CEQ E07
if ($quality.quality.verificationCoverage.checks[0].status -cne 'PASS' -or
    $quality.quality.verificationCoverage.checks[1].status -cne 'BLOCKED') {
    throw 'E07 review must preserve unit PASS and integration BLOCKED separately.'
}

$singleArgs = $execute.Clone()
$stageArgs = $execute.Clone()
$stageArgs.MaxNativeCalls = 1
& $smoke @stageArgs -Session staged-change -Model mock-stage -CaseId E06
$stageCode = $LASTEXITCODE
$stageRun = Read-Evidence 'staged-change/trials/E06/Direct/run.json'
if ($stageCode -ne 1 -or $stageRun.verdict -cne 'FAIL' -or
    './.git/index-entries' -cnotin $stageRun.manifest.fixtureGitChanges -or $stageRun.objective.status -cne 'PASS' -or
    $stageRun.executionScope.preflightStatus -cne 'PASS' -or $stageRun.executionScope.auditStatus -cne 'FAIL' -or
    $stageRun.executionScope.acceptanceStatus -cne 'NOT_RUN') {
    throw 'A correct fixture edit must still fail when the Candidate stages it without authorization.'
}
$singleArgs.MaxNativeCalls = 1
& $smoke @singleArgs -CaseId E05 -Session blocked-single -Model mock-blocked
$singleCode = $LASTEXITCODE
$single = Read-Evidence 'blocked-single/compatibility-summary.json'
if ($singleCode -ne 2 -or $single.cases -isnot [array] -or $single.cases.Count -ne 1 -or
    $single.cases[0].id -cne 'E05' -or $single.cases[0].status -cne 'BLOCKED') {
    throw 'Single-case summaries must retain the cases array.'
}
Write-Host "PASS Candidate-only smoke preview and local shim checks. Evidence $output"
# The CI shell forwards $LASTEXITCODE. Return success only after every assertion passes.
exit 0
