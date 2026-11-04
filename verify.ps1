$repoRoot = $PSScriptRoot
$backupDir = Join-Path $repoRoot "..\PULSE_BACKUPS\v0.1.2_code_only"
Write-Host "=== BACKUP CONTENTS ==="
Get-ChildItem $backupDir | Select-Object Mode,Name,Length | Format-Table -AutoSize
Write-Host "=== CARGO.LOCK VERSION ==="
Select-String -Path (Join-Path $backupDir "Cargo.lock") -Pattern 'name = "pulse_manager"' -Context 0,2 | ForEach-Object { $_.Line; $_.Context.PostContext }
Write-Host "=== BINARY SIZE ==="
Get-Item (Join-Path $repoRoot "..\PULSE_BUILDS\PULSE_v0.1.2_windows_x64.exe") | Select-Object Name,Length
Write-Host "=== LINE COUNTS (KEY FILES) ==="
$files = @("src/modules/memory/chart_view.rs", "src/modules/memory/table_view.rs", "src/modules/memory/table_body.rs", "src/modules/memory/action_bar.rs", "src/modules/memory/page.rs", "src/modules/memory/safe_handle.rs", "src/core/worker.rs", "src/app.rs", "src/core/selftest_tests.rs", "tests/docs_guard.rs")
foreach ($f in $files) { $fullPath = Join-Path $repoRoot $f; $c = (Get-Content $fullPath).Count; Write-Host ("{0}: {1}" -f $f, $c) }
Write-Host "=== UNSAFE COUNT ==="
$count = 0
Get-ChildItem (Join-Path $repoRoot "src") -Recurse -Filter "*.rs" | ForEach-Object { $content = Get-Content $_.FullName -Raw; $count += ($content -split '#\[allow\(unsafe_code\)\]').Count - 1 }
Write-Host "Total #[allow(unsafe_code)]: $count"
Write-Host "=== CYRILLIC/NUL CHECK IN BACKUP ==="
$cyr = 0; $nul = 0
Get-ChildItem $backupDir -Recurse | Where-Object { $_.Extension -in @('.rs','.toml','.md','.yml','.yaml','.json','.txt') } | ForEach-Object {
    $t = Get-Content $_.FullName -Raw
    if ($t -match '[\u0400-\u052F]') { $cyr++; Write-Host "CYR: $($_.FullName)" }
    if ($t -match "`0") { $nul++; Write-Host "NUL: $($_.FullName)" }
}
Write-Host "CYRILLIC_FILES: $cyr, NUL_FILES: $nul"
Write-Host "=== RUSTFMT::SKIP CHECK ==="
$skip = 0
Get-ChildItem (Join-Path $repoRoot "src"),(Join-Path $repoRoot "tests") -Recurse -Filter "*.rs" | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    if ($content -match 'rustfmt::skip') { $skip++; Write-Host "SKIP: $($_.FullName)" }
}
Write-Host "RUSTFMT::SKIP COUNT: $skip"