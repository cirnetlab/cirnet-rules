#requires -Version 7.0
[CmdletBinding(SupportsShouldProcess, DefaultParameterSetName = 'Package')]
param(
    [string]$Source = (Split-Path -Parent $PSScriptRoot),
    [Parameter(Mandatory, ParameterSetName = 'Package')][string]$Output,
    [Parameter(ParameterSetName = 'Package')][string]$Revision,
    [Parameter(ParameterSetName = 'Package')][switch]$AllowDirty,
    [Parameter(ParameterSetName = 'Package')][switch]$Apply,
    [Parameter(Mandatory, ParameterSetName = 'Verify')][switch]$Verify
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'shared/Bundle.psm1') -Force

if ($Verify) { return Read-CirnetBundle -Source $Source }

function Invoke-SourceGit([string[]]$GitArgs) {
    $result = & git --no-optional-locks --no-replace-objects --no-lazy-fetch -c core.fsmonitor=false -c core.quotepath=false -C $root @GitArgs
    if ($LASTEXITCODE -ne 0) { throw "Git could not read the package source ($($GitArgs[0]))." }
    $result
}

function Get-ReleaseBlobId([byte[]]$Bytes) {
    $header = [Text.Encoding]::ASCII.GetBytes("blob $($Bytes.Length)`0")
    $sha = [Security.Cryptography.IncrementalHash]::CreateHash([Security.Cryptography.HashAlgorithmName]::SHA1)
    try {
        $sha.AppendData($header)
        $sha.AppendData($Bytes)
        return [BitConverter]::ToString($sha.GetHashAndReset()).Replace('-', '').ToLowerInvariant()
    }
    finally { $sha.Dispose() }
}

function Write-ZipEntry($Zip, [string]$Path, [byte[]]$Bytes) {
    $stream = $Zip.CreateEntry($Path).Open()
    try { $stream.Write($Bytes, 0, $Bytes.Length) }
    finally { $stream.Dispose() }
}

if (-not $AllowDirty -and -not $Revision) { throw 'A release package requires an explicit -Revision (commit or tag). Use -AllowDirty for development.' }
$root = Resolve-CirnetRoot -Path $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Source)
if (-not [IO.Directory]::Exists($root)) { throw "Package source directory '$root' is missing." }
$gitRoot = Resolve-CirnetRoot -Path (Invoke-SourceGit @('rev-parse', '--show-toplevel'))
if ($root -cne $gitRoot) { throw "Package source must be the Git worktree root '$gitRoot'." }
$head = Invoke-SourceGit @('rev-parse', '--verify', 'HEAD^{commit}')
if ($head -cnotmatch '^[0-9a-f]{40}\z') { throw 'Bundle format 1 requires a SHA-1 Git revision.' }
if ($Revision) {
    $selector = $Revision
    try {
        if ($selector -notmatch '^[0-9a-f]{40}\z') {
            if ($selector -cnotmatch '^refs/tags/') { $selector = "refs/tags/$selector" }
            Invoke-SourceGit @('check-ref-format', $selector) | Out-Null
        }
        $selected = Invoke-SourceGit @('rev-parse', '--verify', '--quiet', '--end-of-options', "$selector^{commit}")
    }
    catch { throw 'Selected -Revision is not a locally available Git commit or tag.' }
    if ($head -cne $selected) { throw 'Selected -Revision does not match the source HEAD.' }
}
$statusArgs = @('status', '--porcelain=v1', '--untracked-files=all', '--ignore-submodules=none')
$dirty = [bool](Invoke-SourceGit $statusArgs)
if ($dirty -and -not $AllowDirty) { throw 'Release packaging requires a clean source worktree. Use -AllowDirty for development.' }

