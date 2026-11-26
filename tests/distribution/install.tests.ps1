#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$install = Join-Path $repo 'scripts/install.ps1'
$restore = Join-Path $repo 'scripts/restore.ps1'
$verify = Join-Path $repo 'scripts/verify-install.ps1'
Import-Module (Join-Path $repo 'scripts/shared/Bundle.psm1')
$testRoot = Resolve-CirnetPath -Root ([IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetFullPath([IO.Path]::GetTempPath()))) -Relative 'toolkit-tests'
$session = Resolve-CirnetPath -Root $testRoot -Relative ('installer-' + [guid]::NewGuid().ToString('N'))
$defaults = 'cirnet-implementation', 'cirnet-debugging', 'cirnet-verification'

function Write-Fixture([string]$Path, [string]$Text) {
    [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Path))
    [IO.File]::WriteAllText($Path, $Text)
}

function New-Bundle([string]$Version, [string]$Baseline = "# CIRNET $Version`n") {
    $root = Join-Path $session "bundle $Version"
    Write-Fixture (Join-Path $root 'AGENTS.md') $Baseline
    Write-Fixture (Join-Path $root 'LICENSE') "Fixture license`n"
    Write-Fixture (Join-Path $root 'THIRD_PARTY_NOTICES.md') "# Fixture notices`n"
    foreach ($name in ($defaults + 'cirnet-extra')) {
        Write-Fixture (Join-Path $root "skills/$name/SKILL.md") "---`nname: $name`ndescription: Use for this fixture skill.`n---`n# $name $Version`n`n[Note](references/note.md)`n"
        Write-Fixture (Join-Path $root "skills/$name/references/note.md") "# Fixture reference $Version`n"
    }
    $files = foreach ($file in Get-ChildItem -LiteralPath $root -Recurse -File) {
        [ordered]@{
            path = [IO.Path]::GetRelativePath($root, $file.FullName).Replace('\', '/')
            sha256 = (Get-FileHash -LiteralPath $file.FullName).Hash.ToLowerInvariant()
        }
    }
    Write-Fixture (Join-Path $root 'bundle.json') (([ordered]@{
        format = 1; version = $Version; revision = ('a' * 40); dirty = $false; files = @($files)
    } | ConvertTo-Json -Depth 5) + "`n")
    $root
}

function Assert-Equal($Actual, $Expected, [string]$Message) {
    if ($Actual -cne $Expected) { throw "$Message`nExpected: $Expected`nActual: $Actual" }
}

function Assert-Fails([scriptblock]$Action, [string]$Pattern) {
    try { & $Action | Out-Null }
    catch {
        if ($_ -match $Pattern) { return }
        throw "Expected failure matching '$Pattern', got '$_'."
    }
    throw "Expected failure matching '$Pattern'."
}

function Get-Snapshot([string]$Root) {
    if (Test-Path -LiteralPath $Root) {
        (Get-ChildItem -LiteralPath $Root -Force -Recurse | Sort-Object FullName | ForEach-Object {
            $part = [IO.Path]::GetRelativePath($Root, $_.FullName)
            if ($_.PSIsContainer) { "directory $part" }
            else { "$part $($_.LastWriteTimeUtc.Ticks) $((Get-FileHash -LiteralPath $_.FullName).Hash)" }
        }) -join "`n"
    }
}

function Get-Backups([string]$Root, [string]$Agent) {
    Get-ChildItem -LiteralPath (Join-Path $Root ".cirnet-toolkit/$($Agent.ToLowerInvariant())/backups") -Recurse -File -Filter receipt.json
}

try {
    $bundle = New-Bundle '0.4.0'
    $next = New-Bundle '0.5.0'
    $fakeHome = Join-Path $session "profile $([char]0x0110)"
    $codexHome = Join-Path $session 'custom codex home'
    $baseline = Join-Path $codexHome 'AGENTS.md'
    $personal = "# Personal`r`n$([char]0x00e9) preferences"
    $personalBytes = [byte[]](@(0xef, 0xbb, 0xbf) + [Text.Encoding]::UTF8.GetBytes($personal))
    [void][IO.Directory]::CreateDirectory($codexHome)
    [IO.File]::WriteAllBytes($baseline, $personalBytes)
    Write-Fixture (Join-Path $codexHome 'AGENTS.override.md') "# Personal override`n"
    $overridePath = Join-Path $codexHome 'AGENTS.override.md'
    $overrideBytes = [IO.File]::ReadAllText($overridePath)
    $params = @{ Agent = 'Codex'; HomeRoot = $fakeHome; CodexHome = $codexHome }
    $before = Get-Snapshot $session
    & $install -Source $bundle @params -WarningAction SilentlyContinue | Out-Null
    & $install -Source $bundle @params -Apply -WhatIf -WarningAction SilentlyContinue | Out-Null
    Assert-Equal (Get-Snapshot $session) $before 'Preview or WhatIf changed a file.'

    $isolatedHome = Join-Path $session 'isolated home'
    $inheritedCodex = Join-Path $session 'inherited codex'
    $savedCodexHome = $env:CODEX_HOME
    try {
        $env:CODEX_HOME = $inheritedCodex
        & $install -Source $bundle -Agent Codex -HomeRoot $isolatedHome -Skills @() -Apply | Out-Null
        Assert-Equal (Test-Path -LiteralPath (Join-Path $isolatedHome '.codex/AGENTS.md')) $true 'Explicit HomeRoot did not isolate Codex.'
        Assert-Equal (Test-Path -LiteralPath $inheritedCodex) $false 'An inherited Codex home escaped the explicit HomeRoot.'
        Assert-Equal (Test-Path -LiteralPath (Join-Path $isolatedHome '.agents')) $false 'Baseline-only install created skill paths.'
    } finally { $env:CODEX_HOME = $savedCodexHome }
    & $install -Source $bundle @params -Apply -WarningAction SilentlyContinue | Out-Null
    $installed = [IO.File]::ReadAllBytes($baseline)
    Assert-Equal ([Convert]::ToHexString($installed[0..($personalBytes.Length - 1)])) ([Convert]::ToHexString($personalBytes)) 'Install changed personal bytes.'
    Assert-Equal ([IO.File]::ReadAllText($overridePath)) $overrideBytes 'Install changed Codex override.'
    Assert-Equal (@(Get-Backups $fakeHome 'Codex').Count) 1 'Expected one backup after fresh install.'
    $initialReceipt = (Get-Backups $fakeHome 'Codex').FullName
    foreach ($name in $defaults) {
        Assert-Equal (Test-Path -LiteralPath (Join-Path $fakeHome ".agents/skills/$name/references/note.md")) $true 'Default skill resource missing.'
    }
    Assert-Equal (Test-Path -LiteralPath (Join-Path $fakeHome '.agents/skills/cirnet-extra')) $false 'Opt-in skill installed by default.'
    $after = Get-Snapshot $session
    & $verify -Source $bundle @params -WarningAction SilentlyContinue | Out-Null
    & $install -Source $bundle @params -Apply -WarningAction SilentlyContinue | Out-Null
    Assert-Equal (Get-Snapshot $session) $after 'Verify or repeated install changed files.'

    & $install -Source $next @params -Skills ($defaults + 'cirnet-extra') -Apply -WarningAction SilentlyContinue | Out-Null
    $update = (Get-Backups $fakeHome 'Codex' | Where-Object FullName -ne $initialReceipt).FullName
    Assert-Fails { & $restore -Backup $initialReceipt @params -Apply } 'conflict|changed'
    $savedReceipt = [IO.File]::ReadAllText($update)
    $receipt = $savedReceipt | ConvertFrom-Json -AsHashtable
    # Change a destination outside the selected root in the receipt.
    $receipt.files[$receipt.files.Count - 1].relative = '../sibling.txt'
    [IO.File]::WriteAllText($update, ($receipt | ConvertTo-Json -Depth 8))
    $after = Get-Snapshot $session
    Assert-Fails { & $restore -Backup $update @params -Apply } 'Invalid backup destination'
    Assert-Equal (Get-Snapshot $session) $after 'A path in an untrusted receipt caused a write.'
    $receipt = $savedReceipt | ConvertFrom-Json -AsHashtable
    $receipt.files = @($receipt.files | Where-Object relative -CNE '.agents/skills/cirnet-extra/references/note.md')
    [IO.File]::WriteAllText($update, ($receipt | ConvertTo-Json -Depth 8))
    $after = Get-Snapshot $session
    Assert-Fails { & $restore -Backup $update @params -Apply } 'omits a file whose ownership would change'
    Assert-Equal (Get-Snapshot $session) $after 'An incomplete receipt caused a write.'
    [IO.File]::WriteAllText($update, $savedReceipt)
    $after = Get-Snapshot $session
    & $restore -Backup $update @params -Apply -WhatIf | Out-Null
    Assert-Equal (Get-Snapshot $session) $after 'Restore WhatIf changed files.'
    & $restore -Backup $update @params -Apply | Out-Null
    & $verify -Source $bundle @params -WarningAction SilentlyContinue | Out-Null
    Assert-Equal (Test-Path -LiteralPath (Join-Path $fakeHome '.agents/skills/cirnet-extra/SKILL.md')) $false 'Update restore left the opt-in skill installed.'

    $backupDir = Split-Path $initialReceipt
    $oldFile = Get-ChildItem -LiteralPath $backupDir -File | Where-Object Name -ne 'receipt.json' | Select-Object -First 1
    $oldBytes = [IO.File]::ReadAllBytes($oldFile.FullName)
    [IO.File]::AppendAllText($oldFile.FullName, 'corrupt')
    $after = Get-Snapshot $session
    Assert-Fails { & $restore -Backup $initialReceipt @params -Apply } 'backup|hash'
    Assert-Equal (Get-Snapshot $session) $after 'Corrupt backup caused a change.'
    [IO.File]::WriteAllBytes($oldFile.FullName, $oldBytes)
    [IO.File]::AppendAllText($baseline, "`n# Later edit`n")
    $after = Get-Snapshot $session
    Assert-Fails { & $restore -Backup $initialReceipt @params -Apply } 'conflict|changed'
    Assert-Equal (Get-Snapshot $session) $after 'Restore overwrote a later personal edit.'
    [IO.File]::WriteAllBytes($baseline, $installed)
    & $restore -Backup $initialReceipt @params -Apply | Out-Null
    Assert-Equal ([Convert]::ToHexString([IO.File]::ReadAllBytes($baseline))) ([Convert]::ToHexString($personalBytes)) 'Restore changed personal bytes.'

    $claude = Join-Path $session 'claude'
    $claudeBaseline = Join-Path $claude '.claude/CLAUDE.md'
    Write-Fixture $claudeBaseline "# Personal Claude rules`n"
    & $install -Source $bundle -Agent Claude -HomeRoot $claude -Apply | Out-Null
    Assert-Equal ([IO.File]::ReadAllText((Join-Path $claude '.claude/cirnet/AGENTS.md'))) "# CIRNET 0.4.0`n" 'Claude adapter target is incorrect.'
    $adapter = [IO.File]::ReadAllText($claudeBaseline)
    Assert-Equal ([regex]::Matches($adapter, '(?m)^@cirnet/AGENTS\.md\r?$').Count) 1 'Claude import missing or duplicated.'
    [IO.File]::AppendAllText($claudeBaseline, "@./cirnet/AGENTS.md`n")
    Assert-Fails { & $install -Source $bundle -Agent Claude -HomeRoot $claude -Apply } 'duplicate|conflict'
    [IO.File]::WriteAllText($claudeBaseline, $adapter + $adapter)
    Assert-Fails { & $install -Source $bundle -Agent Claude -HomeRoot $claude -Apply } 'block|conflict'

    $antigravity = Join-Path $session 'antigravity'
    $occupiedSkill = Join-Path $antigravity '.gemini/config/skills/cirnet-implementation/SKILL.md'
    Write-Fixture $occupiedSkill "# Pre-existing skill`n"
    $after = Get-Snapshot $session
    Assert-Fails { & $install -Source $bundle -Agent Antigravity -HomeRoot $antigravity -Apply } 'ownership|conflict'
    Assert-Equal (Get-Snapshot $session) $after 'Skill collision caused a partial install.'
    $antigravity = Join-Path $session 'antigravity clean'
    & $install -Source $bundle -Agent Antigravity -HomeRoot $antigravity -Apply | Out-Null
    & $verify -Source $bundle -Agent Antigravity -HomeRoot $antigravity | Out-Null
    Assert-Equal (Test-Path -LiteralPath (Join-Path $antigravity '.gemini/AGENTS.md')) $true 'Antigravity baseline missing.'
    Assert-Equal (Test-Path -LiteralPath (Join-Path $antigravity '.gemini/config/skills/cirnet-verification/SKILL.md')) $true 'Antigravity skill missing.'

    $caseHome = Join-Path $session 'case-sensitive fixture'
    Write-Fixture (Join-Path $caseHome '.gemini/agents.md') "# Case variant`n"
    $after = Get-Snapshot $caseHome
    Assert-Fails { & $install -Source $bundle -Agent Antigravity -HomeRoot $caseHome -Apply } 'case collision'
    Assert-Equal (Get-Snapshot $caseHome) $after 'Case conflict caused a write.'

    $overlapHome = Join-Path $session 'backup overlap'
    $overlapBundle = Join-Path $overlapHome '.cirnet-toolkit/codex/backups'
    [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($overlapBundle))
    Copy-Item -LiteralPath $bundle -Destination $overlapBundle -Recurse
    $after = Get-Snapshot $overlapHome
    Assert-Fails { & $install -Source $overlapBundle -Agent Codex -HomeRoot $overlapHome -Apply } 'backup location overlaps bundle'
    Assert-Equal (Get-Snapshot $overlapHome) $after 'Backup destination mutated its input bundle.'

    $linkHome = Join-Path $session 'linked destination'
    $outside = Join-Path $session 'outside destination'
    Write-Fixture (Join-Path $outside 'sentinel.txt') "Outside destination`n"
    [void][IO.Directory]::CreateDirectory($linkHome)
    $link = Join-Path $linkHome '.gemini'
    $linkType = if ($IsWindows) { 'Junction' } else { 'SymbolicLink' }
    try { New-Item -ItemType $linkType -Path $link -Target $outside | Out-Null }
    catch { Write-Host "NOT_RUN $linkType destination fixture: $_" }
    if ([IO.Directory]::Exists($link)) {
        try {
            $after = Get-Snapshot $outside
            Assert-Fails { & $install -Source $bundle -Agent Antigravity -HomeRoot $linkHome -Apply } 'reparse|symbolic link'
            Assert-Equal (Get-Snapshot $outside) $after 'Linked destination allowed an out-of-scope write.'
        } finally { [IO.Directory]::Delete($link) }
    }

    $badBaseline = New-Bundle '0.6.0' "# Binary fixture`0`n"
    $badHome = Join-Path $session 'invalid baseline'
    Assert-Fails { & $install -Source $badBaseline -Agent Codex -HomeRoot $badHome -Apply } 'invalid baseline|UTF-8'
    Assert-Equal (Test-Path -LiteralPath $badHome) $false 'Invalid baseline created an install destination.'

    if ($IsWindows) {
        $path = Join-Path $antigravity '.gemini/config/skills/cirnet-verification/SKILL.md'
        $before = Get-Snapshot (Join-Path $antigravity '.gemini')
        $lock = [IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
        try { Assert-Fails { & $install -Source $next -Agent Antigravity -HomeRoot $antigravity -Apply } 'Target changes rolled back \([1-9][0-9]* files\)' }
        finally { $lock.Dispose() }
        Assert-Equal (Get-Snapshot (Join-Path $antigravity '.gemini')) $before 'Rollback did not restore client data and timestamps.'
        & $verify -Source $bundle -Agent Antigravity -HomeRoot $antigravity | Out-Null
    } else { Write-Host 'NOT_RUN Windows file-lock rollback fixture' }

    Write-Host 'PASS exercised installer fixtures (client loading not exercised)'
} finally {
    if (Test-Path -LiteralPath $session) {
        $expected = Resolve-CirnetPath -Root ([IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetFullPath([IO.Path]::GetTempPath()))) -Relative 'toolkit-tests'
        $null = Resolve-CirnetRoot -Path $session
        $sessionItem = Get-Item -LiteralPath $session -Force
        if ($sessionItem.Parent.FullName -cne $expected -or $sessionItem.Name -cnotmatch '^installer-[0-9a-f]{32}$') {
            throw "Refusing fixture cleanup outside the test session: $session"
        }
        Remove-Item -LiteralPath $sessionItem.FullName -Recurse -Force
    }
}
