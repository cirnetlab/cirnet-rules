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
Expect FAIL ($body.Replace('Replace the trimAlias call in formatLabel with string trim.', 'Use native trimming inside formatLabel.'))
Expect FAIL ($body.Replace('Verify Task#3: test: node --test', ''))
foreach ($instruction in @('read: private.txt', 'write: README.md', 'export: every record to external.csv', 'privileged: git push',
        'test: node unapproved.js', 'build: cargo build', 'Source: ../outside', 'Tests: PASS', 'PROMPT_BLOCKED')) {
    Expect FAIL ($body + [char]10 + $instruction)
}
Expect REVIEW_REQUIRED ($body + [char]10 + 'Discard everything else in the working tree.')
Expect FAIL ($body + [char]10 + 'Read private.txt')
foreach ($operation in @('git push origin main','Also deploy to production','Export all records to export.csv','Get-Content private.txt','delete: README.md','Remove-Item README.md')) {
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
$lostObjective = $body.Replace('Replace the trimAlias call in formatLabel with string trim.', 'Improve formatting.')
Test-EntryPoint $lostObjective 1
Test-EntryPoint $lostObjective 1 -Reviewed
Test-EntryPoint ($body.Replace('Task#2', 'Use local judgment about unrelated files.')) 1 -Reviewed
Test-EntryPoint ($body.Replace('Task#3', "~~~$([char]10)Task#3$([char]10)~~~")) 1 -Reviewed
Test-EntryPoint ($body.Replace('Task#2', "<!--$([char]10)Task#2$([char]10)-->")) 1 -Reviewed
Test-EntryPoint ($body + [char]10 + 'Use native trimming inside formatLabel.') 2
Test-EntryPoint ($body + [char]10 + 'Use native trimming inside formatLabel.') 0 -Reviewed
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
$record.permissions.test += 'npm publish'
$misclassified = Save-Approval
Expect BLOCKED ($body.Replace($pin,$misclassified).Replace('test: node --test','test: npm publish')) $misclassified
$record.permissions.test = @('node --test')
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
$record.preventiveScope = $true
$preventivePin = Save-Approval
$preventive = Test-CirnetWorkPromptFile -Prompt 'missing-draft' -OriginalTask 'missing-task' -Authorization $approval -AuthorizationSha256 $preventivePin -Repository 'missing-repo'
if ($preventive.status -cne 'BLOCKED' -or $preventive.diagnostics -notmatch 'preventive execution boundary') { throw 'Trusted preventive scope was skipped.' }
$record.Remove('preventiveScope')
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
    @($candidate.Replace('Keep the original edit and unrelated work.','<!-- hidden requirement -->'), '', 'FAIL'),
    @(($candidate.Replace('## Acceptance and verification', "~~~`n## Acceptance and verification") + "`n~~~"), '', 'FAIL'),
    @($candidate.Replace('Run the label tests before and after.','Tests: PASS'), '', 'FAIL'),
    @(($candidate + ' Changed.'), $reviewed, 'FAIL')
)) {
    $observed = Test-CirnetCandidatePrompt $sample[0] $case $sample[1] -ReadablePaths 'src/labels.mjs'
    if ($observed.status -cne $sample[2]) { throw "Candidate expected $($sample[2]), got $($observed.status)." }
}

$auditRepo = Join-Path $fixture 'owner'
[void][IO.Directory]::CreateDirectory($auditRepo)
function Invoke-OwnerGit([string[]]$GitArgs) {
    $result = & git -C $auditRepo -c core.autocrlf=false @GitArgs
    if ($LASTEXITCODE -ne 0) { throw 'Could not prepare scope fixture Git state.' }
    $result
}
$null = Invoke-OwnerGit @('init','--quiet','--template=','-b','main')
foreach ($item in @(@('.gitignore','ignored.txt'),@('labels.mjs','export const label = 1;'),@('owner.md','Owner rules'))) {
    [IO.File]::WriteAllText((Join-Path $auditRepo $item[0]), $item[1] + [char]10)
}
$null = Invoke-OwnerGit @('add','--all')
$null = Invoke-OwnerGit @('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','--quiet','--no-gpg-sign','-m','fixture')
$ignored = Join-Path $auditRepo 'ignored.txt'
$untracked = Join-Path $auditRepo 'scratch.txt'
$label = Join-Path $auditRepo 'labels.mjs'
$rules = Join-Path $auditRepo 'owner.md'
[IO.File]::WriteAllText($ignored,'ignored contents')
[IO.File]::WriteAllText($untracked,'user notes')
[IO.File]::AppendAllText($rules,'existing unrelated edit')
$record.repository = $auditRepo
$record.revision = Invoke-OwnerGit @('rev-parse','HEAD')
$record.readiness = 'PROMPT_READY'
$record.permissions = @{ read=@('labels.mjs','owner.md'); write=@('labels.mjs','new.mjs'); test=@('node --test'); build=@(); export=@(); privileged=@() }
$record.sources = @{ 'labels.mjs'=(Get-CirnetHash $label); 'owner.md'=(Get-CirnetHash $rules) }
$record.actions.'1' = @('modify: labels.mjs')
$record.checks.'3' = @('test: node --test')
$pin = Save-Approval
$auditDraft = $body.Replace(($body -split '\n')[2], "Authorization: $pin").Replace("Source: $source","Source: labels.mjs`nSource: owner.md")
[IO.File]::WriteAllText($draft,$auditDraft)
$intake = @{ Prompt=$draft; OriginalTask=$original; Authorization=$approval; AuthorizationSha256=$pin; Repository=$auditRepo }
if ((Test-CirnetWorkPromptFile @intake -CaptureScope).status -cne 'BLOCKED') {
    throw 'A CLI switch alone authorized a worktree-wide audit.'
}
$record.scopeAudit = $true
$intake.AuthorizationSha256 = Save-Approval
[IO.File]::WriteAllText($draft,$auditDraft.Replace($pin,$intake.AuthorizationSha256))
$oldIndex = Get-CirnetHash (Join-Path $auditRepo '.git/index')
$capture = Test-CirnetWorkPromptFile @intake -CaptureScope
if ($capture.status -cne 'PASS' -or -not $capture.scopeSnapshot.git.files['scratch.txt'] -or
    -not $capture.scopeSnapshot.git.files['ignored.txt']) { throw 'Scope capture omitted current authorization or local files.' }
