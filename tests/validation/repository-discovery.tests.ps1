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
    if ($covered.readiness -cne 'PROMPT_READY' -or $covered.authorizationStatus -cne 'READ_POLICY_CHECKED' -or
        $covered.executionStatus -cne 'NOT_RUN' -or -not $covered.prompt.Contains('README.md')) {
        throw 'A current independent read policy could not be reused.'
    }
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
