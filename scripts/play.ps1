# Starts the title with the settings this fork is played with (README.md, "How to run it").
#   scripts\play.ps1 -Exe <path of your own copy's executable>
# -Safe         without the newer, faster ways (the first thing to try when something is wrong)
# -RayTracing   the title's own ray-traced indirect light: lit interiors, about a third slower
# -Build        target\diag (the default, the build all measurements are of) or target\release
# -Display      the title's output size (VICTORY_OUTPUT): 2560x1440 (the default), 1920x1080,
#               1664x936, 1280x720, 'monitor' (this PC's main screen, if it is one of those or
#               3840x2160) or 'console' (left alone: 3840x2160, as the dump's settings say)
# -RenderScale  the title's own upscaler's scale (VICTORY_RENDER_SCALE, Series X): 1.0 (the
#               default: it renders at the output size), 0.5 to 1.0, or 'title' (its own 2/3)
# -More         other settings: -More @{VICTORY_PANEL='1'}
# -Log          also write the whole log to <data_dir>\logs\play-<date>-<time>.log (data_dir from
#               victory.toml or VICTORY_DATA_DIR); -LogFile <path> writes it there instead. The log
#               holds the title's data, so it may not be inside this repository (docs/asset-policy.md).
# Run it from a normal window, not as administrator: the driver then keeps its shader cache elsewhere.
param(
    [Parameter(Mandatory)][string]$Exe,
    [switch]$Safe,
    [switch]$RayTracing,
    [ValidateSet('diag', 'release')][string]$Build = 'diag',
    [ValidateSet('2560x1440', '1920x1080', '1664x936', '1280x720', '3840x2160', 'monitor', 'console')]
    [string]$Display = '2560x1440',
    [ValidatePattern('^(title|0\.[5-9]\d*|1(\.0*)?)$')][string]$RenderScale = '1.0',
    [hashtable]$More = @{},
    [switch]$Log,
    [string]$LogFile = ''
)
$settings = @{
    VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_RELAY = '2'; VICTORY_WIDE_BACK = '1'; VICTORY_CARD_REHASH_RUNS = '1'
    VICTORY_JOIN_COPIES = '1'; VICTORY_INDIRECT_WINDOW = '1'; VICTORY_RELAY_COPY = '1'; VICTORY_CARD_STEPS = '2'
    VICTORY_INDIRECT_BATCH = '2'; VICTORY_MAP_THREADS = '1'; VICTORY_TITLE_PRIORITIES = '0'; VICTORY_CARD_AWAKE = '1'
    VICTORY_SHOW_AFTER = '1'; VICTORY_AMBIENT = '0.8'; VICTORY_D3D12_DRED = 'light'
    # The pipelines the title creates are written down in the cache folder, for `victory warm --used`.
    VICTORY_PIPELINE_BOOK = '1'
    # A compute program whose approximate wave64 form hung the card (docs/findings.md); only the
    # diagnostics build (-Build diag) can skip it.
    VICTORY_SKIP_PIPELINES = '2561af9577c47289'
}
if ($Build -eq 'release') {
    Write-Host "Warning: the release build cannot skip compute 2561af9577c47289, which can hang the card."
}
if ($Safe) {
    $settings.VICTORY_UNDER_WIDE = '0'; $settings.VICTORY_LAND_ECHO = '0'
} else {
    $settings.VICTORY_THIN = '1'; $settings.VICTORY_JOIN_PARTS = '8'; $settings.VICTORY_UPLOAD_AHEAD = '1'
}
if ($RayTracing) { $settings.VICTORY_BVH_BUILDER = '1' }
# The title takes its output size from its own settings (3840x2160 in a Series X dump), not from
# the display mode it is told of: VICTORY_OUTPUT patches it to one of the sizes it has a mode for.
$modes = '3840x2160', '2560x1440', '1920x1080', '1664x936', '1280x720'
$output = $Display
if ($Display -eq 'monitor') {
    # In real pixels (unlike Windows.Forms, which a window without DPI awareness gets scaled).
    $screen = Get-CimInstance Win32_VideoController | Where-Object CurrentHorizontalResolution | Select-Object -First 1
    $size = if ($screen) { "$($screen.CurrentHorizontalResolution)x$($screen.CurrentVerticalResolution)" }
    $output = if ($size -in $modes) { $size } else { '2560x1440' }
    if ($size -notin $modes) { Write-Host "This screen ($size) is no size the title has a mode for: $output." }
}
if ($output -ne 'console') { $settings.VICTORY_OUTPUT = $output }
if ($RenderScale -ne 'title') { $settings.VICTORY_RENDER_SCALE = $RenderScale }
$shown = if ($output -eq 'console') { 'the title''s own (3840x2160)' } else { $output }
$scale = if ($RenderScale -eq 'title') { 'its own 2/3' } else { $RenderScale }
Write-Host "Output size: $shown; render scale: $scale"
foreach ($k in $More.Keys) { $settings[$k] = $More[$k] }

