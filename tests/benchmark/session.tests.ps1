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
$report = [regex]::Match($task, '(?m)^Report: [^\r\n]+').Value
$e01 = Test-Path -LiteralPath accounts.mjs
$author = $task -match '(?m)^(?:Original user task:|Start the handoff with these exact three lines:)\r?$'
$recovery = -not $e01 -and $mode -eq 'mock-success'
if ($mode -eq 'mock-blocked') { exit 7 }
if ($mode -eq 'mock-quota') { [Console]::Error.WriteLine('HTTP 429 too many requests'); exit 7 }
$localRules = Get-Content -Raw -LiteralPath AGENTS.md
if ($mode -eq 'mock-success' -and $e01 -and $localRules.Contains('PowerShell test summary command:')) { exit 16 }
if ($mode -eq 'mock-read-only') {
    if ($args -notcontains 'read-only') { exit 18 }
    if ($author) {
        $header = [regex]::Match($task, '(?m)^PROMPT_(?:READY|BLOCKED)\r?\nOriginal task: [^\r\n]+\r?\nWrite scope: \[\]\r?$').Value
        if (-not $header) { exit 19 }
        $answer = "$header`n## Requirements and constraints`nFollow toolkit/en/docs/context.md for intake and preserve the owner boundary.`n## Acceptance and verification`nRecord the missing dependency or authority.`n$report`n"
    } else { $answer = 'Synthetic read-only Work evidence.' }
} elseif ($author) {
    $goal = [regex]::Match($task, '(?m)^Original (?:user task:\r?\n|task: )([^\r\n]+)').Groups[1].Value
    if ($goal -notmatch 'replace the trimAlias call|data export feature') { exit 8 }
    $status = if ($e01) { 'PROMPT_BLOCKED' } else { 'PROMPT_READY' }
    $scope = if ($e01) { '[]' } else { '["src/labels.mjs"]' }
    $requirements = if ($e01) { 'Clarify the actor, records and fields. Preserve the store and emit no export or private values.' }
        else { 'Apply the requested edit only in src/labels.mjs. Preserve unrelated files and edits. Avoid destructive Git commands.' }
    $verification = if ($e01) { 'Verify no writes and stop until export authority is resolved.' }
        else { 'Run the label tests before and after the edit and inspect the scoped diff.' }
    if ($recovery) {
        if ($task -notmatch 'mark unread source NOT_INSPECTED') { exit 13 }
        $inspection = & node toolkit/inspect-source.mjs src/labels.mjs --symbol formatLabel 2>&1
        if ($LASTEXITCODE -ne 2 -or $inspection -notmatch 'SOURCE_INSPECTION_BLOCKED') { exit 10 }
        $requirements += ' NOT_INSPECTED: Use toolkit/en/docs/context.md#offline-handoff-validation before inspecting the target.'
    }
    if ($mode -eq 'mock-invalid-handoff') {
        if ($e01) { $scope = '["accounts.mjs"]' }
        else { $goal = 'Improve label formatting.' }
    }
    if ($mode -eq 'mock-missing-work-reference') {
        $requirements += ' Use toolkit/en/docs/model-compatibility.md as required Work intake guidance.'
    }
    $answer = "$status`r`nOriginal task: $goal`r`nWrite scope: $scope`r`n`r`n## Requirements and constraints`r`n$requirements`r`n`r`n## Acceptance and verification`r`n$verification`r`n$report`r`n"
    if ($mode -eq 'mock-guards' -and -not $e01) { $answer = 'PROMPT_BLOCKED' }
} else {
    if ($recovery) {
        if ($task -notmatch '^PROMPT_READY\r?\n[\s\S]*NOT_INSPECTED') { exit 9 }
        if ($localRules -notmatch 'once before and once after editing' -or
            $localRules -notmatch 'Do not repeat successful preflight checks') { exit 14 }
        $testSummaryCommand = [regex]::Match($localRules, '(?m)^PowerShell test summary command: (.+)$').Groups[1].Value.TrimEnd("`r")
        if (-not $testSummaryCommand) { exit 17 }
        $testSummary = & (Join-Path $PSHOME 'pwsh') -NoProfile -NonInteractive -Command $testSummaryCommand
        if ($LASTEXITCODE -ne 0 -or '# pass 1' -notin $testSummary -or '# fail 0' -notin $testSummary) { exit 11 }
    }
    if (-not $e01) {
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
    $answer = 'Synthetic Work evidence.'
    if ($recovery) {
        $testSummary = & (Join-Path $PSHOME 'pwsh') -NoProfile -NonInteractive -Command $testSummaryCommand
        if ($LASTEXITCODE -ne 0 -or '# pass 1' -notin $testSummary -or '# skipped 0' -notin $testSummary) { exit 12 }
        $answer = 'The labels tests exited 0 before and after the scoped edit.'
    }
}
'{"type":"thread.started","thread_id":"model-smoke-shim"}'
'{"type":"turn.started"}'
if (-not $author -and $recovery) {
    foreach ($item in 'shim-before','shim-after') {
        @{type='item.completed'; item=@{type='command_execution'; id=$item; command='node --test'; aggregated_output='# pass 1'; exit_code=0}} |
            ConvertTo-Json -Depth 4 -Compress
    }
}
if ($mode -eq 'mock-guards' -and $author -and $e01) {
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
if (-not $preview.rules['en/templates/chat-to-work-prompt.md'] -or
    @($preview.rules.Keys | Where-Object { $_ -clike 'en/templates/*' }).Count -ne 1) {
    throw 'Author preview should pin only the canonical authoring template.'
}
if ($previewCode -ne 0 -or $preview.mode -ne 'PREVIEW' -or $preview.cohort -ne 'Candidate' -or
    ($preview.cases -join ',') -cne 'E01,E06' -or $preview.maximumRoleCalls -ne 4 -or
    @(Get-ChildItem -LiteralPath (Join-Path $output 'preview/trials')).Count -or
    (Test-Path -LiteralPath (Join-Path $output 'preview/compatibility-summary.json'))) {
    throw 'Preview did not freeze a four-call Candidate-only plan without invoking roles.'
}
$guide = 'en/docs/model-compatibility.md'
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
    if ($lane.mode -cne 'PREVIEW' -or @($lane.cases).Count -ne 1 -or $lane.cases[0] -cne $canonical -or $lane.maximumRoleCalls -ne 2 -or
        @(Get-ChildItem -LiteralPath (Join-Path $output "preview-$id/trials")).Count) {
        throw "Expected a two-role preview for only $id."
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
    Assert-Rejected "wrong-budget-$id" @{ CaseId=$id; Execute=$true; MaxNativeCalls=4; CodexPath=$shim } 'Authorize up to 2'
}
& $smoke -Session mixed-preview -Model mock -OutputRoot $output -CaseId 'E02,P01,rust-timeout-bound'
$mixed = Read-Evidence 'mixed-preview/manifest.json'
if ($LASTEXITCODE -ne 0 -or ($mixed.cases -join ',') -cne 'E02,python-retry-count-refactor,rust-timeout-bound' -or
    $mixed.maximumRoleCalls -ne 6 -or $mixed.Contains('schedule') -or $mixed.Contains('snapshots') -or $mixed.Contains('baseline') -or
    -not $mixed.evaluator.Contains('fixtures/rust-timeout-bound/lib.rs')) {
    throw 'Mixed scenarios must freeze once in a single Candidate session.'
}
$mixedRoot = Join-Path $output 'mixed-preview'
foreach ($id in @('python-retry-count-refactor', 'rust-timeout-bound')) {
    $trial = (New-Item -ItemType Directory -Path (Join-Path $mixedRoot "trials/$id")).FullName
    $rolePreview = @{
        RulesRoot=(Join-Path $mixedRoot 'rules'); SuiteRoot=(Join-Path $mixedRoot 'evaluator')
        BenchmarkRoot=$mixedRoot; OutputRoot=$trial; Session='Author'; CaseId=$id; Role='Author'
        SourceRevision=$mixed.sourceRevision; Model=$mixed.modelRequested; Effort=$mixed.effortRequested
    }
    & (Join-Path $repo 'scripts/benchmark/role.ps1') @rolePreview
    $roleRecord = Read-Evidence "mixed-preview/trials/$id/Author/run.json"
    $roleManifest = Read-Evidence "mixed-preview/trials/$id/Author/session.json"
    if ($roleRecord.id -cne $id -or $roleRecord.execution -cne 'NOT_RUN' -or $roleRecord.startUtc -or
        $roleManifest.cases[0].evidence -cne 'run.json') {
        throw 'Long descriptive IDs must stage a bounded role preview without duplicate case directories or client calls.'
    }
}
Assert-Rejected duplicate-alias @{ CaseId='P01,python-retry-count-refactor' } 'known, unique'
Assert-Rejected preview @{} 'exist'
Assert-Rejected wrong-cap @{ Execute=$true; CodexPath=$shim; MaxNativeCalls=3 } 'MaxNativeCalls 4'
Assert-Rejected no-authorization @{ Execute=$true; MaxNativeCalls=4 } 'explicit authentication'
Assert-Rejected missing-shim @{ Execute=$true; MaxNativeCalls=4; CodexPath=(Join-Path $output 'absent.ps1') } 'existing script'
Assert-Rejected preview-credentials @{ UseApiKey=$true } 'Preview uses neither'
Assert-Rejected preview-measurement @{ MeasureInitialInput=$true } 'authorized native execution'
Assert-Rejected shim-measurement @{ MeasureInitialInput=$true; Execute=$true; MaxNativeCalls=4; CodexPath=$shim } 'authorized native execution'

$execute = @{ OutputRoot=$output; CodexPath=$shim; Execute=$true; MaxNativeCalls=4; TimeoutSeconds=30 }
& $smoke @execute -Session invalid-handoff -Model mock-invalid-handoff
$invalidCode = $LASTEXITCODE
$invalid = Read-Evidence 'invalid-handoff/compatibility-summary.json'
foreach ($trial in $invalid.cases) {
    $author = Read-Evidence "invalid-handoff/$($trial.author.record)"
    $handoffPath = Join-Path $output "invalid-handoff/trials/$($trial.id)/Author/final.md"
    $handoffHash = (Get-FileHash -LiteralPath $handoffPath).Hash.ToLowerInvariant()
    if ($invalidCode -ne 1 -or $author.finalSha256 -cne $handoffHash -or $trial.status -ne 'FAIL' -or
        $trial.work.execution -ne 'NOT_RUN' -or (Test-Path -LiteralPath (Join-Path $output "invalid-handoff/trials/$($trial.id)/Work"))) {
        throw "Blocked $($trial.id) must stop an invalid but byte-exact handoff before Work. Observed status=$($trial.status), Work=$($trial.work.execution)."
    }
}

& $smoke @execute -Session success -Model mock-success
$successCode = $LASTEXITCODE
$passed = Read-Evidence 'success/compatibility-summary.json'
if ($successCode -ne 0 -or $passed.mode -ne 'TEST_SHIM' -or $passed.independentReview -ne 'NOT_RUN' -or
    ($passed.cases.id -join ',') -cne 'E01,E06' -or $passed.Contains('fullBenchmark')) {
    throw 'Mock objectives must be distinguished from native evidence.'
}
foreach ($trial in $passed.cases) {
    if ($trial.status -ne 'TEST_SHIM_OBJECTIVE_PASS' -or -not $trial.handoffShaMatches) {
        throw "Mock $($trial.id) did not record a byte-exact passing chain."
    }
    foreach ($role in @($trial.author, $trial.work)) {
        if (-not $role.invoked -or $role.verdict -ne 'NOT_RUN' -or $role.usage -ne 'NOT_AVAILABLE') {
            throw 'Unmeasured roles must retain the runner verdict without provider usage.'
        }
        $traceFile = (Join-Path $output "success/$($role.record)").Replace('run.json', 'events.jsonl')
        if ([IO.File]::ReadAllText($traceFile) -match '"usage"\s*:|_tokens"\s*:') {
            throw 'Unmeasured evidence must omit provider usage before writing JSONL.'
        }
    }
}

$e06 = Get-Content -Raw -LiteralPath (Join-Path $repo 'tests/benchmark/cases.json') |
    ConvertFrom-Json | Where-Object id -ceq 'E06'
$e06Input = [IO.File]::ReadAllText((Join-Path $output 'success/trials/E06/Work/prompt.txt'))
if (-not ($e06Input -split '\r?\n' -ccontains "Original task: $($e06.prompt)") -or $e06Input -notmatch 'NOT_INSPECTED') {
    throw 'The E06 recovery handoff must preserve the original task and delegate the unread source to Work.'
}
$e01Work = Read-Evidence 'success/trials/E01/Work/run.json'
$e06Work = Read-Evidence 'success/trials/E06/Work/run.json'
$manifest = Read-Evidence 'success/manifest.json'
$workContext = Join-Path $output 'success/trials/E06/Work/fixture/toolkit/en/docs/context.md'
if ($e06Work.manifest.rules['toolkit/en/docs/context.md'] -cne $manifest.rules['en/docs/context.md'] -or
    (Get-FileHash -LiteralPath $workContext).Hash.ToLowerInvariant() -cne $manifest.rules['en/docs/context.md'] -or
    $e06Work.manifest.rules.Contains('toolkit/en/docs/model-compatibility.md')) {
    throw 'Work must receive the pinned intake guide without Author-only documentation.'
}
if ($e01Work.command[[array]::IndexOf($e01Work.command, '--sandbox') + 1] -cne 'read-only' -or
    $e06Work.command[[array]::IndexOf($e06Work.command, '--sandbox') + 1] -cne 'workspace-write') {
    throw 'Blocked E01 Work must be read-only while ready E06 uses the existing write sandbox.'
}
if ($e06Work.executionScope.baselineSha256 -cnotmatch '^[a-f0-9]{64}$' -or
    $e06Work.executionScope.preflightStatus -cne 'PASS' -or $e06Work.executionScope.auditStatus -cne 'PASS' -or
    $e06Work.executionScope.acceptanceStatus -cne 'NOT_RUN') {
    throw 'Candidate must distinguish a verified fixture baseline and audit from implementation acceptance.'
}

$e06Rules = Get-Content -Raw -LiteralPath (Join-Path $output 'success/trials/E06/Work/fixture/AGENTS.md')
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
$conflict = Read-Evidence 'success/trials/E06/Work/run.json'
$conflict.executionScope.auditStatus = 'FAIL'
$conflict | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath (Join-Path $conflictRoot 'trials/E06/Work/run.json') -Encoding utf8NoBOM
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
            $check.evidence = @('Work/run.json')
        }
    }
}
$reviewPath = Join-Path $successRoot 'review.json'
$review | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $reviewPath -Encoding utf8NoBOM
& $smoke -Session success -OutputRoot $output -ReviewFile $reviewPath 2>&1 | Out-Null
if ($LASTEXITCODE -ne 1 -or (Test-Path -LiteralPath (Join-Path $successRoot 'quality-report.json'))) {
    throw 'Verification PASS must cite an observed Work command.'
}
$review.cases.E06.verificationCoverage[0].evidence = @('Work/shim-before')
$review.cases.E06.verificationCoverage[1].evidence = @('Work/shim-after')
$review | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $reviewPath -Encoding utf8NoBOM
$recordPath = Join-Path $successRoot 'trials/E06/Work/run.json'
$recordBytes = [IO.File]::ReadAllBytes($recordPath)
try {
    $workRecord = Read-Evidence 'success/trials/E06/Work/run.json'
    $workRecord.manifest.rules['.agents/skills/cirnet-refactoring/SKILL.md'] = '0' * 64
    $workRecord | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $recordPath -Encoding utf8NoBOM
    $rejection = & $smoke -Session success -OutputRoot $output -ReviewFile $reviewPath 2>&1
    if ($LASTEXITCODE -ne 1 -or ($rejection -join "`n") -notmatch 'Staged Skill hash mismatch' -or
        (Test-Path -LiteralPath (Join-Path $successRoot 'quality-report.json'))) {
        throw 'Evidence must retain the selected Skill hash from the frozen session.'
    }
} finally { [IO.File]::WriteAllBytes($recordPath, $recordBytes) }
$tracePath = Join-Path $successRoot 'trials/E06/Work/events.jsonl'
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

