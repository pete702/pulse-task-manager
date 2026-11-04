$repoRoot = $PSScriptRoot
$files = @("src/main.rs", "src/app_tests.rs", "src/theme.rs", "src/theme_fonts.rs", "src/core/selftest.rs", "src/i18n/frame.rs", "src/i18n/memory.rs", "src/i18n/mod.rs", "src/i18n/tests_formatted.rs", "src/i18n/tests_translations_en.rs", "src/modules/memory/action_bar.rs", "src/modules/memory/composition_bar.rs", "src/modules/memory/history.rs", "src/modules/memory/kernel_tiles.rs", "src/modules/memory/model.rs", "src/modules/memory/process_model.rs", "src/modules/memory/table_body.rs", "src/modules/memory/table_widgets.rs", "src/ui/placeholder_tab.rs", "src/ui/sidebar.rs", "src/ui/status_bar.rs", "src/ui/three_walls.rs")
foreach ($f in $files) {
    $fullPath = Join-Path $repoRoot $f
    $content = Get-Content $fullPath -Raw
    $regex = [System.Text.RegularExpressions.Regex]::new('[\u0400-\u052F]')
    $matches = $regex.Matches($content)
    if ($matches.Count -gt 0) {
        Write-Host "FILE: $f - $($matches.Count) matches"
        foreach ($m in $matches) {
            $start = [Math]::Max(0, $m.Index - 20)
            $len = [Math]::Min(40, $content.Length - $start)
            $context = $content.Substring($start, $len)
            Write-Host "  Pos $($m.Index): '$($m.Value)' in context: '$context'"
        }
    }
}