$victory = Join-Path $PSScriptRoot "..\target\$Build\victory.exe"
if (-not (Test-Path $victory)) { Write-Host "No $victory. Build it first (README.md)."; exit 1 }
if (-not (Test-Path $Exe)) { Write-Host "No $Exe."; exit 1 }
# (Other settings left over in this window go; where your files are stays.)
$paths = 'VICTORY_CONFIG', 'VICTORY_GAME_DIR', 'VICTORY_CACHE_DIR', 'VICTORY_DATA_DIR'
$inherited = @(Get-ChildItem Env:VICTORY_* | Where-Object { $_.Name -notin $paths })
foreach ($entry in $inherited) { Remove-Item "Env:$($entry.Name)" }
foreach ($k in $settings.Keys) { Set-Item "Env:$k" $settings[$k] }
$discarded = $inherited | Where-Object {
    $_.Name -notin $More.Keys -and -not $_.Value.Equals([Environment]::GetEnvironmentVariable($_.Name))
} | Sort-Object Name
if ($discarded) {
    Write-Host "Discarded inherited values for $($discarded.Name -join ', '). Use -More to pass them explicitly."
}
$Exe = (Resolve-Path $Exe).Path
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

if ($Log -and -not $LogFile) {
    $data = $env:VICTORY_DATA_DIR
    $toml = Join-Path $root 'victory.toml'
    if (-not $data -and (Test-Path $toml)) {
        $line = Select-String -Path $toml -Pattern '^\s*data_dir\s*=\s*[''"](.+)[''"]' | Select-Object -First 1
        if ($line) { $data = $line.Matches[0].Groups[1].Value }
    }
    if (-not $data) { Write-Host "No data_dir in victory.toml or VICTORY_DATA_DIR: give -LogFile <path>."; return }
    $LogFile = Join-Path $data "logs\play-$(Get-Date -Format 'yyyyMMdd-HHmmss').log"
}
$writer = $null
if ($LogFile) {
    $LogFile = [System.IO.Path]::GetFullPath($LogFile)
    if ($LogFile.StartsWith($root.TrimEnd('\') + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
        Write-Host "The log holds the title's data: write it outside this repository, not $LogFile."
        return
    }
    New-Item -ItemType Directory -Force (Split-Path $LogFile) | Out-Null
    # UTF-8 without colour codes, flushed line by line so that a crash keeps all of it.
    $writer = New-Object System.IO.StreamWriter($LogFile, $false, (New-Object System.Text.UTF8Encoding($false)))
    $writer.AutoFlush = $true
    Write-Host "Log: $LogFile"
}

Push-Location $root   # victory.toml is looked for here
try {
    if ($writer) {
        # The log comes on the error stream: each line as text, so that PowerShell does not show
        # it as errors (red, NativeCommandError).
        $ErrorActionPreference = 'Continue'
        & $victory run $Exe --execute 2>&1 | ForEach-Object {
            $text = "$_"
            $writer.WriteLine(($text -replace "$([char]27)\[[0-9;]*m", ''))
            $text
        }
    } else {
        & $victory run $Exe --execute
    }
} finally {
    if ($writer) { $writer.Dispose() }
    Pop-Location
}
