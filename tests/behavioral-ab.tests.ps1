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
Write-Host "PASS two-role harness checks with test shims. Native Codex NOT_RUN. Evidence $output"
