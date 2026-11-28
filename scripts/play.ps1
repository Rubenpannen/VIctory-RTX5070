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
# -Console      the console the title sees: series-x (the default) or series-s
# -Dlss         NVIDIA DLSS takes the title's upscaling (docs/dlss.md; NVIDIA's files from
#               scripts\get-nvidia-sdks.ps1): off (the default), dlaa, quality, balanced,
#               performance; the scene renders at that mode's scale (-RenderScale only if DLSS
#               turns out not to be available)
# -FrameGen     DLSS frame generation as well (needs -Dlss, not with -Safe); -FrameGenX 2 to 6:
#               frames shown per frame rendered (2 by default)
# -NvidiaDefaults  also the switches VIctory-NVIDIA's launcher sets and this script does not (the
#               list is below, with what each does); not with -Safe. None of them is measured on
#               this PC yet: play the same scene with and without it.
# -More        other settings: -More @{VICTORY_PANEL='1'}
# -Log          also write the whole log to <data_dir>\logs\play-<date>-<time>.log (data_dir from
#               victory.toml or VICTORY_DATA_DIR); -LogFile <path> writes it there instead. The log
#               holds the title's data, so it may not be inside this repository (docs/asset-policy.md).
# -NoSaved      leave out what the F1 settings menu saved (-Safe does too)
#
# Saved settings: what the in-game F1 menu saved, <data_dir>\victory-settings.txt (one
# VICTORY_NAME=value a line; an empty value: unset), applies over this script's own settings but
# under what is given on the command line (-Dlss, -FrameGen, -FrameGenX, -Display, -RenderScale,
# -RayTracing, -More). F3 in the game shows and hides the stats panel.
# Run it from a normal window, not as administrator: the driver then keeps its shader cache elsewhere.
param(
    [Parameter(Mandatory)][string]$Exe,
    [switch]$Safe,
    [switch]$RayTracing,
    [ValidateSet('diag', 'release')][string]$Build = 'diag',
    [ValidateSet('2560x1440', '1920x1080', '1664x936', '1280x720', '3840x2160', 'monitor', 'console')]
    [string]$Display = '2560x1440',
    [ValidatePattern('^(title|0\.[5-9]\d*|1(\.0*)?)$')][string]$RenderScale = '1.0',
    [ValidateSet('series-x', 'series-s')][string]$Console = 'series-x',
    [ValidateSet('off', 'dlaa', 'quality', 'balanced', 'performance')][string]$Dlss = 'off',
    [switch]$FrameGen,
    [ValidateRange(2, 6)][int]$FrameGenX = 2,
    [switch]$NvidiaDefaults,
    [hashtable]$More = @{},
    [switch]$Log,
    [string]$LogFile = '',
    [switch]$NoSaved
)
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
# Paths victory takes from this window (kept below, never taken from the saved settings).
$paths = 'VICTORY_CONFIG', 'VICTORY_GAME_DIR', 'VICTORY_CACHE_DIR', 'VICTORY_DATA_DIR', 'VICTORY_STREAMLINE_DIR'
$modes = '3840x2160', '2560x1440', '1920x1080', '1664x936', '1280x720'
$dlssModes = 'dlaa', 'quality', 'balanced', 'performance'
$scalePattern = '^(title|0\.[5-9]\d*|1(\.0*)?)$'

# The data folder, as victory finds it (victory_core::Config): VICTORY_DATA_DIR, else victory.toml's
# data_dir (or that of the file VICTORY_CONFIG names), else %LOCALAPPDATA%\victory\data.
$data = $env:VICTORY_DATA_DIR
if (-not $data) {
    $toml = if ($env:VICTORY_CONFIG) { $env:VICTORY_CONFIG } else { 'victory.toml' }
    if (-not [System.IO.Path]::IsPathRooted($toml)) { $toml = Join-Path $root $toml }
    if (Test-Path $toml) {
        $line = Select-String -Path $toml -Pattern '^\s*data_dir\s*=\s*[''"](.+)[''"]' | Select-Object -First 1
        if ($line) { $data = $line.Matches[0].Groups[1].Value }
    }
}
if (-not $data -and $env:LOCALAPPDATA) { $data = Join-Path $env:LOCALAPPDATA 'victory\data' }
if ($data -and -not [System.IO.Path]::IsPathRooted($data)) { $data = Join-Path $root $data }

