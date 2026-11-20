#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$bundleRootFiles = @('AGENTS.md', 'LICENSE', 'THIRD_PARTY_NOTICES.md')
$skillNamePattern = '^cirnet-[a-z0-9]+(?:-[a-z0-9]+)*\z'
$versionPattern = '^(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\z'
$strictUtf8 = [Text.UTF8Encoding]::new($false, $true)
$authoringSkill = 'skills/cirnet-work-prompt-authoring/SKILL.md'
$authoringTemplate = 'skills/cirnet-work-prompt-authoring/references/chat-to-work-prompt.md'

function Assert-CirnetName([string]$Name) {
    if ($Name -eq '' -or $Name -match '[\x00-\x1f<>:"/\\|?*]|[ .]\z' -or
        ($Name -split '\.', 2)[0].TrimEnd(' ') -match '^(?:CON|PRN|AUX|NUL|CONIN\$|CONOUT\$|(?:COM|LPT)[1-9¹²³])\z') {
        throw "Invalid path component '$Name'."
    }
}

function Assert-CirnetNotLink([IO.FileSystemInfo]$Item) {
    if ($Item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw "Reparse or symbolic link is forbidden at '$($Item.FullName)'."
    }
}

function Join-CirnetPath([string]$Base, [string[]]$Parts) {
    foreach ($part in $Parts) {
        Assert-CirnetName $part
        $joined = [IO.Path]::Combine($Base, $part)
        if ([IO.Directory]::Exists($Base)) {
            $matches = @([IO.DirectoryInfo]::new($Base).EnumerateFileSystemInfos() | Where-Object {
                [string]::Equals($_.Name, $part, [StringComparison]::OrdinalIgnoreCase)
            })
            if ($matches.Count -gt 1 -or ($matches.Count -eq 1 -and $matches[0].Name -cne $part)) {
                throw "Path case collision for '$part' in '$Base'."
            }
            if ($matches.Count -eq 1) { Assert-CirnetNotLink $matches[0] }
            else {
                try {
                    $null = [IO.File]::GetAttributes($joined)
                    throw "Unrecognized filesystem path alias '$joined'."
                }
                catch [IO.FileNotFoundException], [IO.DirectoryNotFoundException] { }
            }
        }
        elseif ([IO.File]::Exists($Base)) {
            throw "Path ancestor '$Base' is not a directory."
        }
        $Base = $joined
    }
    return $Base
}

function Resolve-CirnetRoot {
    param([Parameter(Mandatory)][string]$Path)
    if (-not [IO.Path]::IsPathFullyQualified($Path) -or $Path -match '^[\\/]{2}[?.][\\/]') {
        throw "Expected an absolute filesystem path, got '$Path'."
    }
    $separator = [IO.Path]::DirectorySeparatorChar
    $Path = $Path.Replace([IO.Path]::AltDirectorySeparatorChar, $separator)
    $anchor = [IO.Path]::GetPathRoot($Path)
    $tail = $Path.Substring($anchor.Length).TrimEnd($separator)
    Assert-CirnetNotLink ([IO.DirectoryInfo]::new($anchor))
    if ($tail) { return Join-CirnetPath $anchor ($tail.Split($separator)) }
    return $anchor
}

function Resolve-CirnetPath {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$Relative
    )
    $parts = $Relative.Split('/')
    foreach ($part in $parts) { Assert-CirnetName $part }
    return Join-CirnetPath (Resolve-CirnetRoot -Path $Root) $parts
}