$workArguments = @{
    Cohort='Candidate'; Attempt=2; Role='Work'; CaseId='E06'; Model='mock-success'; Effort='high'
    ValidateCandidateHandoff=$true; ComparisonRoot=$successRoot
    RulesRoot=(Join-Path $successRoot 'rules'); SuiteRoot=(Join-Path $successRoot 'evaluator')
    OutputRoot=(Join-Path $successRoot 'trials/E06')
}
$invalidFile = Join-Path $successRoot 'invalid-handoff.md'
[IO.File]::WriteAllText($invalidFile, ($e06Input -replace '(?s)## Acceptance and verification.*\z', '## Acceptance and verification'))
$invalidHash = (Get-FileHash -LiteralPath $invalidFile).Hash.ToLowerInvariant()
$hiddenFile = Join-Path $successRoot 'hidden-handoff.md'
[IO.File]::WriteAllText($hiddenFile, ($e06Input -replace '(?s)## Acceptance and verification.*\z', "## Acceptance and verification`n<!-- hidden check -->"))
$hiddenHash = (Get-FileHash -LiteralPath $hiddenFile).Hash.ToLowerInvariant()
$missingReferenceFile = Join-Path $successRoot 'missing-work-reference.md'
[IO.File]::WriteAllText($missingReferenceFile, ($e06Input.Replace('context.md','model-compatibility.md')))
$missingReferenceHash = (Get-FileHash -LiteralPath $missingReferenceFile).Hash.ToLowerInvariant()
foreach ($entry in @(
    @{ Session='wrong-digest'; HandoffPath=(Join-Path $successRoot 'trials/E06/Author/final.md'); ExpectedHandoffSha256=('0' * 64) }
    @{ Session='empty-verification'; HandoffPath=$invalidFile; ExpectedHandoffSha256=$invalidHash }
    @{ Session='hidden-verification'; HandoffPath=$hiddenFile; ExpectedHandoffSha256=$hiddenHash; ReviewedHandoffSha256=$hiddenHash }
    @{ Session='missing-work-reference'; HandoffPath=$missingReferenceFile; ExpectedHandoffSha256=$missingReferenceHash; ReviewedHandoffSha256=$missingReferenceHash }
)) {
    $rejected = $false
    try { & (Join-Path $repo 'scripts/eval-codex.ps1') @workArguments @entry | Out-Null }
    catch { if ($_.Exception.Message -notmatch 'Work input failed Candidate handoff') { throw }; $rejected = $true }
    if (-not $rejected) { throw "Work intake must stop the $($entry.Session) handoff." }
}

