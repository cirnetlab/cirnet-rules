#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
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
$author = $task -match '(?m)^(?:Original user task:|Start the handoff with these exact three lines:)\r?$'
$recovery = -not $e01 -and $mode -eq 'mock-success'
if ($mode -eq 'mock-blocked') { exit 7 }
$localRules = Get-Content -Raw -LiteralPath AGENTS.md
if ($mode -eq 'mock-success' -and $e01 -and $localRules.Contains('PowerShell test summary command:')) { exit 16 }
if ($author) {
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
        $requirements += ' NOT_INSPECTED: Verify the target under the existing read scope before editing.'
    }
    if ($mode -eq 'mock-invalid-handoff') {
        if ($e01) { $scope = '["accounts.mjs"]' }
        else { $goal = 'Improve label formatting.' }
    }
    $answer = "$status`r`nOriginal task: $goal`r`nWrite scope: $scope`r`n`r`n## Requirements and constraints`r`n$requirements`r`n`r`n## Acceptance and verification`r`n$verification`r`n"
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
if ($previewCode -ne 0 -or $preview.mode -ne 'PREVIEW' -or $preview.cohort -ne 'Candidate' -or
    ($preview.cases -join ',') -cne 'E01,E06' -or $preview.maximumRoleCalls -ne 4 -or
    @(Get-ChildItem -LiteralPath (Join-Path $output 'preview/trials')).Count -or
    (Test-Path -LiteralPath (Join-Path $output 'preview/compatibility-summary.json'))) {
    throw 'Preview did not freeze a four-call Candidate-only plan without invoking roles.'
}
$guide = 'en/docs/model-compatibility.md'
$guideHash = (Get-FileHash -LiteralPath (Join-Path $repo $guide)).Hash.ToLowerInvariant()
$casesHash = (Get-FileHash -LiteralPath (Join-Path $repo 'tests/behavioral/cases.json')).Hash.ToLowerInvariant()
if ($preview.rules[$guide] -cne $guideHash -or $preview.evaluator['cases.json'] -cne $casesHash) {
    throw 'Snapshot did not capture the actual guide and fixture suite hashes.'
}

foreach ($id in @('P01', 'E05')) {
    & $smoke -Session "preview-$id" -Model mock -OutputRoot $output -CaseId $id
    if ($LASTEXITCODE -ne 0) { throw "Could not preview $id." }
    $lane = Read-Evidence "preview-$id/manifest.json"
    if ($lane.mode -cne 'PREVIEW' -or @($lane.cases).Count -ne 1 -or $lane.cases[0] -cne $id -or $lane.maximumRoleCalls -ne 2 -or
        @(Get-ChildItem -LiteralPath (Join-Path $output "preview-$id/trials")).Count) {
        throw "Expected a two-role preview for only $id."
    }
    if ($lane.evaluator['fixtures/E01/accounts.mjs'] -cne $preview.evaluator['fixtures/E01/accounts.mjs'] -or
        $lane.rules['en/skills/cirnet-refactoring/SKILL.md'] -cne $preview.rules['en/skills/cirnet-refactoring/SKILL.md']) {
        throw 'The independent lane must preserve the privacy auditor and use the current execution Skill.'
    }
    if ($id -ceq 'P01') {
        $pythonSuite = Join-Path $repo 'tests/behavioral-python'
        foreach ($file in @('cases.json', 'checks.mjs', 'fixtures/P01/retry_count.py', 'fixtures/P01/qa/contract_retries.py')) {
            $hash = (Get-FileHash -LiteralPath (Join-Path $pythonSuite $file)).Hash.ToLowerInvariant()
            if ($lane.evaluator[$file] -cne $hash) { throw "P01 did not freeze $file." }
        }
        if ($lane.evaluator.Contains('fixtures/E05/rank.mjs')) { throw 'The Python suite must stay separate.' }
    } elseif ($lane.evaluator['cases.json'] -cne $casesHash) {
        throw 'E05 must use the original engineering suite.'
    }
    Assert-Rejected "wrong-budget-$id" @{ CaseId=$id; Execute=$true; MaxNativeCalls=4; CodexPath=$shim } 'Authorize up to 2'
}
Assert-Rejected preview @{} 'exist'
Assert-Rejected wrong-cap @{ Execute=$true; CodexPath=$shim; MaxNativeCalls=3 } 'MaxNativeCalls 4'
Assert-Rejected no-authorization @{ Execute=$true; MaxNativeCalls=4 } 'explicit authentication'
Assert-Rejected missing-shim @{ Execute=$true; MaxNativeCalls=4; CodexPath=(Join-Path $output 'absent.ps1') } 'existing script'
Assert-Rejected preview-credentials @{ UseApiKey=$true } 'Preview uses neither'

