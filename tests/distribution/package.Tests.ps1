#requires -Version 7.0
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$module = Join-Path $repo 'scripts/shared/Bundle.psm1'
$package = Join-Path $repo 'scripts/package.ps1'
if (-not (Test-Path -LiteralPath $module -PathType Leaf)) {
    throw 'Cannot verify a standalone bundle because scripts/shared/Bundle.psm1 is missing.'
}
Import-Module $module -Force

function Assert-BundleTest($Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Assert-BundleRejected([scriptblock]$Action, [string]$Expected) {
    try { & $Action | Out-Null }
    catch {
        if ($_.Exception.Message -match $Expected) { return }
        throw
    }
    throw "Expected rejection matching '$Expected'."
}

function Write-TestManifest {
    $manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $fixture 'bundle.json') -Encoding utf8NoBOM
}

$target = Resolve-CirnetPath -Root $repo -Relative 'target'
$session = Resolve-CirnetPath -Root $target -Relative ('package-test-' + [guid]::NewGuid().ToString('N'))
$fixture = Join-Path $session 'fixture Việt Nam'
$resource = 'skills/cirnet-verification/references/chi tiết.md'
$linkDoc = 'skills/cirnet-verification/SKILL.md'

try {
    New-Item -ItemType Directory -Path (Join-Path $fixture 'skills/cirnet-verification/references') -Force | Out-Null
    $payload = [ordered]@{
        'AGENTS.md' = "# Test baseline`n"
        'LICENSE' = "Test notice`n"
        'THIRD_PARTY_NOTICES.md' = "[License](LICENSE)`n"
        $linkDoc = @'
---
name: cirnet-verification
description: Fixture skill.
---

Read [Checklist][check].

[check]: <references/chi tiết.md#checklist>

```markdown
[Example](../../example.md)
```
'@
        $resource = "# Checklist`n`n[Skill](../SKILL.md)`n"
    }
    $manifest = [ordered]@{
        format = 1
        version = '0.2.0'
        revision = 'a' * 40
        dirty = $false
        files = @()
    }
    foreach ($path in $payload.Keys) {
        $file = Join-Path $fixture $path
        [IO.File]::WriteAllText($file, $payload[$path])
        $manifest.files += @{ path = $path; sha256 = (Get-FileHash -LiteralPath $file).Hash.ToLowerInvariant() }
    }
    Write-TestManifest
    $before = (Get-Item -LiteralPath (Join-Path $fixture 'bundle.json')).LastWriteTimeUtc
    $bundle = Read-CirnetBundle -Source $fixture
    Assert-BundleTest ($bundle.Root -ceq $fixture -and $bundle.Manifest.version -eq '0.2.0') 'Valid standalone fixture should load.'
    Assert-BundleTest ((Get-Item -LiteralPath (Join-Path $fixture 'bundle.json')).LastWriteTimeUtc -eq $before) 'Validation touched the manifest.'

    $relativePath = 'skills/cirnet-verification/unknown.json'
    Assert-BundleTest ((Resolve-CirnetPath -Root $fixture -Relative $relativePath) -ceq (Join-Path $fixture $relativePath).Replace('/', [IO.Path]::DirectorySeparatorChar)) 'Safe missing file should resolve.'
    foreach ($badPath in @('../neighbor', '/rooted', 'C:/rooted', 'x\y', 'x//y', 'x/y:stream', 'x/Con.txt', 'x/COM¹.txt', 'x/trailing.', 'x/trailing ', 'SKILLS/cirnet-verification/SKILL.md')) {
        Assert-BundleRejected { Resolve-CirnetPath -Root $fixture -Relative $badPath } 'path|case'
    }

    $manifest.files[0].sha256 = '0' * 64
    Write-TestManifest
    Assert-BundleRejected { Read-CirnetBundle -Source $fixture } 'hash'
    $manifest.files[0].sha256 = (Get-FileHash -LiteralPath (Join-Path $fixture 'AGENTS.md')).Hash.ToLowerInvariant()

    $manifest.files[0].path = '../AGENTS.md'
    Write-TestManifest
    Assert-BundleRejected { Read-CirnetBundle -Source $fixture } 'path'
    $manifest.files[0].path = 'AGENTS.md'

    $files = $manifest.files
    $manifest.files += @{ path = 'skills/cirnet-verification/References/ignored.md'; sha256 = '0' * 64 }
    Write-TestManifest
    Assert-BundleRejected { Read-CirnetBundle -Source $fixture } 'case|path'
    $manifest.files = $files
    Write-TestManifest

    $extra = Join-Path $fixture 'local-secret.txt'
    Set-Content -LiteralPath $extra -Value 'fixture only'
    Assert-BundleRejected { Read-CirnetBundle -Source $fixture } 'unexpected|allowlist'
    Remove-Item -LiteralPath $extra

    Remove-Item -LiteralPath (Join-Path $fixture $resource)
    $manifest.files = $files | Where-Object path -ne $resource
    Write-TestManifest
    Assert-BundleRejected { Read-CirnetBundle -Source $fixture } 'reference|missing'
    [IO.File]::WriteAllText((Join-Path $fixture $resource), $payload[$resource])
    $manifest.files = $files
    $entry = $manifest.files | Where-Object path -eq $linkDoc
    $linkFixture = "---`nname: cirnet-verification`ndescription: Fixture skill.`n---`n"
    foreach ($badLink in @(
        '[unsafe](../../AGENTS.md)',
        '[unsafe](%2e%2e/%2e%2e/AGENTS.md)',
        '[invalid fragment](references/chi%20ti%E1%BA%BFt.md#missing)',
        '[ADS](asset.md:stream)',
        "[Read][detail]`n`n[detail]: ../../AGENTS.md`n[DETAIL]: $($resource.Substring($resource.IndexOf('/references/') + 1).Replace(' ', '%20'))"
    )) {
        [IO.File]::WriteAllText((Join-Path $fixture $linkDoc), $linkFixture + $badLink)
        $entry.sha256 = (Get-FileHash -LiteralPath (Join-Path $fixture $linkDoc)).Hash.ToLowerInvariant()
        Write-TestManifest
        Assert-BundleRejected { Read-CirnetBundle -Source $fixture } 'reference|fragment|path'
    }
    foreach ($invalidMetadata in @(
        ($payload[$linkDoc] -replace 'name: cirnet-verification', 'name: cirnet-debugging'),
        ($payload[$linkDoc] -replace 'description: Fixture skill\.', 'description: []'),
        ($payload[$linkDoc] -replace 'description: Fixture skill\.', "description: Fixture skill.`nname: cirnet-verification")
    )) {
        [IO.File]::WriteAllText((Join-Path $fixture $linkDoc), $invalidMetadata)
        $entry.sha256 = (Get-FileHash -LiteralPath (Join-Path $fixture $linkDoc)).Hash.ToLowerInvariant()
        Write-TestManifest
        Assert-BundleRejected { Read-CirnetBundle -Source $fixture } 'frontmatter|metadata'
    }
    [IO.File]::WriteAllText((Join-Path $fixture $linkDoc), $payload[$linkDoc])
    $entry.sha256 = (Get-FileHash -LiteralPath (Join-Path $fixture $linkDoc)).Hash.ToLowerInvariant()
    Write-TestManifest

    $outside = Join-Path $session 'neighbor'
    New-Item -ItemType Directory -Path $outside | Out-Null
    $link = Join-Path $fixture 'skills/cirnet-verification/escape'
    try {
        if ($IsWindows) { New-Item -ItemType Junction -Path $link -Target $outside | Out-Null }
        else { New-Item -ItemType SymbolicLink -Path $link -Target $outside | Out-Null }
    }
    catch { Write-Host "NOT_RUN: reparse fixture unavailable ($($_.Exception.Message))" }
    if (Test-Path -LiteralPath $link) {
        try {
            Assert-BundleRejected { Resolve-CirnetPath -Root $fixture -Relative 'skills/cirnet-verification/escape/new.md' } 'reparse|symbolic'
            Assert-BundleRejected { Read-CirnetBundle -Source $fixture } 'reparse|symbolic'
        }
        finally { [IO.Directory]::Delete($link) }
    }

    $output = Join-Path $session 'output'
    $revision = git --no-optional-locks -C $repo rev-parse HEAD
    if ($LASTEXITCODE) { throw 'Cannot determine test worktree revision.' }
    $status = git --no-optional-locks -C $repo status --porcelain=v1 --untracked-files=all
    if ($LASTEXITCODE) { throw 'Cannot determine test worktree status.' }
    Assert-BundleRejected { & $package -Output $output } 'revision'
    Assert-BundleRejected { & $package -Output $output -Revision ('0' * 40) } 'revision'
    Assert-BundleRejected { & $package -Output $output -Revision HEAD -AllowDirty } 'revision'
    $selection = @{ Revision = $revision }
    if ($status) {
        Assert-BundleRejected { & $package -Output $output -Revision $revision } 'dirty|clean'
        $selection.AllowDirty = $true
    }
    & $package -Output $output @selection | Out-Null
    & $package -Output $output @selection -Apply -WhatIf | Out-Null
    Assert-BundleTest (-not (Test-Path -LiteralPath $output)) 'Preview or WhatIf created package output.'

    $report = & $package -Output $output @selection -Apply
    Assert-BundleTest (Test-Path -LiteralPath $report.Archive -PathType Leaf) 'ZIP was not created.'
    $extracted = Join-Path $session 'extracted Việt Nam'
    [IO.Compression.ZipFile]::ExtractToDirectory($report.Archive, $extracted)
    $extractedBundle = & $package -Verify -Source $extracted
    $paths = @($extractedBundle.Manifest.files | ForEach-Object path)
    Assert-BundleTest ($paths.Count -ge 4 -and $paths -ccontains 'AGENTS.md' -and $paths -ccontains 'LICENSE') 'Missing packaged runtime files.'
    Assert-BundleTest (-not ($paths | Where-Object { $_ -cnotmatch '^(?:AGENTS\.md|LICENSE|THIRD_PARTY_NOTICES\.md|skills/cirnet-.+)$' })) 'Payload escaped the allowlist.'
    Assert-BundleTest ($extractedBundle.Manifest.revision -ceq $revision -and $extractedBundle.Manifest.dirty -eq [bool]$status) 'Invalid bundle provenance.'
    $examples = 'skills/cirnet-implementation/references/engineering-examples.md'
    $authoringTemplate = 'skills/cirnet-work-prompt-authoring/references/chat-to-work-prompt.md'
    Assert-BundleTest ($paths -ccontains $examples) 'Implementation examples are missing from the extracted bundle.'
    Assert-BundleTest ($paths -ccontains $authoringTemplate) 'The canonical Work Prompt template is missing from the bundle.'
    $templateHash = Get-CirnetHash (Join-Path $repo 'en/templates/chat-to-work-prompt.md')
    Assert-BundleTest ((Get-CirnetHash (Join-Path $extracted $authoringTemplate)) -ceq $templateHash) 'Packaged Work Prompt template differs from its canonical source.'
    $sourceExample = Join-Path $repo ('en/' + $examples)
    Assert-BundleTest ((Get-CirnetHash (Join-Path $extracted $examples)) -ceq (Get-CirnetHash $sourceExample)) 'Packaged examples differ from the reviewed source.'
    Assert-BundleTest (-not ($paths | Where-Object { $_ -match '^vi/|^tests/|^scripts/|(?i)cases\.json|\.jsonl$' })) 'Evaluator or translation material escaped the bundle boundary.'

    $fakeClientHome = Join-Path $session 'fake client home'
    $install = Join-Path $repo 'scripts/install.ps1'
    $verify = Join-Path $repo 'scripts/verify-install.ps1'
    $restore = Join-Path $repo 'scripts/restore.ps1'
    $destination = @{ Agent = 'Codex'; HomeRoot = $fakeClientHome }
    $optionalSkills = @('cirnet-task-continuity', 'cirnet-plan-spec-authoring', 'cirnet-candidate-evaluation')
    & $install -Source $extracted @destination -Apply -WhatIf | Out-Null
    Assert-BundleTest (-not (Test-Path -LiteralPath $fakeClientHome)) 'WhatIf wrote into the fake home.'

    $defaultHome = Join-Path $session 'default client home'
    & $install -Source $extracted -Agent Codex -HomeRoot $defaultHome -Apply | Out-Null
    & $verify -Source $extracted -Agent Codex -HomeRoot $defaultHome | Out-Null
    $defaults = @(Get-ChildItem -LiteralPath (Join-Path $defaultHome '.agents/skills') -Directory | Sort-Object Name | ForEach-Object Name)
    Assert-BundleTest (($defaults -join ',') -ceq 'cirnet-debugging,cirnet-implementation,cirnet-verification') 'Unexpected default Skill selection.'

    foreach ($skill in $optionalSkills) {
        Assert-BundleTest ($paths -ccontains "skills/$skill/SKILL.md") "Opt-in Skill '$skill' is missing from the bundle."
    }
    $selectedSkills = $defaults + $optionalSkills + 'cirnet-work-prompt-authoring'
    & $install -Source $extracted @destination -Skills $selectedSkills -Apply | Out-Null
    & $verify -Source $extracted @destination -Skills $selectedSkills | Out-Null
    foreach ($skill in $optionalSkills) {
        $relative = "skills/$skill/SKILL.md"
        $sourceHash = Get-CirnetHash (Join-Path $repo "en/$relative")
        $installedHash = Get-CirnetHash (Join-Path $fakeClientHome ".agents/$relative")
        Assert-BundleTest ($installedHash -ceq $sourceHash) "Installed opt-in Skill '$skill' differs from its canonical source."
    }
    $installed = Join-Path $fakeClientHome ('.agents/' + $examples)
    $installedTemplate = Join-Path $fakeClientHome ('.agents/' + $authoringTemplate)
    Assert-BundleTest ((Get-CirnetHash $installedTemplate) -ceq $templateHash) 'Installed Work Prompt template differs from its canonical source.'
    $templateBytes = [IO.File]::ReadAllBytes($installedTemplate)
    [IO.File]::AppendAllText($installedTemplate, 'Unexpected bytes.')
    Assert-BundleRejected { & $verify -Source $extracted @destination -Skills $selectedSkills } 'validation|differ|conflict|verify|drift'
    [IO.File]::WriteAllBytes($installedTemplate, $templateBytes)
    Assert-BundleTest ((Get-CirnetHash $installed) -ceq (Get-CirnetHash $sourceExample)) 'Installed examples differ from the reviewed source.'
    $receipts = @(Get-ChildItem -LiteralPath (Join-Path $fakeClientHome '.cirnet-toolkit/codex/backups') -Filter receipt.json -File -Recurse)
    Assert-BundleTest ($receipts.Count -eq 1) 'Expected a single fake-home receipt.'
    & $restore -Backup $receipts[0].FullName @destination -Apply | Out-Null
    Assert-BundleTest (-not (Test-Path -LiteralPath $installed)) 'Restore left the packaged examples installed.'
    Assert-BundleRejected { & $package -Output $output @selection -Apply } 'exists'
    Write-Host 'PASS: package preview, allowlist, provenance, extracted resource, fake install and restore'
}
finally {
    $fullSession = Resolve-CirnetRoot -Path $session
    if ([IO.Path]::GetDirectoryName($fullSession) -cne $target -or [IO.Path]::GetFileName($fullSession) -cnotmatch '^package-test-[0-9a-f]{32}$') {
        throw 'Unexpected test cleanup path.'
    }
    if (Test-Path -LiteralPath $fullSession) { Remove-Item -LiteralPath $fullSession -Recurse -Force }
}
