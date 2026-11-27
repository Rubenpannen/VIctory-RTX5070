# Smoke check of the victory app without starting the title: build, CLI answers, config,
# a read-only look at the title's executable, and the asset-policy check.
#   powershell -ExecutionPolicy Bypass -File .claude\skills\run-victory\smoke.ps1 [-NoBuild] [-Exe <title exe>] [-Policy]
# -Exe defaults to the executable named in <game_dir>\MicrosoftGame.config, if game_dir is set
# (victory.toml or VICTORY_GAME_DIR). Nothing is written inside the repository.
param(
    [switch]$NoBuild,
    [string]$Exe = '',
    [switch]$Policy
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$victory = Join-Path $root 'target\diag\victory.exe'
$failed = @()

function Invoke-Step([string]$name, [scriptblock]$body) {
    Write-Host "== $name" -ForegroundColor Cyan
    & $body
    if ($LASTEXITCODE -ne 0) {
        Write-Host "   FAILED (exit $LASTEXITCODE)" -ForegroundColor Red
        $script:failed += $name
    }
}

Push-Location $root   # victory.toml is looked for in the current folder
try {
    if (-not $NoBuild) {
        Invoke-Step 'build diag' { cargo build --locked --profile diag -p victory --features diagnostics }
    }
    if (-not (Test-Path $victory)) { Write-Host "No ${victory}: build first."; exit 1 }

    Invoke-Step 'version' { & $victory --version }
    Invoke-Step 'config' { & $victory config }

    if (-not $Exe) {
        $game = $env:VICTORY_GAME_DIR
        if (-not $game -and (Test-Path victory.toml)) {
            $line = Select-String -Path victory.toml -Pattern '^\s*game_dir\s*=\s*[''"](.+)[''"]' |
                Select-Object -First 1
            if ($line) { $game = $line.Matches[0].Groups[1].Value }
        }
        $cfg = if ($game) { Join-Path $game 'MicrosoftGame.config' }
        if ($cfg -and (Test-Path $cfg)) {
            $m = Select-String -Path $cfg -Pattern '<Executable Name="([^"]+)"' | Select-Object -First 1
            if ($m) { $Exe = Join-Path $game $m.Matches[0].Groups[1].Value }
        }
    }
    if ($Exe) {
        Invoke-Step 'inspect title exe' { & $victory inspect $Exe }
    } else {
        Write-Host '== inspect skipped: no -Exe and no game_dir with a MicrosoftGame.config'
    }

    if ($Policy) { Invoke-Step 'asset policy' { cargo xtask policy } }
} finally {
    Pop-Location
}
if ($failed) { Write-Host "FAILED: $($failed -join ', ')" -ForegroundColor Red; exit 1 }
Write-Host 'smoke OK' -ForegroundColor Green
