# Starts the title with the settings this fork is played with (README.md, "How to run it").
#   scripts\play.ps1 -Exe <path of your own copy's executable>
# -Safe         without the newer, faster ways (the first thing to try when something is wrong)
# -RayTracing   the title's own ray-traced indirect light: lit interiors, about a third slower
# -Build        target\diag (the default, the build all measurements are of) or target\release
# -Display      the display mode the title is told it is plugged into: 'monitor' (the default, this
#               PC's main screen, e.g. 2560x1440), 'console' (3840x2160, as before) or WxH
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
    [ValidatePattern('^(monitor|console|\d{3,4}[xX]\d{3,4})$')][string]$Display = 'monitor',
    [hashtable]$More = @{},
    [switch]$Log,
    [string]$LogFile = ''
)
$settings = @{
    VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_RELAY = '2'; VICTORY_WIDE_BACK = '1'; VICTORY_CARD_REHASH_RUNS = '1'
    VICTORY_JOIN_COPIES = '1'; VICTORY_INDIRECT_WINDOW = '1'; VICTORY_RELAY_COPY = '1'; VICTORY_CARD_STEPS = '2'
    VICTORY_INDIRECT_BATCH = '2'; VICTORY_MAP_THREADS = '1'; VICTORY_TITLE_PRIORITIES = '0'; VICTORY_CARD_AWAKE = '1'
    VICTORY_SHOW_AFTER = '1'; VICTORY_AMBIENT = '0.8'; VICTORY_D3D12_DRED = 'light'
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
if ($Display -eq 'monitor') {
    # In real pixels (unlike Windows.Forms, which a window without DPI awareness gets scaled).
    $screen = Get-CimInstance Win32_VideoController | Where-Object CurrentHorizontalResolution | Select-Object -First 1
    if ($screen) { $settings.VICTORY_DISPLAY_MODE = "$($screen.CurrentHorizontalResolution)x$($screen.CurrentVerticalResolution)" }
} elseif ($Display -ne 'console') {
    $settings.VICTORY_DISPLAY_MODE = $Display
}
if ($settings.VICTORY_DISPLAY_MODE) { Write-Host "Display mode told to the title: $($settings.VICTORY_DISPLAY_MODE)" }
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
