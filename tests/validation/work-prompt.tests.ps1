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
$goal, $constraints, $outcome = [IO.File]::ReadAllLines($original)
$report = Get-CirnetWorkPromptReportLine
$body = @(
    'PROMPT_READY'
    "Original: $taskHash"
    "Authorization: $pin"
    $goal
    $constraints
    $outcome
    "Owner: $repo"
    "write: $source"
    'Verify Task#3: test: node --test'
    "Source: $source"
    $report
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
Expect FAIL ($body.Replace($constraints, ''))
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
Expect REVIEW_REQUIRED ($body.Replace("$constraints`n$outcome", "$outcome`n$constraints"))
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
foreach ($line in @($goal,$constraints,$outcome,"Owner: $repo","write: $source",$report,'Verify Task#3: test: node --test')) {
    Test-EntryPoint ($body.Replace($line,'## Placeholder')) 1 -Reviewed
}
Test-EntryPoint ($body.Replace($constraints, 'Task#2')) 1 -Reviewed
Test-EntryPoint ($body.Replace($outcome, 'Task#3')) 1 -Reviewed
Test-EntryPoint ($body.Replace($outcome, "~~~`n$outcome`n~~~")) 1 -Reviewed
Test-EntryPoint ($body.Replace($constraints, "<!--`n$constraints`n-->")) 1 -Reviewed
$standard = $body.Replace($goal,"## Goal & Expected Behavior`n$goal").Replace("Owner: $repo","## Permissions and checks`nOwner: $repo")
$deep = $standard.Replace($constraints,"## Requirements & Constraints`n$constraints").
    Replace('Verify Task#3: test: node --test',"## Acceptance & Verification`nVerify Task#3: test: node --test").
    Replace($report,"## Deliverables & Reporting`n$report")
foreach ($layout in @($body,$standard,$deep)) { Test-EntryPoint $layout 0 -Reviewed }
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
$blocked = $body.Replace($pin,$blockedPin).Replace('PROMPT_READY','PROMPT_BLOCKED').Replace('test: node --test',"read: $source").Replace("write: $source",'Write scope: []')
Expect BLOCKED $blocked $blockedPin
Expect FAIL ($blocked + [char]10 + "write: $source") $blockedPin
$pin = $blockedPin
Test-EntryPoint $blocked 3
Test-EntryPoint ($blocked + [char]10 + 'Ask the owner for clarification.') 3 -Reviewed
$catalog = Get-Content -Raw -LiteralPath (Join-Path $repo 'tests/benchmark/cases.json') | ConvertFrom-Json -AsHashtable
$case = $catalog | Where-Object id -CEQ E06
$candidate = @('PROMPT_READY', "Original task: $($case.prompt)", 'Write scope: ["src/labels.mjs"]',
    '## Requirements and constraints', 'Keep the original edit and unrelated work.',
    '## Acceptance and verification', 'Run the label tests before and after.', $report) -join [char]10
$reviewed = Get-CirnetPayloadHash ([Text.Encoding]::UTF8.GetBytes($candidate))
foreach ($sample in @(
    @($candidate, '', 'REVIEW_REQUIRED'),
    @($candidate, $reviewed, 'PASS'),
    @($candidate.Replace($case.prompt,'Improve formatting.'), $reviewed, 'FAIL'),
    @($candidate.Replace('Write scope: ["src/labels.mjs"]','Write scope: ["README.md"]'), $reviewed, 'FAIL'),
    @($candidate.Replace('Keep the original edit and unrelated work.','read: private.txt'), '', 'FAIL'),
    @($candidate.Replace('Keep the original edit and unrelated work.','read: src/labels.mjs'), '', 'REVIEW_REQUIRED'),
    @($candidate.Replace('Keep the original edit and unrelated work.','<!-- hidden requirement -->'), '', 'FAIL'),
    @($candidate.Replace('Keep the original edit and unrelated work.','TBD'), '', 'FAIL'),
    @($candidate.Replace('Run the label tests before and after.','See original task'), '', 'FAIL'),
    @($candidate.Replace($report,''), '', 'FAIL'),
    @(($candidate.Replace('## Acceptance and verification', "~~~`n## Acceptance and verification") + "`n~~~"), '', 'FAIL'),
    @($candidate.Replace('Run the label tests before and after.','Tests: PASS'), '', 'FAIL'),
    @(($candidate + ' Changed.'), $reviewed, 'FAIL')
)) {
    $observed = Test-CirnetCandidatePrompt $sample[0] $case $sample[1] -ReadablePaths 'src/labels.mjs'
    if ($observed.status -cne $sample[2]) { throw "Candidate expected $($sample[2]), got $($observed.status)." }
}
foreach ($invalid in @($candidate.Replace('Run the label tests before and after.','{{CHECK}}'),
        $candidate.Replace('Run the label tests before and after.','<insert verification here>'),
        $candidate.Replace('Run the label tests before and after.',('a' * 40 + ' ' + 'b' * 64)),
        $candidate.Replace($report,''))) {
    $digest = Get-CirnetPayloadHash ([Text.Encoding]::UTF8.GetBytes($invalid))
    if ((Test-CirnetCandidatePrompt $invalid $case $digest).status -cne 'FAIL') {
        throw 'Independent review bypassed Candidate minimum content.'
    }
}
$blockedCase = $catalog | Where-Object id -CEQ E04
$blockedCandidate = @('PROMPT_BLOCKED',"Original task: $($blockedCase.prompt)",'Write scope: []',
    '## Requirements and constraints','The producer contract requires authorization.',
    '## Acceptance and verification','Inspect the pinned consumer release.') -join [char]10
$blockedReview = Test-CirnetCandidatePrompt $blockedCandidate $blockedCase (Get-CirnetPayloadHash ([Text.Encoding]::UTF8.GetBytes($blockedCandidate)))
if ($blockedReview.status -cne 'BLOCKED' -or -not $blockedReview.clarificationApproved -or
    $blockedReview.executionStatus -cne 'NOT_RUN') { throw 'Review made a blocked Candidate executable.' }

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
$auditDraft = $body.Replace(($body -split '\n')[2], "Authorization: $pin").Replace("Source: $source","Source: labels.mjs`nSource: owner.md").
    Replace("Owner: $repo","Owner: $auditRepo").Replace("write: $source",'modify: labels.mjs')
[IO.File]::WriteAllText($draft,$auditDraft)
$intake = @{ Prompt=$draft; OriginalTask=$original; Authorization=$approval; AuthorizationSha256=$pin; Repository=$auditRepo }
if ((Test-CirnetWorkPromptFile @intake -CaptureScope).status -cne 'BLOCKED') {
    throw 'A CLI switch alone authorized a worktree-wide audit.'
}
$handoff = Test-CirnetWorkPromptFile @intake
$noAudit = Test-CirnetWorkPromptFile @intake -ExecutionPreflight
if ($handoff.handoffStatus -cne 'PASS' -or $handoff.executionStatus -cne 'NOT_RUN' -or
    $noAudit.executionStatus -cne 'BLOCKED' -or $noAudit.acceptanceStatus -cne 'NOT_RUN') {
    throw 'Validation alone implied authorized execution or acceptance.'
}
$record.scopeAudit = $true
$intake.AuthorizationSha256 = Save-Approval
[IO.File]::WriteAllText($draft,$auditDraft.Replace($pin,$intake.AuthorizationSha256))
$oldIndex = Get-CirnetHash (Join-Path $auditRepo '.git/index')
$capture = Test-CirnetWorkPromptFile @intake -CaptureScope
if ($capture.baselineStatus -cne 'PASS' -or $capture.executionStatus -cne 'NOT_RUN' -or
    $capture.scopeAuditStatus -cne 'NOT_RUN' -or $capture.acceptanceStatus -cne 'NOT_RUN' -or
    -not $capture.scopeSnapshot.git.files['scratch.txt'] -or
    -not $capture.scopeSnapshot.git.files['ignored.txt']) { throw 'Scope capture omitted current authorization or local files.' }
$snapshot = Join-Path $fixture 'scope.json'
[IO.File]::WriteAllText($snapshot,(ConvertTo-Json $capture -Depth 24 -Compress))
$audit = @{ ScopeSnapshot=$snapshot; ScopeSnapshotSha256=(Get-CirnetHash $snapshot) }
$legacySnapshot = Join-Path $fixture 'scope-without-git-metadata.json'
$legacy = [IO.File]::ReadAllText($snapshot) | ConvertFrom-Json -AsHashtable
$legacy.scopeSnapshot.git.Remove('gitMetadataSha256')
[IO.File]::WriteAllText($legacySnapshot,(ConvertTo-Json $legacy -Depth 24 -Compress))
if ((Test-CirnetWorkPromptFile @intake -ExecutionPreflight -ScopeSnapshot $legacySnapshot -ScopeSnapshotSha256 (Get-CirnetHash $legacySnapshot)).executionStatus -cne 'BLOCKED') {
    throw 'A scope baseline without Git metadata evidence authorized execution.'
}
if ((Test-CirnetWorkPromptFile @intake -ExecutionPreflight).executionStatus -cne 'BLOCKED') { throw 'Audit authority replaced a pinned baseline.' }
$preflight = Test-CirnetWorkPromptFile @intake @audit -ExecutionPreflight
if ($preflight.executionStatus -cne 'PASS' -or $preflight.scopeAuditStatus -cne 'NOT_RUN' -or
    $preflight.acceptanceStatus -cne 'NOT_RUN') { throw 'Pinned baseline did not authorize intake independently of reconciliation.' }
$gitConfig = Join-Path $auditRepo '.git/config'
$originalConfig = [IO.File]::ReadAllBytes($gitConfig)
try {
    $null = Invoke-OwnerGit @('config','--local','cirnet-fixture.marker','local-only')
    $metadataIntake = Test-CirnetWorkPromptFile @intake @audit -ExecutionPreflight
    $metadataAudit = Test-CirnetWorkPromptFile @intake @audit
    if ($metadataIntake.executionStatus -cne 'FAIL' -or $metadataAudit.scopeAuditStatus -cne 'FAIL') {
        throw 'Unauthorized Git config drift passed Work intake or scope reconciliation.'
    }
} finally { [IO.File]::WriteAllBytes($gitConfig,$originalConfig) }
$extraRef = 'refs/tags/cirnet-audit-probe'
try {
    $null = Invoke-OwnerGit @('update-ref',$extraRef,'HEAD')
    if ((Test-CirnetWorkPromptFile @intake @audit).scopeAuditStatus -cne 'FAIL') {
        throw 'A new Git ref passed scope reconciliation without a change to HEAD.'
    }
} finally { $null = Invoke-OwnerGit @('update-ref','-d',$extraRef) }
$cliIntake = & (Join-Path $PSHOME 'pwsh') -NoProfile -File (Join-Path $repo 'scripts/validate-work-prompt.ps1') @intake @audit -ExecutionPreflight
if ($LASTEXITCODE -ne 0 -or ($cliIntake | ConvertFrom-Json).executionStatus -cne 'PASS') { throw 'CLI Work intake did not expose the execution gate.' }
$mismatch = Join-Path $fixture 'mismatched-scope.json'
$different = [IO.File]::ReadAllText($snapshot) | ConvertFrom-Json -AsHashtable
$different.scopeSnapshot.effectsSha256 = '0' * 64
[IO.File]::WriteAllText($mismatch,(ConvertTo-Json $different -Depth 24 -Compress))
if ((Test-CirnetWorkPromptFile @intake -ExecutionPreflight -ScopeSnapshot $mismatch -ScopeSnapshotSha256 (Get-CirnetHash $mismatch)).executionStatus -cne 'FAIL') {
    throw 'An independently pinned baseline for different effects passed execution intake.'
}
[IO.File]::AppendAllText($rules,'stale reference')
if ((Test-CirnetWorkPromptFile @intake @audit -ExecutionPreflight).status -cne 'FAIL') { throw 'Stale pinned evidence passed execution intake.' }
[IO.File]::WriteAllText($rules,"Owner rules`nexisting unrelated edit")
[IO.File]::AppendAllText($untracked,'stale baseline')
if ((Test-CirnetWorkPromptFile @intake @audit -ExecutionPreflight).executionStatus -cne 'FAIL') { throw 'Stale working tree passed execution intake.' }
[IO.File]::WriteAllText($untracked,'user notes')
[IO.File]::AppendAllText($label,"`n// authorized")
$reconciled = Test-CirnetWorkPromptFile @intake @audit
if ($reconciled.scopeAuditStatus -cne 'PASS' -or $reconciled.executionStatus -cne 'NOT_RUN' -or
    $reconciled.acceptanceStatus -cne 'NOT_RUN') { throw 'Reconciliation missed a scoped modification or certified acceptance.' }
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
$moved = Join-Path $auditRepo 'moved.mjs'
[IO.File]::Move($label,$moved)
try {
    if ((Test-CirnetWorkPromptFile @intake @audit).scopeAuditStatus -cne 'FAIL') { throw 'An unapproved move passed audit.' }
} finally { [IO.File]::Move($moved,$label) }
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
$executionAuthority = $intake.AuthorizationSha256
$executionDraft = [IO.File]::ReadAllText($draft)
$readGoal = 'Inspect the owner rules and identify the approved label behavior.'
$readConstraints = 'Only read the owner guidance and preserve every local edit.'
$readOutcome = 'Report whether the owner guidance specifies the label behavior.'
[IO.File]::WriteAllLines($original,@($readGoal,$readConstraints,$readOutcome))
$record.originalSha256 = Get-CirnetHash $original
$record.sources.Remove('labels.mjs')
$record.actions.'1' = @('read: owner.md')
$record.checks.'3' = @('read: owner.md')
$record.Remove('scopeAudit')
$intake.AuthorizationSha256 = Save-Approval
$readDraft = $executionDraft.Replace($executionAuthority,$intake.AuthorizationSha256).Replace("Original: $taskHash","Original: $($record.originalSha256)").
    Replace($goal,$readGoal).Replace($constraints,$readConstraints).Replace($outcome,$readOutcome).
    Replace('modify: labels.mjs',"read: owner.md`nWrite scope: []").
    Replace("Source: labels.mjs`n",'').Replace('Verify Task#3: test: node --test','Verify Task#3: read: owner.md')
[IO.File]::WriteAllText($draft,$readDraft)
$stream = [IO.File]::Open($ignored,'Open','Write')
try { $stream.SetLength(32MB + 1) } finally { $stream.Dispose() }
$readOnly = Test-CirnetWorkPromptFile @intake -ExecutionPreflight
if ($readOnly.executionStatus -cne 'PASS' -or $readOnly.scopeAuditStatus -cne 'NOT_RUN') {
    throw 'Read-only preflight required or traversed the ignored-file audit.'
}
$readPin = $intake.AuthorizationSha256
$record.checks.'3' = @('test: node --test')
$record.scopeAudit = $true
$intake.AuthorizationSha256 = Save-Approval
[IO.File]::WriteAllText($draft,$readDraft.Replace($readPin,$intake.AuthorizationSha256).Replace('Verify Task#3: read: owner.md','Verify Task#3: test: node --test'))
if ((Test-CirnetWorkPromptFile @intake -ExecutionPreflight).executionStatus -cne 'BLOCKED' -or
    (Test-CirnetWorkPromptFile @intake -CaptureScope).baselineStatus -cne 'BLOCKED') {
    throw 'A potentially mutating test bypassed baseline or ignored-file limits.'
}
$record.permissions.build = @('cargo build')
$record.checks.'3' = @('build: cargo build')
$intake.AuthorizationSha256 = Save-Approval
[IO.File]::WriteAllText($draft,$readDraft.Replace($readPin,$intake.AuthorizationSha256).Replace('Verify Task#3: read: owner.md','Verify Task#3: build: cargo build'))
if ((Test-CirnetWorkPromptFile @intake -ExecutionPreflight).executionStatus -cne 'BLOCKED') {
    throw 'A build command bypassed the execution baseline.'
}
[IO.File]::WriteAllText($ignored,'ignored contents')
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