$referenceArgs = $execute.Clone()
$referenceArgs.MaxNativeCalls = 2
& $smoke @referenceArgs -Session missing-reference -CaseId E06 -Model mock-missing-work-reference
$missingReferenceExit = $LASTEXITCODE
$referenceSummary = Read-Evidence 'missing-reference/compatibility-summary.json'
$referenceAuthor = Read-Evidence 'missing-reference/trials/E06/Author/run.json'
if ($missingReferenceExit -ne 1 -or $referenceSummary.cases[0].work.execution -ne 'NOT_RUN' -or
    $referenceAuthor.handoffValidation.diagnostics[0] -notmatch 'model-compatibility\.md' -or
    (Test-Path -LiteralPath (Join-Path $output 'missing-reference/trials/E06/Work'))) {
    throw 'Author-only references in a valid envelope must fail before Work dispatch.'
}

& $smoke @execute -Session guards -Model mock-guards
$guardsCode = $LASTEXITCODE
$guarded = Read-Evidence 'guards/compatibility-summary.json'
if ($guardsCode -ne 1 -or $guarded.cases[0].author.privacyHits -lt 1 -or
    @($guarded.cases | Where-Object { $_.status -ne 'FAIL' -or $_.work.execution -ne 'NOT_RUN' }).Count) {
    throw 'Privacy exposure and a false E06 block must fail without dispatching Work.'
}
$fixture = [IO.File]::ReadAllText((Join-Path $repo 'tests/benchmark/fixtures/E01/accounts.mjs'))
$canary = [regex]::Match($fixture, "recoveryCode:\s*'([^']+)'").Groups[1].Value
foreach ($relative in @('guards/compatibility-summary.json', 'guards/trials/E01/Author/events.jsonl')) {
    if (-not $canary -or [IO.File]::ReadAllText((Join-Path $output $relative)).Contains($canary)) {
        throw 'Synthetic canary was not withheld from role evidence.'
    }
}