$execute = @{ OutputRoot=$output; CodexPath=$shim; Execute=$true; MaxNativeCalls=4; TimeoutSeconds=30 }
& $smoke @execute -Session invalid-handoff -Model mock-invalid-handoff
$invalidCode = $LASTEXITCODE
$invalid = Read-Evidence 'invalid-handoff/compatibility-summary.json'
foreach ($trial in $invalid.cases) {
    $author = Read-Evidence "invalid-handoff/$($trial.author.record)"
    $handoffPath = Join-Path $output "invalid-handoff/trials/$($trial.id)/Author/$($trial.id)/final.md"
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
    ($passed.cases.id -join ',') -cne 'E01,E06' -or $passed.fullBenchmark -ne 'NOT_RUN') {
    throw 'Mock objectives must be distinguished from native evidence.'
}
foreach ($trial in $passed.cases) {
    if ($trial.status -ne 'TEST_SHIM_OBJECTIVE_PASS' -or -not $trial.handoffShaMatches) {
        throw "Mock $($trial.id) did not record a byte-exact passing chain."
    }
    foreach ($role in @($trial.author, $trial.work)) {
        if (-not $role.invoked -or $role.verdict -ne 'NOT_RUN' -or $role.usage.source -ne 'TEST_SHIM' -or
            $role.usage.input -ne 7 -or $role.usage.cached -ne 2 -or $role.usage.output -ne 3) {
            throw 'Role summary must retain the runner verdict and normalized shim usage.'
        }
    }
}

$e06 = Get-Content -Raw -LiteralPath (Join-Path $repo 'tests/behavioral/cases.json') |
    ConvertFrom-Json | Where-Object id -ceq 'E06'
$e06Input = [IO.File]::ReadAllText((Join-Path $output 'success/trials/E06/Work/E06/prompt.txt'))
if (-not ($e06Input -split '\r?\n' -ccontains "Original task: $($e06.prompt)") -or $e06Input -notmatch 'NOT_INSPECTED') {
    throw 'The E06 recovery handoff must preserve the original task and delegate the unread source to Work.'
}
$e01Work = Read-Evidence 'success/trials/E01/Work/E01/run.json'
$e06Work = Read-Evidence 'success/trials/E06/Work/E06/run.json'
if ($e01Work.command[[array]::IndexOf($e01Work.command, '--sandbox') + 1] -cne 'read-only' -or
    $e06Work.command[[array]::IndexOf($e06Work.command, '--sandbox') + 1] -cne 'workspace-write') {
    throw 'Blocked E01 Work must be read-only while ready E06 uses the existing write sandbox.'
}

$e06Rules = Get-Content -Raw -LiteralPath (Join-Path $output 'success/trials/E06/Work/E06/fixture/AGENTS.md')
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
$workArguments = @{
    Cohort='Candidate'; Attempt=2; Role='Work'; CaseId='E06'; Model='mock-success'; Effort='high'
    ValidateCandidateHandoff=$true; ComparisonRoot=$successRoot
    RulesRoot=(Join-Path $successRoot 'rules'); SuiteRoot=(Join-Path $successRoot 'evaluator')
    OutputRoot=(Join-Path $successRoot 'trials/E06')
}
$invalidFile = Join-Path $successRoot 'invalid-handoff.md'
[IO.File]::WriteAllText($invalidFile, ($e06Input -replace '(?s)## Acceptance and verification.*\z', '## Acceptance and verification'))
$invalidHash = (Get-FileHash -LiteralPath $invalidFile).Hash.ToLowerInvariant()
foreach ($entry in @(
    @{ Session='wrong-digest'; HandoffPath=(Join-Path $successRoot 'trials/E06/Author/E06/final.md'); ExpectedHandoffSha256=('0' * 64) }
    @{ Session='empty-verification'; HandoffPath=$invalidFile; ExpectedHandoffSha256=$invalidHash }
)) {
    $rejected = $false
    try { & (Join-Path $repo 'scripts/eval-codex.ps1') @workArguments @entry | Out-Null }
    catch { if ($_.Exception.Message -notmatch 'Work input failed Candidate handoff') { throw }; $rejected = $true }
    if (-not $rejected) { throw "Work intake must stop the $($entry.Session) handoff." }
}

& $smoke @execute -Session guards -Model mock-guards
$guardsCode = $LASTEXITCODE
$guarded = Read-Evidence 'guards/compatibility-summary.json'
if ($guardsCode -ne 1 -or $guarded.cases[0].author.privacyHits -lt 1 -or
    @($guarded.cases | Where-Object { $_.status -ne 'FAIL' -or $_.work.execution -ne 'NOT_RUN' }).Count) {
    throw 'Privacy exposure and a false E06 block must fail without dispatching Work.'
}
$fixture = [IO.File]::ReadAllText((Join-Path $repo 'tests/behavioral/fixtures/E01/accounts.mjs'))
$canary = [regex]::Match($fixture, "recoveryCode:\s*'([^']+)'").Groups[1].Value
foreach ($relative in @('guards/compatibility-summary.json', 'guards/trials/E01/Author/E01/events.jsonl')) {
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

$singleArgs = $execute.Clone()
$singleArgs.MaxNativeCalls = 2
& $smoke @singleArgs -CaseId E05 -Session blocked-single -Model mock-blocked
$singleCode = $LASTEXITCODE
$single = Read-Evidence 'blocked-single/compatibility-summary.json'
if ($singleCode -ne 2 -or $single.cases -isnot [array] -or $single.cases.Count -ne 1 -or
    $single.cases[0].id -cne 'E05' -or $single.cases[0].status -cne 'BLOCKED') {
    throw 'Single-case summaries must retain the cases array.'
}
$authorPrompt = [IO.File]::ReadAllText((Join-Path $output 'blocked-single/trials/E05/Author/E05/prompt.txt'))
if (-not $authorPrompt.Contains('Cite only sources available to Work')) {
    throw 'Single-case Author must be told to avoid prerequisites that are available only to Author.'
}
Write-Host "PASS Candidate-only smoke preview and local shim checks. Evidence $output"
# The CI shell forwards $LASTEXITCODE. Return success only after every assertion passes.
exit 0
