#requires -Version 7.0
[CmdletBinding()]
param(
    [string]$Base,
    [string]$Root = (Split-Path -Parent $PSScriptRoot)
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$checkout = (Get-Item -LiteralPath $Root).FullName
$manifest = Join-Path (Split-Path -Parent $PSScriptRoot) 'Cargo.toml'
$utf8 = [Text.UTF8Encoding]::new($false, $true)

function Invoke-ReadOnlyGit([string[]]$GitArgs) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = (Get-Command git -CommandType Application | Select-Object -First 1).Path
    $info.WorkingDirectory = $checkout
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    # Prevent automatic object fetches and deny transports on older Git, too.
    $info.Environment['GIT_NO_LAZY_FETCH'] = '1'
    $info.Environment['GIT_ALLOW_PROTOCOL'] = ''
    foreach ($arg in @('--no-optional-locks', '--no-replace-objects') + $GitArgs) {
        $info.ArgumentList.Add($arg)
    }
    $process = [Diagnostics.Process]::Start($info)
    $bytes = [IO.MemoryStream]::new()
    try {
        $stderr = $process.StandardError.ReadToEndAsync()
        $stdout = $process.StandardOutput.BaseStream.CopyToAsync($bytes)
        $process.WaitForExit()
        $null = $stdout.GetAwaiter().GetResult()
        $null = $stderr.GetAwaiter().GetResult()
        [pscustomobject]@{ Code = $process.ExitCode; Text = $utf8.GetString($bytes.ToArray()) }
    }
    finally {
        $bytes.Dispose()
        $process.Dispose()
    }
}

function Select-CheckedMarkdown([string]$Path) {
    $parts = $Path.Split('/')
    if ($parts[-1] -notmatch '.+\.md\z' -or
        ($parts.Count -gt 1 -and $parts[0] -cnotin @('en', 'vi') -and $Path -cnotlike 'docs/changelogs/*') -or
        ($parts | Where-Object { $_ -in @('.git', '.superpowers') })) {
        return
    }
    if ($Path -match '[\\:\p{Cc}]' -or ($parts | Where-Object { $_ -in @('', '.', '..') })) {
        Write-Warning 'Skipping an unsupported Markdown path in the Git diff. Use a plain relative filename.'
        return
    }
    $current = $checkout
    for ($index = 0; $index -lt $parts.Count; $index++) {
        $part = $parts[$index]
        $current = Join-Path $current $part
        $entry = Get-Item -LiteralPath $current -Force -ErrorAction SilentlyContinue
        if ($null -eq $entry -or $entry.Name -cne $part -or
            ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
            ($entry.PSIsContainer -ne ($index -lt $parts.Count - 1))) {
            Write-Warning 'Skipping a Markdown path that is not a regular file in the checkout.'
            return
        }
    }
    "--new-file=$Path"
}

$added = @()
$unavailable = 'Supply a nonzero full local Git commit SHA.'
if ($Base -match '^(?:[0-9a-f]{40}|[0-9a-f]{64})\z' -and $Base -notmatch '^0+\z') {
    try {
        $prefix = Invoke-ReadOnlyGit @('rev-parse', '--show-prefix')
        $before = Invoke-ReadOnlyGit @('rev-parse', '--verify', '--end-of-options', "$Base^{commit}")
        if ($prefix.Code -ne 0 -or $prefix.Text.TrimEnd() -ne '' -or $before.Code -ne 0 -or $before.Text.TrimEnd() -ne $Base) {
            $unavailable = 'The Git root or base commit could not be verified locally.'
        }
        elseif ((Invoke-ReadOnlyGit @('merge-base', '--is-ancestor', $Base, 'HEAD')).Code -ne 0) {
            $unavailable = 'The base is not a confirmed ancestor of checkout HEAD.'
        }
        else {
            $diff = Invoke-ReadOnlyGit @('diff', '--no-ext-diff', '--no-textconv', '--find-renames', '--name-only', '--diff-filter=A', '-z', $Base, 'HEAD', '--')
            if ($diff.Code -ne 0 -or ($diff.Text -and -not $diff.Text.EndsWith([char]0))) {
                $unavailable = 'The Git diff between base and checkout HEAD did not complete.'
            }
            else {
                $added = @($diff.Text.Split([char]0, [StringSplitOptions]::RemoveEmptyEntries) | ForEach-Object { Select-CheckedMarkdown $_ })
                $unavailable = $null
            }
        }
    }
    catch [System.ComponentModel.Win32Exception], [Text.DecoderFallbackException], [System.Management.Automation.CommandNotFoundException] {
        $unavailable = 'Local Git could not run or return valid UTF-8 paths.'
    }
}

if ($unavailable) {
    Write-Warning "new-file review skipped: $unavailable Running the regular documentation check."
}
else {
    Write-Host "Reviewing $($added.Count) added Markdown file(s) from Git base to checkout HEAD."
}
$checkerArgs = @('run', '--manifest-path', $manifest, '--offline', '--locked', '--bin', 'check-docs', '--', '--root', $checkout) + $added
& cargo @checkerArgs
exit $LASTEXITCODE
