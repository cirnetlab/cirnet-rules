#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$toolkit = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$fixture = (New-Item -ItemType Directory -Path (Join-Path $toolkit ('target/discovery-tests/' + [guid]::NewGuid().ToString('N'))) -Force).FullName
$main = (New-Item -ItemType Directory -Path (Join-Path $fixture 'primary checkout root')).FullName
$extra = (New-Item -ItemType Directory -Path (Join-Path $fixture 'other root')).FullName
$registry = Join-Path $fixture 'registry.json'
$task = Join-Path $fixture 'request.txt'
Import-Module (Join-Path $toolkit 'scripts/shared/WorkPrompt.psm1')

function New-TestCheckout([string]$Path) {
    $null = New-Item -ItemType Directory -Path $Path
    git -C $Path init -q
    git -C $Path -c user.name=Fixture -c user.email=fixture@example.invalid -c commit.gpgsign=false commit -q --allow-empty -m 'fixture'
    if ($LASTEXITCODE) { throw 'Cannot create test checkout.' }
}

$names = @('cirnet-remote','cirnet-contracts','cirnet-interface','cirnet-hardware','cirnet-simulator','cirnet-fpga')
foreach ($name in $names) { New-TestCheckout (Join-Path $main $name) }
$config = [ordered]@{ schemaVersion=1; searchRoots=@($main.Replace('\','/')); discoverGitChildren=$true; repositories=@() }
function Save-Registry {
    [IO.File]::WriteAllText($registry, (ConvertTo-Json $config -Depth 5))
}
function Prepare([string]$Request, [hashtable]$Options = @{}) {
    [IO.File]::WriteAllText($task, $Request)
    New-CirnetWorkPromptProposal -Autonomous -OriginalTask $task -Registry $registry @Options
}
Save-Registry
$request = 'Improve the CircuitGraph editing experience while preserving Undo/Redo and existing graph semantics.'
$result = Prepare $request
if ($result.readiness -cne 'PROMPT_READY' -or $result.repository -cne (Join-Path $main 'cirnet-interface') -or
    $result.authorizationStatus -cne 'PENDING' -or $result.executionStatus -cne 'NOT_RUN' -or
    -not $result.prompt.Contains($request) -or -not $result.prompt.Contains('Work preflight')) {
    throw 'The canonical domain did not produce a scoped, approval-pending handoff.'
}
foreach ($name in $names) {
    $result = Prepare "Fix documentation in $name."
    if ($result.repository -cne (Join-Path $main $name) -or $result.readiness -cne 'PROMPT_READY') {
        throw "Discovery failed for $name."
    }
}
foreach ($case in @(
    @('Fix Remote API validation.', 'cirnet-remote'),
    @('Update canonical shared schemas.', 'cirnet-contracts'),
    @('Improve Simulator worker behavior.', 'cirnet-simulator')
)) {
    $routed = Prepare $case[0]
    if ($routed.readiness -cne 'PROMPT_READY' -or $routed.repositoryName -cne $case[1]) { throw 'Architecture routing lost an owner.' }
}
$intentCases = @(
    @{ text='Hãy kiểm tra cirnet-interface, nếu phát hiện lỗi thì sửa và chạy kiểm thử.'; owner='cirnet-interface'; scope='Edit' },
    @{ text='Tôi muốn cải thiện cirnet-remote nhưng không thay đổi cirnet-contracts.'; owner='cirnet-remote'; scope='Edit'; related='cirnet-contracts' },
    @{ text='Review the current module, fix the confirmed defects, and run the relevant tests.'; owner='cirnet-interface'; explicitOwner=$true; scope='Edit' },
    @{ text='Hãy kiểm tra cirnet-hardware và đề xuất các cải tiến, không chỉnh sửa mã nguồn.'; owner='cirnet-hardware'; scope='ReadOnly' },
    @{ text='Giúp tôi rà soát cirnet-interface, chỉ sửa khi cần thiết và không xóa tệp.'; owner='cirnet-interface'; scope='Edit' },
    @{ text='Vui lòng triển khai mã xử lý mới trong cirnet-interface.'; owner='cirnet-interface'; scope='Edit'; risk='STANDARD' },
    @{ text='Fix the cirnet-remote API compatibility issue with cirnet-contracts, but do not modify Contracts.'; owner='cirnet-remote'; scope='Edit'; related='cirnet-contracts' },
    @{ text='Review cirnet-interface without modifying or deleting files.'; owner='cirnet-interface'; scope='ReadOnly' },
    @{ text='Kiểm tra cirnet-interface, không xóa tệp và sửa lỗi nếu phát hiện.'; owner='cirnet-interface'; scope='Edit' },
    @{ text='Review cirnet-interface without deleting files and fix any confirmed defects.'; owner='cirnet-interface'; scope='Edit' },
    @{ text='Kiểm tra cirnet-hardware và đề xuất cải tiến, không sửa mã nguồn.'; owner='cirnet-hardware'; scope='ReadOnly' },
    @{ text='Không thay đổi cirnet-contracts; chỉ sửa API trong cirnet-remote.'; owner='cirnet-remote'; scope='Edit'; related='cirnet-contracts' },
    @{ text='Review the implementation, but do not change any source files.'; owner='cirnet-interface'; explicitOwner=$true; scope='ReadOnly' },
    @{ text='Review the implementation and update documentation only if necessary.'; owner='cirnet-interface'; explicitOwner=$true; scope='Edit'; risk='LIGHT' },
    @{ text='Refactor the core state-management logic in cirnet-interface.'; owner='cirnet-interface'; scope='Edit'; requestedRisk='LIGHT'; risk='STANDARD' },
    @{ text='Fix a documentation typo in cirnet-interface.'; owner='cirnet-interface'; scope='Edit'; requestedRisk='DEEP'; risk='DEEP' },
    @{ text='Fix the interface and npm publish.'; owner='cirnet-interface'; explicitOwner=$true; scope='Edit'; hold='deployment' },
    @{ text='Fix cirnet-interface and run pnpm publish.'; owner='cirnet-interface'; scope='Edit'; hold='deployment' },
    @{ text='Fix cirnet-interface, but do not git push or run pnpm publish.'; owner='cirnet-interface'; scope='Edit'; risk='STANDARD'; noHold=$true },
    @{ text='Fix cirnet-interface, but do not git push and git tag v1.0.'; owner='cirnet-interface'; scope='Edit'; risk='STANDARD'; noHold=$true },
    @{ text='Fix cirnet-interface and run `npm publish`.'; owner='cirnet-interface'; scope='Edit'; hold='deployment' },
    @{ text='Review cirnet-interface without deleting or modifying any source files.'; owner='cirnet-interface'; scope='ReadOnly'; noHold=$true },
    @{ text='Review cirnet-interface and explain how to run git tag.'; owner='cirnet-interface'; scope='ReadOnly'; noHold=$true },
    @{ text='Fix cirnet-interface and git tag v1.0.'; owner='cirnet-interface'; scope='Edit'; hold='git state change' },
    @{ text='Fix cirnet-interface and run kubectl apply for production.'; owner='cirnet-interface'; scope='Edit'; hold='deployment' },
    @{ text='Publish cirnet-interface with npm publish.'; owner='cirnet-interface'; scope='ReadOnly'; hold='deployment'; prepOnly=$true },
    @{ text='Fix the CircuitGraph editor. Do not modify cirnet-contracts.'; owner='cirnet-interface'; scope='Edit'; related='cirnet-contracts' },
    @{ text="Improve cirnet-interface without modifying 'graph semantics'."; owner='cirnet-interface'; scope='Edit' },
    @{ text='Remove the stale UI toggle from cirnet-interface.'; owner='cirnet-interface'; scope='Edit'; risk='STANDARD' },
    @{ text='Review cirnet-interface. The note says "edit cirnet-remote and deploy".'; owner='cirnet-interface'; scope='ReadOnly' },
    @{ text='Improve the UI and deploy it to production.'; owner='cirnet-interface'; explicitOwner=$true; scope='Edit'; hold='deployment' },
    @{ text='Fix the CircuitGraph UI, then export the output to a third party.'; owner='cirnet-interface'; scope='Edit'; hold='export' },
    @{ text='Fix the CircuitGraph UI and delete the original working files.'; owner='cirnet-interface'; scope='Edit'; hold='destructive' },
    @{ text='Fix the CircuitGraph UI and git push the branch.'; owner='cirnet-interface'; scope='Edit'; hold='git state change' },
    @{ text='Review cirnet-fpga and flash the physical FPGA.'; owner='cirnet-fpga'; scope='ReadOnly'; hold='physical' },
    @{ text='Update authentication in cirnet-remote.'; owner='cirnet-remote'; scope='Edit'; risk='DEEP' },
    @{ text='Fix a documentation typo in cirnet-interface.'; owner='cirnet-interface'; scope='Edit'; risk='LIGHT' }
)
$intentFailures = @(foreach ($case in $intentCases) {
    $opts = @{}
    if ($case['explicitOwner']) { $opts.Owner = $case.owner }
    if ($case['requestedRisk']) { $opts.Risk = $case.requestedRisk }
    elseif ($case['hold'] -or $case['risk'] -eq 'DEEP') { $opts.Risk = 'LIGHT' }
    $handoff = Prepare $case.text $opts
    if ($handoff.readiness -cne 'PROMPT_READY' -or $handoff.repositoryName -cne $case.owner -or
        $handoff.approvalProposal.scope -cne $case.scope -or $handoff.authorizationStatus -cne 'PENDING' -or
        $handoff.executionStatus -cne 'NOT_RUN' -or -not $handoff.prompt.Contains($case.text) -or
        ($case['related'] -and -not $handoff.prompt.Contains("Related owners require separately permitted reads: $($case.related).")) -or
        ($case['hold'] -and ($handoff.risk -cne 'DEEP' -or $case.hold -cnotin $handoff.pendingOperations -or $handoff.approvalProposal.grants.Count)) -or
        ($case['noHold'] -and $handoff.pendingOperations.Count) -or
        ($case['prepOnly'] -and $handoff.prompt -notmatch 'preflight only') -or
        ($case['risk'] -and $handoff.risk -cne $case.risk)) {
        "$($case.text) -> $($handoff.readiness)"
    }
})
$protected = Prepare 'Inspect cirnet-interface and read customer records.' @{Risk='LIGHT'}
if ($protected.readiness -cne 'PROMPT_BLOCKED') { $intentFailures += 'Protected read was handed off without preventive access.' }
$observed = Prepare "Review cirnet-interface.`nDo not modify the source."
if ($observed.readiness -cne 'PROMPT_READY' -or $observed.approvalProposal.scope -cne 'ReadOnly') {
    $intentFailures += 'A separate prohibitive instruction was interpreted as an edit.'
}
$contradiction = Prepare 'Edit cirnet-interface, but do not modify the source.'
if ($contradiction.readiness -cne 'PROMPT_BLOCKED') { $intentFailures += 'A conflicting blanket edit prohibition was ignored.' }
if ($intentFailures.Count) { throw ($intentFailures -join [char]10) }
$directOwner = Join-Path $main 'cirnet-interface'
foreach ($directRequest in @('Review the current implementation.', 'Fix the cirnet-interface editor using cirnet-implementation.')) {
    [IO.File]::WriteAllText($task,$directRequest)
    $direct = New-CirnetWorkPromptProposal -Autonomous -OriginalTask $task -Repository $directOwner
    if ($direct.readiness -cne 'PROMPT_READY' -or $direct.authorizationStatus -cne 'PENDING' -or
        $direct.executionStatus -cne 'NOT_RUN' -or $direct.proposedScope -cne $(if ($directRequest.StartsWith('Fix')) { 'Edit' } else { 'ReadOnly' })) {
        throw 'An unambiguous direct checkout required another scope argument.'
    }
}
[IO.File]::WriteAllText($task, 'Fix the cirnet-remote API.')
if ((New-CirnetWorkPromptProposal -Autonomous -OriginalTask $task -Repository $directOwner -AuthorizedScope Edit).readiness -cne 'PROMPT_BLOCKED') {
    throw 'An explicit checkout overrode a conflicting named owner.'
}
$seventh = 'cirnet-new-fixture'
New-TestCheckout (Join-Path $main $seventh)
$result = Prepare "Update documentation in $seventh."
if ($result.repository -cne (Join-Path $main $seventh)) { throw 'A new child required a registry edit.' }
New-TestCheckout (Join-Path $extra 'cirnet-interface')
$config.searchRoots += $extra
Save-Registry
$duplicate = Prepare $request
if ($duplicate.readiness -cne 'PROMPT_BLOCKED' -or $duplicate.diagnostics[0] -notmatch 'multiple') { throw 'Ambiguous checkout accepted.' }
$outside = Join-Path $fixture 'relocated checkout'
New-TestCheckout $outside
$config.repositories = @(@{name='cirnet-interface';path=$outside}, @{name='graph-team';path=$outside})
Save-Registry
foreach ($known in @($request,'Fix graph-team formatting.')) {
    $resolved = Prepare $known
    if ($resolved.repository -cne $outside) { throw 'Explicit override or alias failed.' }
}
git -C $outside remote add origin 'https://github.com/cirnetlab/cirnet-remote.git'
if ((Prepare $request).readiness -cne 'PROMPT_BLOCKED') { throw 'A matching checkout path with a conflicting canonical identity was accepted.' }
git -C $outside remote set-url origin 'git@github.com:cirnetlab/cirnet-interface.git'
$config.repositories[1].origin = 'cirnetlab/cirnet-interface'
Save-Registry
$matched = Prepare 'Fix graph-team formatting.'
if ($matched.readiness -cne 'PROMPT_READY' -or $matched.repositoryIdentityStatus -cne 'ORIGIN_MATCH') {
    throw 'A relocated alias did not preserve its configured identity.'
}
git -C $outside remote remove origin
if ((Prepare 'Fix graph-team formatting.').readiness -cne 'PROMPT_BLOCKED') { throw 'An explicit origin pin was silently skipped.' }
$config.repositories[1].Remove('origin')
Save-Registry
$notMapped = (New-Item -ItemType Directory -Path (Join-Path $fixture 'unlisted')).FullName
New-TestCheckout (Join-Path $notMapped 'private-owner')
if ((Prepare 'Fix documentation in private-owner.' @{Owner='private-owner'}).readiness -cne 'PROMPT_BLOCKED') {
    throw 'Unconfigured external checkout was found.'
}
$config.repositories += @{name='graph-team';path=(Join-Path $extra 'cirnet-interface')}
Save-Registry
if ((Prepare 'Fix graph-team formatting.').diagnostics[0] -notmatch 'duplicate') { throw 'Repeated alias accepted.' }
$config.repositories = @($config.repositories[0..1])
Save-Registry
if ((Prepare "Fix cirnet-fpga and cirnet-hardware.").readiness -cne 'PROMPT_BLOCKED' -or
    (Prepare "Improve the CircuitGraph editor.`nGit push origin master").readiness -cne 'PROMPT_BLOCKED' -or
    (Prepare "Improve the CircuitGraph editor." @{Sensitive=$true}).readiness -cne 'PROMPT_BLOCKED') {
    throw 'An ambiguous or unauthorized effect reached authoring readiness.'
}
$config.repositories[0].path = Join-Path $fixture 'moved'
Save-Registry
if ((Prepare $request).readiness -cne 'PROMPT_BLOCKED') { throw 'Missing mapped checkout was silently substituted.' }
$config.repositories[0].path = "$main/../relocated checkout"
Save-Registry
if ((Prepare $request).readiness -cne 'PROMPT_BLOCKED') { throw 'Traversal mapping was accepted.' }
$config.repositories[0].path = $fixture
Save-Registry
if ((Prepare $request).readiness -cne 'PROMPT_BLOCKED') { throw 'A non-checkout mapping was accepted.' }
$link = Join-Path $main 'linked-owner'
$linked = $false
try {
    $kind = if ($IsWindows) { 'Junction' } else { 'SymbolicLink' }
    $null = New-Item -ItemType $kind -Path $link -Target $outside
    $linked = $true
} catch { Write-Host 'NOT_RUN: repository link fixture unavailable on this host.' }
if ($linked -and (Prepare 'Fix documentation in linked-owner.').readiness -cne 'PROMPT_BLOCKED') {
    throw 'A checkout escaped via a junction or symbolic link.'
}
$deferred = Prepare $request @{NoLocalDiscovery=$true}
if ($deferred.readiness -cne 'PROMPT_READY' -or $deferred.repository -or $deferred.revision -or
    $deferred.repositoryStatus -cne 'NOT_INSPECTED' -or -not $deferred.prompt.Contains('cirnet-interface')) {
    throw 'A no-local-access handoff invented a checkout or blocked safe preflight.'
}
$preserved = Prepare 'Review cirnet-hardware compatibility with cirnet-fpga.' @{Owner='cirnet-hardware';AuthorizedScope='ReadOnly'}
if ($preserved.risk -cne 'DEEP' -or $preserved.repositoryName -cne 'cirnet-hardware' -or
    -not $preserved.prompt.Contains('Related owners require separately permitted reads: cirnet-fpga.')) {
    throw 'An identified cross-repository dependency lost its independent boundary.'
}

$policy = Join-Path $fixture 'read-policy.json'
$rule = [ordered]@{
    schemaVersion=1; approvedBy='Fixture operator'; action='ReadOnly'; repository=(Join-Path $main 'cirnet-fpga')
    expiresAtUnixSeconds=[DateTimeOffset]::UtcNow.AddHours(1).ToUnixTimeSeconds(); readPaths=@('README.md')
}
[IO.File]::WriteAllText((Join-Path $rule.repository 'README.md'), 'Synthetic owner guide.')
function Save-Policy {
    [IO.File]::WriteAllText($policy, (ConvertTo-Json $rule))
    (Get-FileHash -LiteralPath $policy).Hash.ToLowerInvariant()
}
$options = @{Policy=$policy; PolicySha256=(Save-Policy)}
foreach ($inputText in @('Read README.md in cirnet-fpga.', 'Inspect documentation in cirnet-fpga.')) {
    $covered = Prepare $inputText $options
    if ($covered.readiness -cne 'PROMPT_READY' -or $covered.authorizationStatus -cne 'PENDING' -or
        $covered.policyStatus -cne 'INTEGRITY_CHECKED' -or $covered.executionStatus -cne 'NOT_RUN' -or
        -not $covered.prompt.Contains('README.md')) {
        throw 'Policy scope was lost or a self-supplied pin was presented as verified authority.'
    }
}
$unattested = Prepare 'Inspect documentation in cirnet-fpga.' ($options + @{SourcePath=@('README.md')})
if (@($unattested.evidence | Where-Object status -eq 'PINNED').Count) {
    throw 'A self-supplied policy pin triggered a source content read.'
}
if ((Prepare 'Edit README.md in cirnet-fpga.' $options).readiness -cne 'PROMPT_BLOCKED' -or
    (Prepare "Inspect cirnet-fpga.`nread: other.md" $options).readiness -cne 'PROMPT_BLOCKED' -or
    (Prepare 'Inspect cirnet-hardware.' $options).readiness -cne 'PROMPT_BLOCKED') {
    throw 'The read policy covered broader actions, paths or owners.'
}
$rule.expiresAtUnixSeconds = [DateTimeOffset]::UtcNow.AddHours(-1).ToUnixTimeSeconds()
$expired = Save-Policy
if ((Prepare 'Inspect cirnet-fpga.' $options).readiness -cne 'PROMPT_BLOCKED' -or
    (Prepare 'Inspect cirnet-fpga.' @{Policy=$policy;PolicySha256=$expired}).readiness -cne 'PROMPT_BLOCKED') {
    throw 'Changed or expired approval was accepted.'
}
Write-Host 'PASS dynamic registry, path safety, canonical routing, and deferred repository verification.'
