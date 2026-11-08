param(
    [Parameter(Mandatory = $true)][string] $InstallTree,
    [Parameter(Mandatory = $true)][string] $OutputDirectory,
    [Parameter(Mandatory = $true)][string] $QtVersion
)

$ErrorActionPreference = 'Stop'
$sourceRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$stage = (Resolve-Path -LiteralPath $InstallTree).Path
$output = (Resolve-Path -LiteralPath $OutputDirectory).Path
if ($output -eq $stage -or $output.StartsWith($stage + [IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {
    throw 'The output directory must be outside the installed tree.'
}
$runtime = Join-Path $stage 'bin/cgacp-runtime.exe'
$dashboard = Join-Path $stage 'bin/cgacp-dashboard.exe'
if (!(Test-Path -LiteralPath $dashboard -PathType Leaf) -or !(Test-Path -LiteralPath $runtime -PathType Leaf)) {
    throw 'Install the Release dashboard and runtime before packaging.'
}
$identity = & $runtime --version | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Could not verify runtime identity.' }
$versionFile = (Get-Content -LiteralPath (Join-Path $sourceRoot 'VERSION') -Raw).Trim()
$version = [string]$identity.version
if ($identity.product_generation -ne 'CGACP v1' -or $version -ne $versionFile -or
    $version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' -or
    $identity.release_name -ne "cirnet-rules-$version") {
    throw 'Runtime, source version and release name disagree.'
}
$tag = "v$version"
git --no-optional-locks -C $sourceRoot show-ref --verify --quiet "refs/tags/$tag"
if ($LASTEXITCODE -eq 0) { throw "A local $tag tag already exists. Review its provenance before creating a candidate." }
if ($LASTEXITCODE -ne 1) { throw 'Could not verify local tag status.' }
$head = (git --no-optional-locks -C $sourceRoot rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $head -notmatch '^[a-f0-9]{40,64}$') { throw 'Could not resolve the source revision.' }

$stageRows = [System.Collections.Generic.List[object]]::new()
foreach ($entry in Get-ChildItem -LiteralPath $stage -Force -Recurse | Sort-Object FullName) {
    $rel = [IO.Path]::GetRelativePath($stage,$entry.FullName).Replace('\','/')
    if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $rel -match '(^|/)(\.git|\.cgacp|secrets)(/|$)|\.sqlite3($|-)|^\.env') {
        throw "Unexpected data or link in staging tree: $rel"
    }
    if (!$entry.PSIsContainer) {
        $stageRows.Add([ordered]@{path=$rel; bytes=$entry.Length; sha256=(Get-FileHash -LiteralPath $entry.FullName -Algorithm SHA256).Hash.ToLowerInvariant()})
    }
}
if (!($stageRows.path -contains 'bin/Qt6Core.dll') -or !($stageRows.path -match '(^|/)platforms/qwindows\.dll$')) {
    throw 'The Qt Windows runtime was not deployed into the staging tree.'
}
$sourceLines = [System.Collections.Generic.List[string]]::new()
foreach ($rel in (git -c core.quotepath=false --no-optional-locks -C $sourceRoot ls-files --cached --others --exclude-standard | Sort-Object -Unique)) {
    $file = Join-Path $sourceRoot $rel
    if (!(Test-Path -LiteralPath $file -PathType Leaf)) { continue }
    $item = Get-Item -LiteralPath $file -Force
    if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Unexpected source link: $rel" }
    $digest = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
    $sourceLines.Add("$digest  $rel")
}
$sourceFingerprint = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
    [Text.Encoding]::UTF8.GetBytes(($sourceLines -join [char]10)))).ToLowerInvariant()
$name = "$($identity.release_name)-windows-x64-candidate"
$archive = Join-Path $output "$name.zip"
$manifest = Join-Path $output "$name.manifest.json"
$checksum = Join-Path $output "$name.sha256"
foreach ($file in @($archive,$manifest,$checksum)) {
    if (Test-Path -LiteralPath $file) { throw "Candidate output already exists: $file" }
}
[IO.Compression.ZipFile]::CreateFromDirectory($stage,$archive,[IO.Compression.CompressionLevel]::Optimal,$false)
$archiveHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
[ordered]@{
    schema=1
    product_generation=$identity.product_generation
    application_version=$version
    release_name=$identity.release_name
    proposed_tag=$tag
    release_status='RELEASE_BLOCKED'
    target='x86_64-pc-windows-msvc'
    qt_version=$QtVersion
    base_source_revision=$head
    source_worktree_sha256=$sourceFingerprint
    archive=[ordered]@{filename="$name.zip"; bytes=(Get-Item -LiteralPath $archive).Length;sha256=$archiveHash}
    files=$stageRows
    known_gates=@('native Agent verification','isolated execution','independent authenticated review','clean Windows installation','redistribution review')
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifest -Encoding utf8NoBOM
"$archiveHash  $name.zip" | Set-Content -LiteralPath $checksum -Encoding utf8NoBOM
Get-Item -LiteralPath $archive,$manifest,$checksum | Select-Object FullName,Length
