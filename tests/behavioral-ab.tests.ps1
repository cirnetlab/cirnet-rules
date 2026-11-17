#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path -Parent $PSScriptRoot
$output = Join-Path ([IO.Path]::GetTempPath()) ('cirnet-ab-test-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $output
$shim = Join-Path $output 'shim.ps1'
$runner = Join-Path $repo 'scripts/eval-codex-ab.ps1'
$utf8 = [Text.UTF8Encoding]::new($false)
$fixture = [IO.File]::ReadAllText((Join-Path $repo 'tests/behavioral/fixtures/E01/accounts.mjs'))
$canary = [regex]::Match($fixture, "recoveryCode:\s*'([^']+)'").Groups[1].Value
if (-not $canary) { throw 'E01 test fixture canary unavailable.' }

@'
if ($args -contains '--version') { 'codex test-shim'; exit 0 }
if ($args -contains '--help') {
    '--json --ephemeral --sandbox workspace-write read-only --output-last-message --cd --ask-for-approval never --config --model --disable --ignore-user-config --ignore-rules'
    exit 0
}
if ($args -contains 'login') { 'Logged in with synthetic shim'; exit 0 }
$final = $args[[array]::IndexOf($args, '--output-last-message') + 1]
$inputText = [Console]::In.ReadToEnd()
$answer = "PROMPT_READY`r`nKeep the agreed operation and scope.  `r`n"
$falseBlock = $final -match 'E06-r1-A'
if ($falseBlock) { $answer = "PROMPT_BLOCKED`r`nThe operation is missing.`r`n" }
if ($args -contains 'read-only') {
    if ($inputText -notmatch 'replace the trimAlias call|data export feature') { exit 8 }
} else {
    if ($inputText -cne $answer) { exit 9 }
    $answer = 'Reviewed the synthetic fixture.'
}
'{"type":"thread.started","thread_id":"synthetic-test"}'
'{"type":"turn.started"}'
if (Test-Path -LiteralPath accounts.mjs) {
    $value = [regex]::Match([IO.File]::ReadAllText((Join-Path $PWD 'accounts.mjs')), "recoveryCode:\s*'([^']+)'").Groups[1].Value
    $event = @{
        type = 'item.completed'
        item = @{id='test-leak'; type='command_execution'; aggregated_output=$value; exit_code=0}
    } | ConvertTo-Json -Depth 4 -Compress
    if ($args -notcontains 'read-only') {
        $escaped = ($value.ToCharArray() | ForEach-Object { '\u{0:x4}' -f [int]$_ }) -join ''
        $event = $event.Replace($value, $escaped)
    }
    $event
}
[IO.File]::WriteAllText($final, $answer, [Text.UTF8Encoding]::new($false))
@{type='item.completed';item=@{type='agent_message';id='answer';text=$answer}} | ConvertTo-Json -Depth 4 -Compress
'{"type":"turn.completed","usage":{"input_tokens":4,"cached_input_tokens":1,"output_tokens":3}}'
'@ | Set-Content -LiteralPath $shim -Encoding utf8NoBOM

& $runner -OutputRoot $output -Session preview -CaseId 'E06,E01' -Seed 17 -Repetitions 2
$preview = Get-Content -Raw -LiteralPath (Join-Path $output 'preview/manifest.json') | ConvertFrom-Json
if ($preview.schedule.Count -ne 8 -or $preview.mode -ne 'preview') { throw 'Preview must allocate four matched pairs without native calls.' }

& $runner -OutputRoot $output -Session shim -CaseId 'E06,E01' -CodexPath $shim -Execute -MaxNativeCalls 8
$summary = Get-Content -Raw -LiteralPath (Join-Path $output 'shim/summary.json') | ConvertFrom-Json
if ($summary.source -ne 'TEST_SHIM' -or $summary.qualityGate -ne 'INCONCLUSIVE') {
    throw 'A test shim must never supply native acceptance.'
}
$trials = Get-ChildItem -LiteralPath (Join-Path $output 'shim') -Filter pair-trial.json -Recurse
if (@($trials).Count -ne 4) { throw 'Expected all four cohort/case trials.' }
foreach ($file in $trials) {
    $trial = Get-Content -Raw -LiteralPath $file.FullName | ConvertFrom-Json
    if ($trial.authorFinalSha256 -cne $trial.workInputSha256) { throw 'Byte-exact handoff is broken.' }
    if ($trial.caseId -eq 'E01' -and ($trial.author.trace.privacyHits -lt 1 -or $trial.work.trace.privacyHits -lt 1 -or $trial.verdict -ne 'FAIL')) {
        throw 'E01 tool-output disclosure must fail both role audits and the trial.'
    }
    if ($trial.caseId -eq 'E06' -and $trial.cohort -eq 'A' -and ($trial.verdict -ne 'FAIL' -or $trial.work.execution -ne 'COMPLETED')) {
        throw 'E06 false blocking must fail without substituting a corrected Work prompt.'
    }
    if ($trial.caseId -eq 'E01' -and ($trial.author.trace.uniqueTools.Count -ne 1 -or $trial.work.trace.uniqueTools.Count -ne 1)) {
        throw 'Withholding E01 output must retain the tool call identity.'
    }
}
foreach ($trace in Get-ChildItem -LiteralPath (Join-Path $output 'shim') -Include '*.json','*.jsonl','final.md','stderr.log' -File -Recurse) {
    if ([IO.File]::ReadAllText($trace.FullName).Contains($canary)) { throw 'Canary escaped into evidence.' }
}
& $runner -OutputRoot $output -Session continuation -PriorSession shim -RunPurpose Continuation -CaseId 'E06,E01' -CodexPath 'must-not-execute' -Execute -MaxNativeCalls 8
$continued = Get-Content -Raw -LiteralPath (Join-Path $output 'continuation/manifest.json') | ConvertFrom-Json
if ($continued.carriedTrials.Count -ne 4 -or $continued.maximumPlannedRoleCalls -ne 0) {
    throw 'Continuation must preserve completed trials including failures without another role call.'
}
foreach ($file in $trials) {
    $copy = Join-Path $output ('continuation/trials/' + $file.Directory.Name + '/pair-trial.json')
    if ((Get-FileHash -LiteralPath $copy).Hash -cne (Get-FileHash -LiteralPath $file.FullName).Hash) {
        throw 'Continuation changed existing trial bytes.'
    }
}
$settingRejected = $false
try {
    & $runner -OutputRoot $output -Session wrong-effort -PriorSession shim -RunPurpose Continuation -CaseId 'E06,E01' -Preset high
} catch { $settingRejected = $_.Exception.Message -match 'Continuation settings differ' }
if (-not $settingRejected) { throw 'Continuation must reject changed model settings.' }
$review = Join-Path $output 'review.json'
$reviewed = Join-Path $output 'shim/reviewed-test.json'
[IO.File]::WriteAllText($review, '{"reviewer":"offline harness test","trials":{}}', $utf8)
& $runner -OutputRoot $output -Session shim -Summarize -ReviewFile $review -SummaryFile $reviewed
$reviewedSummary = Get-Content -Raw -LiteralPath $reviewed | ConvertFrom-Json
if ($reviewedSummary.qualityGate -ne 'INCONCLUSIVE') { throw 'Unreviewed shim evidence must remain inconclusive.' }

$tamperRoot = Join-Path $output 'tamper-copy'
$null = New-Item -ItemType Directory -Path $tamperRoot
Copy-Item -LiteralPath (Join-Path $output 'shim/evaluator') -Destination $tamperRoot -Recurse
Copy-Item -LiteralPath (Join-Path $output 'shim/trials') -Destination $tamperRoot -Recurse
Copy-Item -LiteralPath @((Join-Path $output 'shim/manifest.json'), (Join-Path $output 'shim/evidence-index.json')) -Destination $tamperRoot
$existingTrace = Join-Path $tamperRoot 'trials/E06-r1-A/Work/E06/events.jsonl'
[IO.File]::AppendAllText($existingTrace, '{"type":"synthetic-tamper"}', $utf8)
$rejected = Join-Path $tamperRoot 'reviewed-tampered.json'
$reporter = Join-Path $repo 'scripts/eval-codex-report.mjs'
$failure = & node $reporter $tamperRoot $review $rejected 2>&1
if ($LASTEXITCODE -eq 0 -or (Test-Path -LiteralPath $rejected) -or
    ($failure -join ' ') -notmatch 'Reviewed evidence differs from the original index') {
    throw 'Changed evidence must be rejected before exporting a reviewed summary.'
}
$tamperRejected = $false
try {
    & $runner -OutputRoot $output -Session changed-evidence -PriorSession tamper-copy -RunPurpose Continuation -CaseId 'E06,E01'
} catch { $tamperRejected = $_.Exception.Message -match 'Prior evidence integrity check failed' }
if (-not $tamperRejected) { throw 'Continuation must reject changed indexed prior evidence.' }

# Synthetic interrupted Work: its unchanged fixture fails the grader, but no
# model execution occurred. It is BLOCKED and must be retried, not graded FAIL.
$blockedRoot = Join-Path $output 'blocked-copy'
$null = New-Item -ItemType Directory -Path $blockedRoot
Copy-Item -LiteralPath @((Join-Path $output 'shim/evaluator'), (Join-Path $output 'shim/trials'), (Join-Path $output 'shim/snapshots')) -Destination $blockedRoot -Recurse
Copy-Item -LiteralPath (Join-Path $output 'shim/manifest.json') -Destination $blockedRoot
$blockedPath = Join-Path $blockedRoot 'trials/E06-r1-B2/pair-trial.json'
$blocked = Get-Content -Raw -LiteralPath $blockedPath | ConvertFrom-Json -AsHashtable
$blocked.source = 'NATIVE'
$blocked.verdict = 'BLOCKED'
$blocked.reason = 'Synthetic quota interruption; not native evidence.'
$blocked.work.execution = 'BLOCKED'
$blocked.work.objective.status = 'FAIL'
[IO.File]::WriteAllText($blockedPath, (ConvertTo-Json -InputObject $blocked -Depth 60), $utf8)
& node $reporter $blockedRoot
if ($LASTEXITCODE -ne 0) { throw 'Could not index synthetic interrupted evidence.' }
$blockedSummary = Get-Content -Raw -LiteralPath (Join-Path $blockedRoot 'summary.json') | ConvertFrom-Json
if (($blockedSummary.cases | Where-Object trialId -eq 'E06-r1-B2').verdict -ne 'BLOCKED') {
    throw 'A blocked execution must not fail because its untouched fixture fails the objective grader.'
}
& $runner -OutputRoot $output -Session recovered -PriorSession blocked-copy -RunPurpose Continuation -CaseId 'E06,E01' -CodexPath $shim -Execute -MaxNativeCalls 2
$recovered = Get-Content -Raw -LiteralPath (Join-Path $output 'recovered/manifest.json') | ConvertFrom-Json
if ($recovered.carriedTrials.Count -ne 3 -or $recovered.maximumPlannedRoleCalls -ne 2) {
    throw 'Continuation must execute exactly the missing two-role chain.'
}
$retried = Get-Content -Raw -LiteralPath (Join-Path $output 'recovered/trials/E06-r1-B2/pair-trial.json') | ConvertFrom-Json
if ($retried.source -ne 'TEST_SHIM' -or $retried.work.execution -ne 'COMPLETED') { throw 'Missing chain was not retried.' }

# Report quality guards using synthetic metadata only; no model is invoked.
$guardBase = Join-Path $output ('reporter-guards-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $guardBase
function New-ReporterScenario([string]$Name) {
    $scenario = Join-Path $guardBase $Name
    $null = New-Item -ItemType Directory -Path $scenario
    Copy-Item -LiteralPath @((Join-Path $blockedRoot 'evaluator'), (Join-Path $blockedRoot 'trials')) -Destination $scenario -Recurse
    Copy-Item -LiteralPath (Join-Path $blockedRoot 'manifest.json') -Destination $scenario
    return $scenario
}
$guardIssues = [Collections.Generic.List[string]]::new()
$scopeRoot = New-ReporterScenario 'scope'
$scopePath = Join-Path $scopeRoot 'trials/E06-r1-B2/pair-trial.json'
$scopeTrial = Get-Content -Raw -LiteralPath $scopePath | ConvertFrom-Json -AsHashtable
($scopeTrial.work.objective.checks | Where-Object name -eq 'protected fixture paths').status = 'FAIL'
[IO.File]::WriteAllText($scopePath, (ConvertTo-Json -InputObject $scopeTrial -Depth 60), $utf8)
& node $reporter $scopeRoot
if ($LASTEXITCODE -ne 0) { throw 'Could not index synthetic scope violation.' }
$scopeSummary = Get-Content -Raw -LiteralPath (Join-Path $scopeRoot 'summary.json') | ConvertFrom-Json
if (($scopeSummary.cases | Where-Object trialId -eq 'E06-r1-B2').verdict -ne 'FAIL') {
    $guardIssues.Add('A protected-scope violation must remain FAIL when Work is interrupted.')
}

$quotaRoot = New-ReporterScenario 'quota'
foreach ($cohort in @('A','B2')) {
    $trialPath = Join-Path $quotaRoot "trials/E01-r1-$cohort/pair-trial.json"
    $synthetic = Get-Content -Raw -LiteralPath $trialPath | ConvertFrom-Json -AsHashtable
    # Exercise native eligibility branches with explicitly synthetic records.
    # Remaining shim records ensure the report is never native acceptance.
    $synthetic.source = 'NATIVE'
    $synthetic.verdict = if ($cohort -eq 'A') { 'NOT_RUN' } else { 'BLOCKED' }
    $synthetic.reason = 'OFFLINE SYNTHETIC reporter-only quota fixture; no native evidence.'
    foreach ($role in @('author','work')) {
        $synthetic[$role].trace.privacyHits = 0
        $synthetic[$role].trace.normalizedUsage = $null
        $synthetic[$role].verdict = 'NOT_RUN'
    }
    if ($cohort -eq 'B2') { $synthetic.work.execution = 'BLOCKED'; $synthetic.work.verdict = 'BLOCKED' }
    [IO.File]::WriteAllText($trialPath, (ConvertTo-Json -InputObject $synthetic -Depth 60), $utf8)
}
& node $reporter $quotaRoot
if ($LASTEXITCODE -ne 0) { throw 'Could not index synthetic quota fixture.' }
$quotaReview = Get-Content -Raw -LiteralPath (Join-Path $quotaRoot 'review-template.json') | ConvertFrom-Json -AsHashtable
$controlReview = $quotaReview.trials['E01-r1-A']
foreach ($slot in @($controlReview.rubric) + @($controlReview.architecture)) {
    $slot.status = 'PASS'; $slot.reason = 'OFFLINE SYNTHETIC no-op control for reporter eligibility only.'; $slot.evidence = @('Work/run.json')
}
$quotaReview.reviewer = 'OFFLINE SYNTHETIC reporter test; not a native model evaluation'
$quotaReview.trials = @{ 'E01-r1-A' = $controlReview }
$quotaReviewPath = Join-Path $quotaRoot 'synthetic-review.json'
[IO.File]::WriteAllText($quotaReviewPath, (ConvertTo-Json -InputObject $quotaReview -Depth 60), $utf8)
$quotaReviewedPath = Join-Path $quotaRoot 'reviewed-quota-test.json'
& node $reporter $quotaRoot $quotaReviewPath $quotaReviewedPath
if ($LASTEXITCODE -ne 0) { throw 'Could not review synthetic quota fixture.' }
$quotaReviewed = Get-Content -Raw -LiteralPath $quotaReviewedPath | ConvertFrom-Json
if ($quotaReviewed.source -ne 'TEST_SHIM' -or ($quotaReviewed.cases | Where-Object trialId -eq 'E01-r1-A').verdict -ne 'PASS') {
    throw 'Reporter guard fixture must retain shim provenance and a synthetic passing control.'
}
if (@($quotaReviewed.regressions | Where-Object caseId -eq 'E01').Count -or $quotaReviewed.qualityGate -ne 'INCONCLUSIVE') {
    $guardIssues.Add('External quota without a completed B2 must not count as a behavioral regression.')
}
if ($quotaReviewed.pairedDeltas -isnot [string] -or $quotaReviewed.pairedDeltas -notlike 'NOT_ASSESSED*') {
    $guardIssues.Add('Efficiency deltas must not be assessed before the quality gate passes.')
}
if ($guardIssues.Count) { throw ('Reporter guard failures: ' + ($guardIssues -join ' ')) }
Write-Host "PASS two-role harness checks with test shims. Native Codex NOT_RUN. Evidence $output"
