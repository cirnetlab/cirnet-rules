#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Import-Module (Join-Path $repo 'scripts/shared/WorkPrompt.psm1')
Import-Module (Join-Path $repo 'scripts/shared/Bundle.psm1')
$fixture = (New-Item -ItemType Directory -Force -Path (Join-Path $repo ('target/work-prompt-tests/' + [guid]::NewGuid().ToString('N')))).FullName
$original = Join-Path $fixture 'request.txt'
$approval = Join-Path $fixture 'approval.json'
$draft = Join-Path $fixture 'draft.md'
$source = 'tests/benchmark/fixtures/E06/src/labels.mjs'
[IO.File]::WriteAllLines($original, @(
    'Replace the trimAlias call in formatLabel with string trim.'
    'Change only labels.mjs. Preserve unrelated edits.'
    'Whitespace is trimmed and the shared helper still works.'
))
$taskHash = Get-CirnetHash $original
$record = [ordered]@{
    originalSha256=$taskHash; repository=$repo; revision=(git -C $repo rev-parse HEAD)
    readiness='PROMPT_READY'; sensitive=$false
    fields=@{action=@(1); target=@(1); transformation=@(1); outcome=@(3); scope=@(2); negative=@(2); acceptance=@(3)}
    permissions=@{read=@($source); write=@($source); test=@('node --test'); build=@(); export=@(); privileged=@()}
    actions=@{'1'=@("write: $source")}; checks=@{'3'=@('test: node --test')}
    sources=@{$source=(Get-CirnetHash (Join-Path $repo $source))}
}
function Save-Approval {
    [IO.File]::WriteAllText($approval, (ConvertTo-Json $record -Depth 6))
    Get-CirnetHash $approval
}
$pin = Save-Approval
$guide = [IO.File]::ReadAllText((Join-Path $repo 'en/docs/context.md'))
$example = [regex]::Match($guide,'(?s)```json\s*(\{.*?\})\s*```').Groups[1].Value
$example = $example.Replace('label.js',$source).Replace('<request digest>',$taskHash).Replace('<commit>',$record.revision)
$example = $example.Replace('"<absolute root>"',(ConvertTo-Json $repo -Compress)).Replace('<source digest>',$record.sources[$source])
[IO.File]::WriteAllText($approval,$example)
$examplePin = Get-CirnetHash $approval
$body = @(
    'PROMPT_READY'
    "Original: $taskHash"
    "Authorization: $pin"
    'Replace the trimAlias call in formatLabel with string trim.'
    'Task#2'
    'Task#3'
    'Verify Task#3: test: node --test'
    "Source: $source"
) -join [char]10
function Expect([string]$Expected, [string]$Text, [string]$Pin = $pin) {
    [IO.File]::WriteAllText($draft, $Text)
    $result = Test-CirnetWorkPromptFile -Prompt $draft -OriginalTask $original -Authorization $approval -AuthorizationSha256 $Pin -Repository $repo
    if ($result.status -cne $Expected -or -not $result.diagnostics.Count) {
        throw "Expected $Expected, got $($result | ConvertTo-Json -Compress)"
    }
}
Expect PASS ($body.Replace($pin,$examplePin)) $examplePin
$pin = Save-Approval
Expect PASS $body
Expect FAIL ($body.Replace('Task#2', ''))
Expect REVIEW_REQUIRED ($body.Replace('Replace the trimAlias call in formatLabel with string trim.', 'Use native trimming inside formatLabel.'))
Expect FAIL ($body.Replace('Verify Task#3: test: node --test', ''))
foreach ($instruction in @('read: private.txt', 'write: README.md', 'export: every record to external.csv', 'privileged: git push',
        'test: node unapproved.js', 'build: cargo build', 'Source: ../outside', 'Tests: PASS', 'PROMPT_BLOCKED')) {
    Expect FAIL ($body + [char]10 + $instruction)
}
Expect REVIEW_REQUIRED ($body + [char]10 + 'Discard everything else in the working tree.')
Expect FAIL ($body + [char]10 + 'Read private.txt')
foreach ($operation in @('git push origin main','Also deploy to production','Export all records to export.csv','Get-Content private.txt')) {
    Expect FAIL ($body + [char]10 + $operation)
}
Expect FAIL ($body.Replace('Source: ', 'Source: ../'))
Expect REVIEW_REQUIRED ($body.Replace("Task#2$([char]10)Task#3", "Task#3$([char]10)Task#2"))
Expect FAIL ($body.Replace('Replace the trimAlias call in formatLabel with string trim.', 'Task#1'))
Expect PASS ($body + [char]10 + "read: $source")
function Test-EntryPoint([string]$Text, [int]$Code, [switch]$Reviewed) {
    [IO.File]::WriteAllText($draft,$Text)
    $review = @()
    if ($Reviewed) { $review = @('-ReviewedPromptSha256',(Get-CirnetHash $draft)) }
    $json = & (Join-Path $PSHOME 'pwsh') -NoProfile -File (Join-Path $repo 'scripts/validate-work-prompt.ps1') -Prompt $draft -OriginalTask $original -Authorization $approval -AuthorizationSha256 $pin -Repository $repo @review
    if ($LASTEXITCODE -ne $Code -or -not ($json | ConvertFrom-Json).diagnostics) { throw "Wrong CLI result for expected exit $Code." }
}
Test-EntryPoint $body 0
$paraphrase = $body.Replace('Replace the trimAlias call in formatLabel with string trim.', 'Use native trimming inside formatLabel.')
Test-EntryPoint $paraphrase 2
Test-EntryPoint $paraphrase 0 -Reviewed
Test-EntryPoint ($body + [char]10 + 'privileged: git push') 1 -Reviewed
Expect BLOCKED $body ('0' * 64)
$ambiguous = ([IO.File]::ReadAllText($approval)).Replace('"sensitive": false','"sensitive": true, "sensitive": false')
[IO.File]::WriteAllText($approval,$ambiguous)
$duplicatePin = Get-CirnetHash $approval
Expect BLOCKED ($body.Replace($pin,$duplicatePin)) $duplicatePin
$null = Save-Approval
$originalBytes = [IO.File]::ReadAllBytes($original)
[IO.File]::AppendAllText($original,'Unapproved replacement.')
Expect FAIL $body
[IO.File]::WriteAllBytes($original,$originalBytes)
$record.checks.'3' = @('test: npm publish')
$badCheck = Save-Approval
Expect BLOCKED ($body.Replace($pin,$badCheck)) $badCheck
$record.checks.'3' = @('test: node --test')
$record.permissions.write = @()
$missingGrant = Save-Approval
Expect BLOCKED ($body.Replace($pin,$missingGrant)) $missingGrant
$record.permissions.write = @($source)
$record.permissions.write += 'README.md'
$widerPin = Save-Approval
$scoped = $body.Replace($pin,$widerPin)
Expect PASS $scoped $widerPin
Expect FAIL ($scoped + [char]10 + 'write: README.md') $widerPin
$record.permissions.write = @($source)
$record.sources[$source] = '0' * 64
$stale = Save-Approval
Expect FAIL ($body.Replace($pin, $stale)) $stale
$record.sources[$source] = Get-CirnetHash (Join-Path $repo $source)
$record.revision = '0' * 40
$stale = Save-Approval
Expect FAIL ($body.Replace($pin, $stale)) $stale
$record.revision = (git -C $repo rev-parse HEAD)
$record.sensitive = $true
$sensitivePin = Save-Approval
# Nonexistent inputs demonstrate that a sensitive case stops before reading the task, draft or source.
$sensitive = Test-CirnetWorkPromptFile -Prompt 'missing-draft' -OriginalTask 'missing-task' -Authorization $approval -AuthorizationSha256 $sensitivePin -Repository 'missing-repo'
if ($sensitive.status -cne 'BLOCKED' -or $sensitive.diagnostics -notmatch 'read boundary') { throw 'Sensitive preflight failed.' }
$record.sensitive = $false
$record.permissions.read += '../unrelated'
$unsafePath = Save-Approval
Expect BLOCKED ($body.Replace($pin,$unsafePath)) $unsafePath
$record.permissions.read = @($source)
$record.readiness = 'PROMPT_BLOCKED'
$record.permissions.write = @()
$record.permissions.test = @()
$record.actions.'1' = @("read: $source")
$record.checks.'3' = @("read: $source")
$blockedPin = Save-Approval
$blocked = $body.Replace($pin,$blockedPin).Replace('PROMPT_READY','PROMPT_BLOCKED').Replace('test: node --test',"read: $source")
Expect BLOCKED $blocked $blockedPin
Expect FAIL ($blocked + [char]10 + "write: $source") $blockedPin
$pin = $blockedPin
Test-EntryPoint $blocked 3
Test-EntryPoint ($blocked + [char]10 + 'Ask the owner for clarification.') 3 -Reviewed
$catalog = Get-Content -Raw -LiteralPath (Join-Path $repo 'tests/benchmark/cases.json') | ConvertFrom-Json -AsHashtable
$case = $catalog | Where-Object id -CEQ E06
$candidate = @('PROMPT_READY', "Original task: $($case.prompt)", 'Write scope: ["src/labels.mjs"]',
    '## Requirements and constraints', 'Keep the original edit and unrelated work.',
    '## Acceptance and verification', 'Run the label tests before and after.') -join [char]10