# What the F1 menu saved (crates\victory-gpu\src\host\menu.rs), unless -NoSaved or -Safe (the
# first thing to try when something is wrong: none of the saved speed switches either).
$savedFile = if ($data) { Join-Path $data 'victory-settings.txt' } else { $null }
if ($Safe -and -not $NoSaved -and $savedFile -and (Test-Path $savedFile)) {
    Write-Host "-Safe: the settings the F1 menu saved ($savedFile) are left out."
    $NoSaved = $true
}
$saved = [ordered]@{}
if (-not $NoSaved -and $savedFile -and (Test-Path $savedFile)) {
    foreach ($line in Get-Content $savedFile) {
        if ($line -match '^\s*(VICTORY_[A-Z0-9_]+)\s*=(.*)$' -and $Matches[1] -notin $paths) {
            $saved[$Matches[1]] = $Matches[2].Trim()
        }
    }
}
$given = @($PSBoundParameters.Keys)
$applied = New-Object System.Collections.Generic.List[string]
$overruled = New-Object System.Collections.Generic.List[string]
$fromSaved = @{}
# A saved line taken (or one the command line kept its own value over).
function Register-Saved([string]$name, [bool]$taken) {
    $text = if ($saved[$name] -eq '') { "$name=(unset)" } else { "$name=$($saved[$name])" }
    if ($taken) { $applied.Add($text) } else { $overruled.Add($name) }
}
function Skip-Saved([string]$name, [string]$why) {
    $applied.Remove("$name=$($saved[$name])") | Out-Null
    Write-Host "Saved $name=$($saved[$name]) left out: $why ($savedFile; -NoSaved: without saved settings)."
}

# The saved switches this script has options for set those options, unless they were given.
if ($saved.Contains('VICTORY_DLSS') -or $saved.Contains('VICTORY_DLSS_MODE')) {
    $names = @('VICTORY_DLSS', 'VICTORY_DLSS_MODE') | Where-Object { $saved.Contains($_) }
    if ('Dlss' -in $given) {
        $names | ForEach-Object { Register-Saved $_ $false }
    } else {
        $on = if ($saved.Contains('VICTORY_DLSS')) { $saved['VICTORY_DLSS'] -eq '1' } else { $Dlss -ne 'off' }
        $mode = if ($saved['VICTORY_DLSS_MODE']) { $saved['VICTORY_DLSS_MODE'] } elseif ($Dlss -ne 'off') { $Dlss } else { 'quality' }
        if ($on -and $mode -notin $dlssModes) {
            Skip-Saved 'VICTORY_DLSS_MODE' "not one of $($dlssModes -join ', ')"
        } else {
            $Dlss = if ($on) { $mode } else { 'off' }
            $fromSaved.Dlss = $true
            $names | ForEach-Object { Register-Saved $_ $true }
        }
    }
}
if ($saved.Contains('VICTORY_FRAMEGEN')) {
    if ('FrameGen' -in $given) { Register-Saved 'VICTORY_FRAMEGEN' $false } else {
        $FrameGen = $saved['VICTORY_FRAMEGEN'] -eq '1'
        $fromSaved.FrameGen = $true
        Register-Saved 'VICTORY_FRAMEGEN' $true
    }
}
# Frames shown per frame rendered; 0 (ready but off, switched on from the F1 menu) only from there.
$fgTimes = "$FrameGenX"
if ($saved.Contains('VICTORY_FRAMEGEN_X')) {
    $value = $saved['VICTORY_FRAMEGEN_X']
    if ('FrameGenX' -in $given) { Register-Saved 'VICTORY_FRAMEGEN_X' $false }
    elseif ($value -eq '') { Register-Saved 'VICTORY_FRAMEGEN_X' $true }
    elseif ($value -match '^[02-6]$') { $fgTimes = $value; Register-Saved 'VICTORY_FRAMEGEN_X' $true }
    else { Skip-Saved 'VICTORY_FRAMEGEN_X' 'not 0 or 2 to 6' }
}
if ($saved.Contains('VICTORY_OUTPUT')) {
    $value = $saved['VICTORY_OUTPUT']
    if ('Display' -in $given) { Register-Saved 'VICTORY_OUTPUT' $false }
    elseif ($value -eq '' -or $value -in $modes) {
        $Display = if ($value -eq '') { 'console' } else { $value }
        $fromSaved.Display = $true
        Register-Saved 'VICTORY_OUTPUT' $true
    } else { Skip-Saved 'VICTORY_OUTPUT' "not one of $($modes -join ', ')" }
}
if ($saved.Contains('VICTORY_RENDER_SCALE')) {
    $value = $saved['VICTORY_RENDER_SCALE']
    if ('RenderScale' -in $given) { Register-Saved 'VICTORY_RENDER_SCALE' $false }
    elseif ($value -eq '' -or $value -match $scalePattern) {
        $RenderScale = if ($value -eq '') { 'title' } else { $value }
        Register-Saved 'VICTORY_RENDER_SCALE' $true
    } else { Skip-Saved 'VICTORY_RENDER_SCALE' 'not 0.5 to 1.0' }
}

