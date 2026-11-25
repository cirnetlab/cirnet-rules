#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

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

Export-ModuleMember -Function Resolve-CirnetRoot, Resolve-CirnetPath, Get-CirnetHash, Get-CirnetPayloadHash
