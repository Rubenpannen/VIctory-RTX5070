---
name: run-victory
description: Build, smoke-test and check the victory app (the Xbox Series compatibility layer) on Windows. Use when asked to build victory.exe, run or start victory, check that a change still builds and answers, inspect the title's executable, or run the asset-policy check, without starting the title.
---

# Run victory

`victory.exe` (in `apps/victory`) is a CLI that loads an Xbox Series title's executable into
its own process and runs it on host D3D12. The agent path is the smoke script
`.claude/skills/run-victory/smoke.ps1`. It builds the diag binary, checks that the CLI
answers, prints the resolved config, runs a read-only `inspect` of the title's executable when
one is known, and can run the asset-policy check. **It never starts the title.** Paths here are
relative to the repo root. Commands are PowerShell on Windows (the project is
`x86_64-pc-windows-msvc` only).

## Prerequisites

- MSVC build tools; the Rust toolchain comes from `rust-toolchain.toml`.
- A copy of the title, if `inspect` should run: set `VICTORY_GAME_DIR` (or `game_dir` in a
  git-ignored `victory.toml`, copied from `victory.example.toml`). It must be **outside** the
  repo (`docs/asset-policy.md`). Without it the smoke script skips `inspect`.

## Build

```powershell
cargo build --locked --profile diag -p victory --features diagnostics
```

This makes `target\diag\victory.exe`, the build all measurements are of. A rebuild after
changes in `victory-gpu`/`victory-shader` took **8-10 minutes** (thin LTO, one codegen
unit), so run it in the background with a 10-minute timeout. The one warning it gives
(`method get is never used`, `victory-gpu/src/resource.rs`) is expected.

## Run (agent path): the smoke script

```powershell
powershell -ExecutionPolicy Bypass -File .claude\skills\run-victory\smoke.ps1 -NoBuild -Policy
```

Without a game folder this printed `version`, `config` and `asset policy: <n> files ok`, then
`smoke OK` with exit 0. Leave out `-NoBuild` to build first.

With the title's folder set, it finds the executable from `<game_dir>\MicrosoftGame.config`
and prints `victory inspect` (PE headers, sections, TLS, imports, about 100 lines):

```powershell
$env:VICTORY_GAME_DIR='<your game folder>'; $o = powershell -ExecutionPolicy Bypass -File .claude\skills\run-victory\smoke.ps1 -NoBuild; "exit $LASTEXITCODE"; $o | Select-Object -Last 3
```

`-Exe <path>` names an executable directly. The script exits 1 and lists the failed steps if any
step fails.

## Starting the title (not covered)

`scripts\play.ps1 -Exe <title exe>` starts the title with this fork's switches (see
`CLAUDE.md`). It opens a game window and needs the GPU and many GB of VRAM. **It has not been
run from this skill**: the owner asked to leave launching out, so nothing here drives or
screenshots the running title.

## Gotchas

- **Don't pipe the script into `Select-Object -First N`.** Stopping the pipeline early made
  PowerShell report exit 255 / -1 although every step passed. Capture the output into a
  variable and take `-Last N`, as above.
- `victory.toml` is looked for in the **current folder**. The script `Push-Location`s to the
  repo root, so it works from any folder. A bare `target\diag\victory.exe config` run elsewhere
  reports `config file : (none)`.
- `cargo xtask policy` checks the tracked files only. New untracked files are first checked at
  commit time by the pre-commit hook (`cargo xtask policy --staged`).
- `inspect` prints the title's PDB path and section layout: fine for a terminal, but **never save
  that output in the repo** (asset policy).
- In PowerShell strings, `"$var:"` is a parse error (it reads as a drive-qualified variable).
  Write `"${var}:"`.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `Variable reference is not valid. ':' was not followed by a valid variable name character` | `"$name:"` inside a string: use `"${name}:"`. |
| `== inspect skipped: no -Exe and no game_dir with a MicrosoftGame.config` | Set `VICTORY_GAME_DIR` or pass `-Exe`. |
| Smoke step output ends early and exit is 255 / -1 | You piped into `Select-Object -First`; capture the output instead. |

## Tests

`cargo test --workspace --locked` needs no game data. `CLAUDE.md` has the CI variants. They are
not part of this skill's verified path.