$settings = @{
    VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_RELAY = '2'; VICTORY_WIDE_BACK = '1'; VICTORY_CARD_REHASH_RUNS = '1'
    VICTORY_JOIN_COPIES = '1'; VICTORY_INDIRECT_WINDOW = '1'; VICTORY_RELAY_COPY = '1'; VICTORY_CARD_STEPS = '2'
    VICTORY_INDIRECT_BATCH = '2'; VICTORY_MAP_THREADS = '1'; VICTORY_TITLE_PRIORITIES = '0'; VICTORY_CARD_AWAKE = '1'
    VICTORY_SHOW_AFTER = '1'; VICTORY_AMBIENT = '0.8'; VICTORY_D3D12_DRED = 'light'
    # The pipelines the title creates are written down in the cache folder, for `victory warm --used`.
    VICTORY_PIPELINE_BOOK = '1'
    # A compute program whose approximate wave64 form hung the card (docs/findings.md): its
    # pipeline is left out, in either build.
    VICTORY_SKIP_PIPELINES = '2561af9577c47289'
}
if ($Safe) {
    $settings.VICTORY_UNDER_WIDE = '0'; $settings.VICTORY_LAND_ECHO = '0'
} else {
    $settings.VICTORY_THIN = '1'; $settings.VICTORY_JOIN_PARTS = '8'; $settings.VICTORY_UPLOAD_AHEAD = '1'
    # A submission is cut behind a copy at its place that later lists read on the CPU, instead of waiting for
    # the flights: the wait 34.6 -> 2.5 ms a frame, the frame about 4 ms shorter (A/B run of 2026-11-28).
    $settings.VICTORY_PLACED_ORDER = '1'
    # Below 4K output, the interface on a 3840x2160 plane of its own (its menus are laid out in 4K pixels).
    $settings.VICTORY_UI_PLANE = '1'
    # Two presents in the line: the title records the next frame while ours sends this one
    # (avatarmanz/VIctory-7900XTX#379). Empty texture blocks kept follow the card's headroom
    # instead of up to 4 GiB of them on a 12 GB card (avatarmanz/VIctory-7900XTX#285).
    $settings.VICTORY_SUBMIT_FRAMES = '2'; $settings.VICTORY_SPARES_BY_HEADROOM = '1'
}
if ($RayTracing) { $settings.VICTORY_BVH_BUILDER = '1' }
if ($NvidiaDefaults -and $Safe) {
    Write-Host '-NvidiaDefaults is left out with -Safe.'
} elseif ($NvidiaDefaults) {
    # What VIctory-NVIDIA's launcher (appsictory\src\play.rs at 9ce3cf9) sets and this script does
    # not, of the switches this fork's code reads with another default. The saved settings and -More
    # still go over them.
    # Shader writes into card copies of write-combined title memory of 1 MiB and more stay on the
    # card: no compare, fetch or merge back.
    $settings.VICTORY_WC_NO_BACK = '1'
    # Title buffers from 64 KiB up get a copy in the card's memory (unset: sized from the budget).
    $settings.VICTORY_CARD_BUFFERS_MIN_KB = '64'
    # Textures picked by index: 16 ms a pass for making and staging them (250), no triple allowance
    # for a backlog, no blocks made ahead to age, uploads decided at submission and copied on the
    # list's prelude, and textures the host writes left out of the pass's queue.
    $settings.VICTORY_TEXTURE_STAGE_MS = '16'; $settings.VICTORY_TEXTURE_CATCHUP = '0'
    $settings.VICTORY_RIPE_BLOCKS = '0'; $settings.VICTORY_PRELUDE_UPLOADS = '1'
    $settings.VICTORY_QUEUE_HOST_TEXTURES = '0'
    # A later table read in the same frame takes a slot written after the frame's first read.
    $settings.VICTORY_TABLED_REREAD = '1'
    # The heaps' records: a submission's round begins as soon as it holds the submission lock, and
    # each thread is handed fewer pages at a time.
    $settings.VICTORY_HEAP_SYNC_FIRST = '1'; $settings.VICTORY_SYNC_FINE = '1'
    # The light stand-in: the air scatters at most 1.5 times the light around (a change of the
    # picture, not of speed).
    $settings.VICTORY_AMBIENT_AIR = '1.5'
    # Not taken: VICTORY_TEXTURE_ROOM_PERCENT with frame generation (VICTORY_FRAMEGEN_ROOM_MB keeps
    # that room here already), its memory switches (VICTORY_SLOT_EVICT, VICTORY_ALLOCATOR_TRIM,
    # VICTORY_BUDGET_PACK: unplayed here, give them with -More), VICTORY_LEAVE_OUT and the switches
    # this fork does not have.
    Write-Host '-NvidiaDefaults: the switches of VIctory-NVIDIA''s launcher are set as well.'
}
# The title takes its output size from its own settings (3840x2160 in a Series X dump), not from
# the display mode it is told of: VICTORY_OUTPUT patches it to one of the sizes it has a mode for.
$output = $Display
if ($Display -eq 'monitor') {
    # In real pixels (unlike Windows.Forms, which a window without DPI awareness gets scaled).
    $screen = Get-CimInstance Win32_VideoController | Where-Object CurrentHorizontalResolution | Select-Object -First 1
    $size = if ($screen) { "$($screen.CurrentHorizontalResolution)x$($screen.CurrentVerticalResolution)" }
    $output = if ($size -in $modes) { $size } else { '2560x1440' }
    if ($size -notin $modes) { Write-Host "This screen ($size) is no size the title has a mode for: $output." }
}
if ($output -ne 'console') { $settings.VICTORY_OUTPUT = $output }
# Frame generation from the saved settings that cannot run is left out (with a line saying so);
# given on the command line, the start stops.
$fgSaved = $FrameGen -and $fromSaved.FrameGen
if ($FrameGen -and $Dlss -eq 'off') {
    if ($fgSaved) { Skip-Saved 'VICTORY_FRAMEGEN' 'frame generation needs DLSS'; $FrameGen = $false } else {
        $where = if ($fromSaved.Dlss) { " DLSS is off in the saved settings ($savedFile)." } else { '' }
        Write-Host "-FrameGen needs -Dlss (frame generation alone stops the title).$where"; exit 1
    }
}
if ($FrameGen -and $Safe) {
    if ($fgSaved) { Skip-Saved 'VICTORY_FRAMEGEN' 'not with -Safe'; $FrameGen = $false } else {
        Write-Host '-FrameGen needs the line of submissions -Safe leaves out (VICTORY_THIN).'; exit 1
    }
}
if ($FrameGen -and $output -in 'console', '3840x2160') {
    # The frame without the interface is the scene plane, which a 3840x2160 output does not have.
    if ($fgSaved) { Skip-Saved 'VICTORY_FRAMEGEN' 'it needs an output smaller than 3840x2160'; $FrameGen = $false } else {
        $where = if ($fromSaved.Display) { " The output size is from the saved settings ($savedFile)." } else { '' }
        Write-Host "-FrameGen needs an output smaller than 3840x2160 (the interface on a plane of its own).$where"; exit 1
    }
}
if ($Dlss -ne 'off') {
    # DLSS sets the scene's size; the render scale below is left out then (said in the log), and
    # applies only if DLSS is not available (NVIDIA's files missing, another card).
    $settings.VICTORY_DLSS = '1'; $settings.VICTORY_DLSS_MODE = $Dlss
    if ($FrameGen) { $settings.VICTORY_FRAMEGEN = '1'; $settings.VICTORY_FRAMEGEN_X = $fgTimes }
}
if ($RenderScale -ne 'title') { $settings.VICTORY_RENDER_SCALE = $RenderScale }
$shown = if ($output -eq 'console') { 'the title''s own (3840x2160)' } else { $output }
$scale = if ($Dlss -ne 'off') {
    $fg = if (-not $FrameGen) { '' } elseif ($fgTimes -eq '0') { ', frame generation ready (off)' } else { ", frame generation ${fgTimes}x" }
    "DLSS $Dlss$fg"
} elseif ($RenderScale -eq 'title') { 'its own 2/3' } else { $RenderScale }
Write-Host "Output size: $shown; render scale: $scale"
# The other saved switches, over this script's own; an empty value removes one.
$options = 'VICTORY_DLSS', 'VICTORY_DLSS_MODE', 'VICTORY_FRAMEGEN', 'VICTORY_FRAMEGEN_X', 'VICTORY_OUTPUT', 'VICTORY_RENDER_SCALE'
foreach ($k in $saved.Keys) {
    if ($k -in $options) { continue }
    if ($k -in $More.Keys -or ($k -eq 'VICTORY_BVH_BUILDER' -and 'RayTracing' -in $given)) { Register-Saved $k $false; continue }
    if ($saved[$k] -eq '') { $settings.Remove($k) } else { $settings[$k] = $saved[$k] }
    Register-Saved $k $true
}
if ($saved.Count) {
    $kept = if ($overruled.Count) { "; the command line's kept over $($overruled -join ', ')" } else { '' }
    Write-Host "Saved settings (F1 menu, $savedFile): $($applied -join ', ')$kept. -NoSaved: without them."
}
foreach ($k in $More.Keys) { $settings[$k] = $More[$k] }

$victory = Join-Path $PSScriptRoot "..\target\$Build\victory.exe"
if (-not (Test-Path $victory)) { Write-Host "No $victory. Build it first (README.md)."; exit 1 }
if (-not (Test-Path $Exe)) { Write-Host "No $Exe."; exit 1 }
# (Other settings left over in this window go; where your files are stays.)
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

if ($Log -and -not $LogFile) {
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
        & $victory run $Exe --execute --console $Console 2>&1 | ForEach-Object {
            $text = "$_"
            $writer.WriteLine(($text -replace "$([char]27)\[[0-9;]*m", ''))
            $text
        }
    } else {
        & $victory run $Exe --execute --console $Console
    }
} finally {
    if ($writer) { $writer.Dispose() }
    Pop-Location
}