$versionPath = Resolve-CirnetPath -Root $root -Relative 'VERSION'
$versionBytes = [IO.File]::ReadAllBytes($versionPath)
$versionLine = [Text.Encoding]::UTF8.GetString($versionBytes)
if (-not $versionLine.EndsWith("`n")) { throw 'VERSION must have one toolkit version line ending in LF.' }
$version = $versionLine.Substring(0, $versionLine.Length - 1)
if (-not (Test-CirnetVersion -Version $version)) { throw 'VERSION must have one MAJOR.MINOR.PATCH line ending in LF.' }
$payload = Get-CirnetPayload -Root $root -SkillRoot 'en/skills'
$paths = [string[]]@($payload.Keys)
[Array]::Sort($paths, [StringComparer]::Ordinal)

if (-not $dirty) {
    $gitSkills = @(Invoke-SourceGit @('ls-tree', '-d', '--name-only', "$head`:en/skills") | Where-Object { $_ -cmatch '^cirnet-' })
    $packagedSkills = @($paths | ForEach-Object {
        if ($_ -cmatch '^skills/([^/]+)/SKILL\.md\z') { $Matches[1] }
    })
    if ($gitSkills.Count -ne $packagedSkills.Count -or @($gitSkills | Where-Object { $_ -cnotin $packagedSkills }).Count -gt 0) {
        throw 'The checkout skills differ from the selected Git revision.'
    }
    $tracked = @{ VERSION = $versionBytes }
    foreach ($path in $paths) {
        $relative = $path
        if ($path.StartsWith('skills/', [StringComparison]::Ordinal)) { $relative = "en/$path" }
        $tracked[$relative] = $payload[$path]
    }
    foreach ($relative in $tracked.Keys) {
        $expected = Invoke-SourceGit @('rev-parse', '--verify', '--quiet', '--end-of-options', "$head`:$relative")
        if ($expected -cne (Get-ReleaseBlobId $tracked[$relative])) {
            throw "Checkout bytes differ from the selected Git revision at '$relative'."
        }
    }
}
if ($head -cne (Invoke-SourceGit @('rev-parse', '--verify', 'HEAD^{commit}')) -or
    $dirty -ne [bool](Invoke-SourceGit $statusArgs)) {
    throw 'The package source changed during selection. Run packaging again once it is stable.'
}

$records = @(
    foreach ($path in $paths) {
        [ordered]@{ path = $path; sha256 = Get-CirnetPayloadHash -Bytes $payload[$path] }
    }
)
$manifest = [ordered]@{ format = 1; version = $version; revision = $head; dirty = $dirty; files = $records }
$suffixedRevision = $head.Substring(0, 12)
if ($dirty) { $suffixedRevision += '-dirty' }
$outputRoot = Resolve-CirnetRoot -Path $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Output)
$archive = Resolve-CirnetPath -Root $outputRoot -Relative "cirnet-rules-$version-$suffixedRevision.zip"
$report = [pscustomobject]@{ Archive = $archive; Version = $version; Revision = $head; Dirty = $dirty; Applied = $false }
Write-Host "Package destination: $archive"
if (-not $Apply -or -not $PSCmdlet.ShouldProcess($archive, 'Create local bundle')) { return $report }
if (Test-Path -LiteralPath $archive) { throw "Package destination already exists: '$archive'." }

[void][IO.Directory]::CreateDirectory($outputRoot)
$staging = Join-Path $outputRoot ('.cirnet-package-' + [guid]::NewGuid().ToString('N') + '.tmp')
$handle = [IO.File]::Open($staging, [IO.FileMode]::CreateNew, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
try {
    try {
        $zip = [IO.Compression.ZipArchive]::new($handle, [IO.Compression.ZipArchiveMode]::Create, $true, [Text.Encoding]::UTF8)
        try {
            foreach ($path in $paths) { Write-ZipEntry $zip $path $payload[$path] }
            $manifestBytes = [Text.Encoding]::UTF8.GetBytes(($manifest | ConvertTo-Json -Depth 5) + "`n")
            Write-ZipEntry $zip 'bundle.json' $manifestBytes
        }
        finally { $zip.Dispose() }
    }
    finally { $handle.Dispose() }
    [IO.File]::Move($staging, $archive)
}
finally { [IO.File]::Delete($staging) }
$report.Applied = $true
return $report