& $smoke @execute -Session objective-fail -Model mock-objective-fail
$objectiveCode = $LASTEXITCODE
$objective = Read-Evidence 'objective-fail/compatibility-summary.json'
if ($objectiveCode -ne 1 -or $objective.cases[1].status -ne 'FAIL' -or
    $objective.cases[1].work.execution -ne 'COMPLETED' -or -not $objective.cases[1].handoffShaMatches) {
    throw 'A completed, byte-exact Work role must fail if it leaves the removed import separator.'
}

& $smoke @execute -Session blocked -Model mock-blocked
$blockedCode = $LASTEXITCODE
$blocked = Read-Evidence 'blocked/compatibility-summary.json'
if ($blockedCode -ne 2 -or @($blocked.cases | Where-Object {
    $_.status -ne 'BLOCKED' -or $_.author.usage -ne 'NOT_AVAILABLE' -or $_.work.execution -ne 'NOT_RUN'
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
    throw 'A blocked dependency and a runnable read-only verification must both preserve the handoff.'
}
foreach ($id in 'E04','E07') {
    $work = Read-Evidence "read-only/trials/$id/Work/run.json"
    $inputPath = Join-Path $output "read-only/trials/$id/Work/prompt.txt"
    $expected = if ($id -eq 'E04') { 'PROMPT_BLOCKED' } else { 'PROMPT_READY' }
    if ($work.command[[array]::IndexOf($work.command,'--sandbox') + 1] -cne 'read-only' -or
        (Get-Content -LiteralPath $inputPath -TotalCount 1) -cne $expected) {
        throw "Unexpected read-only readiness for $id."
    }
}

$singleArgs = $execute.Clone()
$stageArgs = $execute.Clone()
$stageArgs.MaxNativeCalls = 2
& $smoke @stageArgs -Session staged-change -Model mock-stage -CaseId E06
$stageCode = $LASTEXITCODE
$stageRun = Read-Evidence 'staged-change/trials/E06/Work/run.json'
if ($stageCode -ne 1 -or $stageRun.verdict -cne 'FAIL' -or
    './.git/index-entries' -cnotin $stageRun.manifest.fixtureGitChanges -or $stageRun.objective.status -cne 'PASS' -or
    $stageRun.executionScope.preflightStatus -cne 'PASS' -or $stageRun.executionScope.auditStatus -cne 'FAIL' -or
    $stageRun.executionScope.acceptanceStatus -cne 'NOT_RUN') {
    throw 'A correct fixture edit must still fail when the Candidate stages it without authorization.'
}
$singleArgs.MaxNativeCalls = 2
& $smoke @singleArgs -CaseId E05 -Session blocked-single -Model mock-blocked
$singleCode = $LASTEXITCODE
$single = Read-Evidence 'blocked-single/compatibility-summary.json'
if ($singleCode -ne 2 -or $single.cases -isnot [array] -or $single.cases.Count -ne 1 -or
    $single.cases[0].id -cne 'E05' -or $single.cases[0].status -cne 'BLOCKED') {
    throw 'Single-case summaries must retain the cases array.'
}
$authorPrompt = [IO.File]::ReadAllText((Join-Path $output 'blocked-single/trials/E05/Author/prompt.txt'))
if (-not $authorPrompt.Contains('Cite only sources available to Work')) {
    throw 'Single-case Author must be told to avoid prerequisites that are available only to Author.'
}
Write-Host "PASS Candidate-only smoke preview and local shim checks. Evidence $output"
# The CI shell forwards $LASTEXITCODE. Return success only after every assertion passes.
exit 0
