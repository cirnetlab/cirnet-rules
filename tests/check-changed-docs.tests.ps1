#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
Import-Module (Join-Path $repo 'scripts/Bundle.psm1') -Force

$target = Resolve-CirnetPath -Root $repo -Relative 'target'
$fixture = Resolve-CirnetPath -Root $target -Relative ('changed-docs-test-' + [guid]::NewGuid().ToString('N'))
$utf8 = [Text.UTF8Encoding]::new($false)
$bridge = Join-Path $repo 'scripts/check-changed-docs.ps1'

function Write-Document([string]$Path, [string]$Text) {
    $destination = Join-Path $fixture $Path
    New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
    [IO.File]::WriteAllText($destination, $Text, $utf8)
}

function Write-Pair([string]$Path, [string]$English, [string]$Vietnamese) {
    Write-Document "en/$Path" $English
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $fixture "en/$Path")).Hash.ToLowerInvariant()
    Write-Document "vi/$Path" "$Vietnamese`n<!-- source: en/$Path -->`n<!-- source-sha256: $hash -->`n"
}

function Invoke-FixtureGit([string[]]$GitArgs) {
    $hooks = Join-Path $fixture '.git/disabled-fixture-hooks'
    $excludes = Join-Path $fixture '.git/absent-excludes'
    $result = & git -C $fixture -c core.autocrlf=false -c core.fsmonitor=false -c "core.hooksPath=$hooks" -c "core.excludesFile=$excludes" @GitArgs
    if ($LASTEXITCODE -ne 0) { throw "Fixture Git failed: $($GitArgs[0])" }
    $result
}

function Invoke-Bridge([string]$Base, [int]$ExpectedCode) {
    $output = (& $bridge -Root $fixture -Base $Base 3>&1 | Out-String)
    if ($LASTEXITCODE -ne $ExpectedCode) {
        throw "Expected checker exit $ExpectedCode, got $LASTEXITCODE.`n$output"
    }
    $output
}