function Get-CirnetHash {
    param([Parameter(Mandatory)][string]$Path)
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-CirnetPayloadHash {
    param([Parameter(Mandatory)][AllowEmptyCollection()][byte[]]$Bytes)
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($sha.ComputeHash($Bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Test-CirnetVersion {
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Version)
    return $Version -cmatch $versionPattern
}

function Get-CirnetText([byte[]]$Bytes, [string]$Path) {
    try { return $strictUtf8.GetString($Bytes) }
    catch { throw "Invalid UTF-8 in '$Path'." }
}

function Get-MarkdownProse([string]$Text, [switch]$KeepCodeSpans) {
    $fence = ''
    $Text = [regex]::Replace($Text, '(?s)<!--.*?(?:-->|\z)', '')
    foreach ($line in $Text -split '\r?\n') {
        $start = [regex]::Match($line, '^ {0,3}(`{3,}|~{3,})(.*)$')
        if ($fence) {
            if ($start.Success -and $start.Groups[1].Value[0] -ceq $fence[0] -and
                $start.Groups[1].Length -ge $fence.Length -and -not $start.Groups[2].Value.Trim()) { $fence = '' }
        }
        elseif ($start.Success) { $fence = $start.Groups[1].Value }
        elseif ($KeepCodeSpans) { $line }
        else { [regex]::Replace($line, '(`+)[^`]*\1(?!`)', '') }
    }
}

function Assert-CirnetSkillMetadata([string]$Text, [string]$Path, [string]$ExpectedName) {
    $frontmatter = [regex]::Match($Text, '\A---\r?\n(?s:(?<metadata>.+?))\r?\n---(?:\r?\n|\z)')
    if (-not $frontmatter.Success) { throw "Missing skill frontmatter in '$Path'." }
    $fields = @{}
    foreach ($line in $frontmatter.Groups['metadata'].Value -split '\r?\n') {
        $pair = [regex]::Match($line, '^(?<key>name|description): (?<value>.+)$')
        $key = $pair.Groups['key'].Value
        if (-not $pair.Success -or $fields.ContainsKey($key)) { throw "Invalid or duplicate skill frontmatter field in '$Path'." }
        $value = $pair.Groups['value'].Value
        if ($value -cmatch "^'([^']*)'$" ) { $value = $Matches[1] }
        elseif ($value.StartsWith('[') -or $value -match '^[\]{}''"&*!|>@`#%]|^[-?:]\s|:\s|\s#|^(?:true|false|yes|no|on|off|null|~|[+-]?[0-9]+(?:\.[0-9]+)?)$') {
            throw "Skill frontmatter field '$key' needs a supported single-line string in '$Path'."
        }
        $fields[$key] = $value
    }
    if ($fields['name'] -cne $ExpectedName -or -not $fields.ContainsKey('description') -or
        [string]::IsNullOrWhiteSpace($fields['description']) -or $fields['description'].Length -gt 1024) {
        throw "Skill frontmatter in '$Path' must name '$ExpectedName' and include a description of up to 1024 characters."
    }
}

function Get-MarkdownReferences([string]$Text, [string]$Path) {
    $definitions = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::OrdinalIgnoreCase)
    $lines = [Collections.Generic.List[string]]::new()
    $destination = '(?<destination><[^>\r\n]+>|[^\s)]+)'
    foreach ($line in Get-MarkdownProse $Text) {
        $definition = [regex]::Match($line, ('^ {0,3}\[(?<label>[^\]]+)\]:\s*' + $destination))
        if ($definition.Success) {
            $label = ($definition.Groups['label'].Value.Trim() -replace '\s+', ' ')
            [void]$definitions.TryAdd($label, $definition.Groups['destination'].Value)
        }
        else { $lines.Add($line) }
    }
    $definitions.Values
    $body = $lines -join "`n"
    $inline = '(?<!\\)\[[^\]\n]*\]\(\s*' + $destination + '(?:[ \t]+(?:"[^"\n]*"|''[^''\n]*''))?\s*\)'
    foreach ($match in [regex]::Matches($body, $inline)) { $match.Groups['destination'].Value }
    $body = [regex]::Replace($body, $inline, '')
    foreach ($match in [regex]::Matches($body, '(?<!\\)\[(?<text>[^\]\n]+)\]\[(?<label>[^\]\n]*)\]')) {
        $label = $match.Groups['label'].Value
        if (-not $label) { $label = $match.Groups['text'].Value }
        $label = $label.Trim() -replace '\s+', ' '
        if (-not $definitions.ContainsKey($label)) { throw "Undefined reference '$label' in '$Path'." }
    }
}

function Get-MarkdownAnchors([string]$Text) {
    $Text = [regex]::Replace($Text, '\A---\r?\n(?s:.*?)\r?\n---(?:\r?\n|\z)', '')
    $lines = @(Get-MarkdownProse $Text -KeepCodeSpans)
    $counts = [Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $heading = [regex]::Match($lines[$i], '^ {0,3}#{1,6}\s+(?<title>.+?)\s*#*\s*$')
        if ($heading.Success) { $title = $heading.Groups['title'].Value }
        elseif ($i + 1 -lt $lines.Count -and $lines[$i].Trim() -and $lines[$i + 1] -match '^ {0,3}(?:=+|-+)\s*$') {
            $title = $lines[$i].Trim()
        }
        else { continue }
        $title = $title -creplace '!?\[([^\]]+)\]\([^)]+\)', '$1' -creplace '<[^>]+>', ''
        $slug = ($title.ToLowerInvariant() -creplace '[^\p{L}\p{N}\s_-]', '').Replace(' ', '-')
        if ($counts.ContainsKey($slug)) {
            $counts[$slug]++
            "$slug-$($counts[$slug])"
        }
        else {
            $counts[$slug] = 0
            $slug
        }
    }
}

function Resolve-CirnetReference([string]$Path, [string]$Link) {
    $Link = $Link.Trim('<', '>')
    if ($Link -match '^(?:https?://|mailto:)') { return }
    $url, $fragment = $Link -split '#', 2
    $target = [Uri]::UnescapeDataString(($url -split '\?', 2)[0])
    if ($target.StartsWith('/') -or $target.Contains('\')) { throw "Unsafe reference '$Link' in '$Path'." }
    if ($target) {
        $parts = [Collections.Generic.List[string]]::new()
        $sourceParts = $Path.Split('/')
        $limit = 0
        if ($sourceParts[0] -ceq 'skills') { $limit = 2 }
        for ($i = 0; $i -lt $sourceParts.Count - 1; $i++) { $parts.Add($sourceParts[$i]) }
        foreach ($part in $target.Split('/')) {
            if ($part -ceq '.') { continue }
            if ($part -ceq '..') {
                if ($parts.Count -le $limit) { throw "Reference '$Link' escapes its scope in '$Path'." }
                $parts.RemoveAt($parts.Count - 1)
            }
            else {
                Assert-CirnetName $part
                $parts.Add($part)
            }
        }
        $target = $parts -join '/'
        if ($limit -eq 0 -and $target -cnotin $bundleRootFiles) {
            throw "Reference '$Link' is outside the root payload allowlist in '$Path'."
        }
    }
    else { $target = $Path }
    return [pscustomobject]@{ Path = $target; Fragment = [Uri]::UnescapeDataString([string]$fragment); Link = $Link }
}

function Get-CirnetSkillEntries([string]$Root, [string]$SkillRoot) {
    $directory = Resolve-CirnetPath -Root $Root -Relative $SkillRoot
    if (-not [IO.Directory]::Exists($directory)) { throw "Missing skill directory '$directory'." }
    $entries = @([IO.DirectoryInfo]::new($directory).EnumerateFileSystemInfos() | Where-Object { $_.Name -like 'cirnet-*' })
    if ($entries.Count -eq 0) { throw "No namespaced skills found in '$directory'." }
    foreach ($entry in $entries) {
        Assert-CirnetNotLink $entry
        if ($entry -isnot [IO.DirectoryInfo] -or $entry.Name -cnotmatch $skillNamePattern -or $entry.Name.Length -gt 64) {
            throw "Invalid namespaced skill directory '$($entry.FullName)'."
        }
        "skills/$($entry.Name)/SKILL.md"
    }
}

function Get-CirnetPayloadSourcePath([string]$Path, [string]$SkillRoot = 'en/skills') {
    if ($Path -ceq $authoringTemplate -and $SkillRoot -ceq 'en/skills') { return 'en/templates/chat-to-work-prompt.md' }
    if ($Path.StartsWith('skills/', [StringComparison]::Ordinal)) { return "$SkillRoot/$($Path.Substring(7))" }
    $Path
}

function Get-CirnetPayload {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][string]$SkillRoot
    )
    $pending = [Collections.Generic.Queue[object]]::new()
    $payload = [Collections.Generic.Dictionary[string,byte[]]]::new([StringComparer]::Ordinal)
    foreach ($path in $bundleRootFiles + @(Get-CirnetSkillEntries $Root $SkillRoot)) {
        $pending.Enqueue([pscustomobject]@{ Path = $path; Fragment = ''; From = 'bundle' })
    }
    while ($pending.Count -gt 0) {
        $request = $pending.Dequeue()
        $path = $request.Path
        $isMarkdown = $path.EndsWith('.md', [StringComparison]::OrdinalIgnoreCase)
        if (-not $payload.ContainsKey($path)) {
            $relative = Get-CirnetPayloadSourcePath $path $SkillRoot
            $file = Resolve-CirnetPath -Root $Root -Relative $relative
            if (-not [IO.File]::Exists($file)) { throw "Missing bundled file '$path' referenced by '$($request.From)'." }
            $payload[$path] = [IO.File]::ReadAllBytes($file)
            if ($isMarkdown) {
                $md = Get-CirnetText $payload[$path] $path
                if ($path -cmatch '^skills/([^/]+)/SKILL\.md\z') { Assert-CirnetSkillMetadata $md $path $Matches[1] }
                if ($path -ceq $authoringSkill -and $md -cmatch '(?m)^<!-- bundle-resource: references/chat-to-work-prompt\.md -->\r?$') {
                    $pending.Enqueue([pscustomobject]@{ Path = $authoringTemplate; Fragment = ''; From = $path })
                }
                foreach ($link in Get-MarkdownReferences $md $path) {
                    $reference = Resolve-CirnetReference $path $link
                    if ($reference) {
                        $pending.Enqueue([pscustomobject]@{ Path = $reference.Path; Fragment = $reference.Fragment; From = $path })
                    }
                }
            }
        }
        if ($request.Fragment -and (-not $isMarkdown -or $request.Fragment -cnotin @(Get-MarkdownAnchors (Get-CirnetText $payload[$path] $path)))) {
            throw "Missing Markdown fragment '$($request.Fragment)' in '$path' referenced by '$($request.From)'."
        }
    }
    return ,$payload
}

function Assert-CirnetTree([string]$Directory, [string]$Prefix, $ExpectedFiles, $ExpectedDirectories) {
    $names = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($entry in [IO.DirectoryInfo]::new($Directory).EnumerateFileSystemInfos()) {
        Assert-CirnetName $entry.Name
        Assert-CirnetNotLink $entry
        if (-not $names.Add($entry.Name)) { throw "Bundle case collision at '$Prefix$($entry.Name)'." }
        $path = "$Prefix$($entry.Name)"
        if ($entry -is [IO.DirectoryInfo]) {
            if (-not $ExpectedDirectories.Contains($path)) { throw "Unexpected bundle directory '$path'." }
            Assert-CirnetTree $entry.FullName "$path/" $ExpectedFiles $ExpectedDirectories
        }
        elseif (-not $ExpectedFiles.Contains($path)) { throw "Unexpected bundle file '$path'." }
    }
}

function Read-CirnetBundle {
    param([Parameter(Mandatory)][string]$Source)
    $root = Resolve-CirnetRoot -Path $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Source)
    $manifestPath = Resolve-CirnetPath -Root $root -Relative 'bundle.json'
    if (-not [IO.File]::Exists($manifestPath)) { throw "Missing bundle.json in '$root'." }
    $manifestText = Get-CirnetText ([IO.File]::ReadAllBytes($manifestPath)) 'bundle.json'
    $manifest = ConvertFrom-Json -InputObject $manifestText -Depth 8 -AsHashtable
    if ($manifest -isnot [Collections.IDictionary] -or
        -not ($manifest.Contains('format') -and $manifest.Contains('version') -and $manifest.Contains('revision') -and
            $manifest.Contains('dirty') -and $manifest.Contains('files')) -or $manifest.Count -ne 5 -or
        $manifest.format -isnot [long] -or $manifest.format -ne 1 -or
        $manifest.version -isnot [string] -or -not (Test-CirnetVersion -Version $manifest.version) -or
        $manifest.revision -isnot [string] -or $manifest.revision -cnotmatch '^[0-9a-f]{40}\z' -or
        $manifest.dirty -isnot [bool] -or $manifest.files -isnot [array]) {
        throw 'Invalid bundle.json manifest format or provenance.'
    }
    $files = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $caseInsensitive = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $directories = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    [void]$files.Add('bundle.json')
    foreach ($record in $manifest.files) {
        if ($record -isnot [Collections.IDictionary] -or $record.Count -ne 2 -or
            -not ($record.Contains('path') -and $record.Contains('sha256')) -or $record.path -isnot [string] -or
            $record.sha256 -isnot [string] -or $record.sha256 -cnotmatch '^[0-9a-f]{64}\z') {
            throw 'Invalid file path or hash in bundle.json.'
        }
        $path = $record.path
        if (($path -cnotin $bundleRootFiles -and $path -cnotmatch '^skills/cirnet-[a-z0-9]+(?:-[a-z0-9]+)*/.+\z') -or
            -not $caseInsensitive.Add($path)) { throw "Invalid or case-colliding bundle path '$path'." }
        $file = Resolve-CirnetPath -Root $root -Relative $path
        if (-not [IO.File]::Exists($file)) { throw "Missing manifest file '$path'." }
        [void]$files.Add($path)
        while (($slash = $path.LastIndexOf('/')) -ge 0) {
            $path = $path.Substring(0, $slash)
            [void]$directories.Add($path)
        }
    }
    Assert-CirnetTree $root '' $files $directories
    $payload = Get-CirnetPayload -Root $root -SkillRoot 'skills'
    if ($payload.Count -ne $manifest.files.Count) { throw 'Bundle manifest and the referenced payload have different file sets.' }
    foreach ($record in $manifest.files) {
        if (-not $payload.ContainsKey($record.path)) { throw "Bundle path '$($record.path)' is not a referenced payload file." }
        $hash = Get-CirnetPayloadHash -Bytes $payload[$record.path]
        if ($hash -cne $record.sha256) { throw "SHA-256 hash mismatch for '$($record.path)'." }
    }
    return [pscustomobject]@{ Root = $root; Manifest = [pscustomobject]$manifest }
}

Export-ModuleMember -Function Read-CirnetBundle, Resolve-CirnetRoot, Resolve-CirnetPath, Get-CirnetHash, Get-CirnetPayload,
    Get-CirnetPayloadHash, Get-CirnetPayloadSourcePath, Test-CirnetVersion
