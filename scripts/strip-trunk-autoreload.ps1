$indexPath = Join-Path $PSScriptRoot '..\flashcards\frontend\dist\index.html'

if (-not (Test-Path $indexPath)) {
    throw "Built index.html not found at $indexPath"
}

$content = Get-Content $indexPath -Raw
$pattern = '(?s)</script><script>"use strict";.*?\{\{__TRUNK_ADDRESS__\}\}.*?</script>'
$updated = [System.Text.RegularExpressions.Regex]::Replace($content, $pattern, '</script>')

if ($updated -eq $content) {
    Write-Output 'No Trunk autoreload script found in dist/index.html.'
    exit 0
}

[System.IO.File]::WriteAllText($indexPath, $updated, [System.Text.UTF8Encoding]::new($false))
Write-Output 'Removed Trunk autoreload script from dist/index.html.'