$reviewed = Get-CirnetPayloadHash ([Text.Encoding]::UTF8.GetBytes($candidate))
foreach ($sample in @(
    @($candidate, '', 'REVIEW_REQUIRED'),
    @($candidate, $reviewed, 'PASS'),
    @($candidate.Replace($case.prompt,'Improve formatting.'), $reviewed, 'FAIL'),
    @($candidate.Replace('Write scope: ["src/labels.mjs"]','Write scope: ["README.md"]'), $reviewed, 'FAIL'),
    @($candidate.Replace('Keep the original edit and unrelated work.','read: private.txt'), '', 'FAIL'),
    @($candidate.Replace('Keep the original edit and unrelated work.','read: src/labels.mjs'), '', 'REVIEW_REQUIRED'),
    @($candidate.Replace('Run the label tests before and after.','Tests: PASS'), '', 'FAIL'),
    @(($candidate + ' Changed.'), $reviewed, 'FAIL')
)) {
    $observed = Test-CirnetCandidatePrompt $sample[0] $case $sample[1] -ReadablePaths 'src/labels.mjs'
    if ($observed.status -cne $sample[2]) { throw "Candidate expected $($sample[2]), got $($observed.status)." }
}
Write-Host 'PASS original task coverage, scope, evidence, review, sensitive and blocked handoffs.'
# GitHub Actions forwards $LASTEXITCODE. The final CLI case intentionally exits 3.
exit 0