try {
    New-Item -ItemType Directory -Path $fixture | Out-Null
    Invoke-FixtureGit @('init', '--quiet', '--template=')
    Write-Document 'VERSION' "0.2.0`n"
    Write-Document 'Cargo.toml' "[package]`nname = 'fixture'`nversion = `"0.2.0`"`n"
    Write-Document 'AGENTS.md' "# Fixture instructions`n"
    Write-Document 'CLAUDE.md' "@AGENTS.md`n"
    Write-Pair 'AGENTS.md' "# Fixture instructions`n" "# Chỉ dẫn thử nghiệm`n"
    Write-Pair 'docs/changelog.md' "# Changes`n`n## 0.2.0`n" "# Thay đổi`n`n## 0.2.0`n"
    Write-Pair 'skills/test-skill/SKILL.md' "---`nname: test-skill`ndescription: Test skill.`n---`n`n# Skill`n" "# Kỹ năng thử nghiệm`n"
    Write-Document 'move-from.md' "# Rename example`n`nThis existing document will move with its contents unchanged.`n"
    Write-Document 'existing.md' "# Existing document`n`nThe earlier value is three bins.`n"
    Invoke-FixtureGit @('add', '--all')
    Invoke-FixtureGit @('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','commit.gpgsign=false','commit','--quiet','-m','fixture base')
    $base = Invoke-FixtureGit @('rev-parse', 'HEAD')

    Invoke-FixtureGit @('mv', '--', 'move-from.md', 'move-to.md')
    Write-Document 'existing.md' "# Existing document`n`nThe current value is four bins.`n"
    $unicodeName = 'docs/Đường dẫn $tên ; & bản mới.md'
    Write-Pair $unicodeName "# Local printer`n`nSpecify the connection and paper trays.`n" "# Máy in cục bộ`n`nNêu kết nối và khay giấy.`n"
    Write-Document '--looks-like-an-option.md' "# Root document`n`nRecord the checked release date in the manifest.`n"
    Write-Document 'external/not-in-scan.md' "# Outside checker inventory`n"
    Write-Document 'en/docs/settings.json' "{}`n"
    Write-Document 'en/.superpowers/hidden.md' "# Excluded input`n"
    if (-not $IsWindows) {
        # POSIX supports a literal LF in a filename. The selector must not split it.
        Write-Document "control`nname.md" "# Unsupported control character`n"
    }
    Invoke-FixtureGit @('add', '--all')
    Invoke-FixtureGit @('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','commit.gpgsign=false','commit','--quiet','-m','fixture head')
    Write-Document 'only-staged.md' "# Unpublished work`n`nThis file was staged after the checked commit.`n"
    Invoke-FixtureGit @('add', '--', 'only-staged.md')
    Write-Document 'only-untracked.md' "# Draft note`n`nThis file has not been added to Git.`n"
    $before = Invoke-FixtureGit @('status', '--porcelain=v1', '--untracked-files=all')

    $review = Invoke-Bridge -Base $base -ExpectedCode 0
    foreach ($path in @("en/$unicodeName", "vi/$unicodeName", '--looks-like-an-option.md')) {
        if ($review -notmatch ('NEW_DOCUMENT_REVIEW\s+file: ' + [regex]::Escape($path) + '\s')) {
            throw "New Markdown path did not reach the checker: $path`n$review"
        }
    }
    if ([regex]::Matches($review, 'NEW_DOCUMENT_REVIEW\s+file:').Count -ne 3) {
        throw "An unsupported path was nominated for new-file review.`n$review"
    }
    if (-not $IsWindows -and $review -notmatch 'unsupported Markdown path') {
        throw "The bridge did not report the control-filename limitation.`n$review"
    }

    # Confirm the checker exit from the bridge, not just successful argument forwarding.
    Write-Document 'first-copy.md' "# Duplicate test`n`nAllocate the new verification run under the owned trace directory.`n"
    Write-Document 'second-copy.md' "# Duplicate test`n`nAllocate the new verification run under the owned trace directory.`n"
    Invoke-FixtureGit @('add', '--', 'first-copy.md', 'second-copy.md')
    Invoke-FixtureGit @('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','commit.gpgsign=false','commit','--quiet','-m','fixture duplicate','--','first-copy.md','second-copy.md')
    $duplicate = Invoke-Bridge -Base $base -ExpectedCode 1
    if ($duplicate -notmatch 'ERROR: DOC_NEW_DUPLICATE') { throw "Missing projected duplicate error.`n$duplicate" }
    $unrelated = Invoke-FixtureGit @('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','commit.gpgsign=false','commit-tree','HEAD^{tree}','-m','unrelated fixture commit')
    foreach ($invalid in @('','--help',('0' * 40),('f' * 40),$unrelated)) {
        $fallback = Invoke-Bridge -Base $invalid -ExpectedCode 0
        if ($fallback -notmatch 'new-file review skipped' -or $fallback -match 'NEW_DOCUMENT_REVIEW|DOC_NEW_DUPLICATE' -or $fallback -notmatch 'PASS: bilingual documentation checks') {
            throw "The fallback did not report the missing base and run the regular check.`n$fallback"
        }
    }
    $after = Invoke-FixtureGit @('status', '--porcelain=v1', '--untracked-files=all')
    if (($before -join "`n") -cne ($after -join "`n")) { throw 'The bridge changed fixture files or its index.' }
    Write-Host 'PASS: read-only changed Markdown selection and checker exit propagation'
}
finally {
    $resolved = Resolve-CirnetRoot -Path $fixture
    if ([IO.Path]::GetDirectoryName($resolved) -cne $target -or [IO.Path]::GetFileName($resolved) -cnotmatch '^changed-docs-test-[0-9a-f]{32}$') {
        throw 'Unexpected fixture cleanup path.'
    }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}