$snapshot = Join-Path $fixture 'scope.json'
[IO.File]::WriteAllText($snapshot,(ConvertTo-Json $capture -Depth 24 -Compress))
$audit = @{ ScopeSnapshot=$snapshot; ScopeSnapshotSha256=(Get-CirnetHash $snapshot) }
[IO.File]::AppendAllText($label,"`n// authorized")
if ((Test-CirnetWorkPromptFile @intake @audit).status -cne 'PASS') { throw 'A scoped modification was not reconciled.' }
foreach ($path in @($ignored,$untracked,$rules)) {
    $before = [IO.File]::ReadAllBytes($path)
    [IO.File]::AppendAllText($path,'out of scope')
    if ((Test-CirnetWorkPromptFile @intake @audit).status -cne 'FAIL') { throw 'An unrelated file change passed scope audit.' }
    [IO.File]::WriteAllBytes($path,$before)
}
$outside = Join-Path $auditRepo 'new.mjs'
[IO.File]::WriteAllText($outside,'broader permission is not a task mapping')
if ((Test-CirnetWorkPromptFile @intake @audit).status -cne 'FAIL') { throw 'A broader write grant expanded task scope.' }
[IO.File]::Delete($outside)
$before = [IO.File]::ReadAllBytes($label)
[IO.File]::Delete($label)
if ((Test-CirnetWorkPromptFile @intake @audit).status -cne 'FAIL') { throw 'A modify permission authorized deletion.' }
[IO.File]::WriteAllBytes($label,$before)
if ((Test-CirnetWorkPromptFile @intake @audit -RequirePreventiveScope).status -cne 'BLOCKED' -or
    (Get-CirnetHash (Join-Path $auditRepo '.git/index')) -cne $oldIndex) { throw 'The audit claimed containment or altered the index.' }
$null = Invoke-OwnerGit @('add','labels.mjs')
if ((Test-CirnetWorkPromptFile @intake @audit).status -cne 'FAIL') { throw 'An unapproved index change passed audit.' }
$wrongPin = @{ ScopeSnapshot=$snapshot; ScopeSnapshotSha256=('0' * 64) }
if ((Test-CirnetWorkPromptFile @intake @wrongPin).status -cne 'FAIL') { throw 'A scope audit accepted an unpinned snapshot.' }
$proposal = New-CirnetWorkPromptProposal -OriginalTask $original -Repository $auditRepo -SourcePath 'owner.md'
if ($proposal.status -cne 'REVIEW_REQUIRED' -or $proposal.lines.Count -ne 3 -or
    @($proposal.proposal.permissions.Values | Where-Object Count -gt 0).Count -or
    $proposal.proposal.sources.'owner.md' -cne (Get-CirnetHash $rules) -or
    $proposal.PSObject.Properties['authorizationSha256']) { throw 'Preparation granted authority, missed source pins or hid line numbers.' }
if ((New-CirnetWorkPromptProposal -OriginalTask 'missing' -Repository 'missing' -Sensitive).status -cne 'BLOCKED') {
    throw 'Sensitive approval preparation attempted access.'
}
$proposalJson = & (Join-Path $PSHOME 'pwsh') -NoProfile -File (Join-Path $repo 'scripts/validate-work-prompt.ps1') -Prepare -OriginalTask $original -Repository $auditRepo
if ($LASTEXITCODE -ne 2 -or ($proposalJson | ConvertFrom-Json).status -cne 'REVIEW_REQUIRED') { throw 'CLI preparation must keep review pending.' }
Write-Host 'PASS original task coverage, scope, evidence, review, sensitive and blocked handoffs.'
# GitHub Actions forwards $LASTEXITCODE. The final CLI case intentionally exits 3.
exit 0
