# DLSS and DLSS frame generation

NVIDIA DLSS can take the title's upscaling, and DLSS frame generation can add frames between the
ones it renders, through NVIDIA's Streamline library. Both are off by default. Ported by hand from
VIctory-NVIDIA (readmet3xt-lab's fc8b9ca, d11233a after DexNom's fork, 4fcaebd, b8aadfa, 24d6bbb,
1d0fa87, 8a93538, and trustno1-coder's 22c6be2 and 64d4931); its FSR 3 frame generation, F1 menu
and 1440p image are not.

Nothing of NVIDIA's is in this repository or needed to build. The DLLs are loaded when the title
runs; if anything is missing (the folder, NVIDIA's signature on a file, the feature on this card),
the log says what, and the title upscales and presents as it does without them.

## Setup

1. Get NVIDIA's files (about 70 MB from NVIDIA's own GitHub, Streamline v2.14.16, under NVIDIA's
   licences, which are saved beside them):

   ```powershell
   scripts\get-nvidia-sdks.ps1
   ```

   They go into `<data_dir>\nvidia-sdk\Streamline\bin\x64`, where victory looks for them
   (`data_dir` from `victory.toml` or `VICTORY_DATA_DIR`, else `%LOCALAPPDATA%\victory\data`).
   Elsewhere: `-To <folder>`, then `VICTORY_STREAMLINE_DIR=<folder>\Streamline\bin\x64`. The
   script refuses a folder inside the repository. A newer `nvngx_dlss.dll` (NVIDIA's DLSS SDK)
   can replace the one there.
2. Check the PC without the title (a few seconds each):
   - `victory upscaler-check`: Streamline loads, and what this card supports.
   - `victory upscaler-check --device`: our D3D12 device is handed to Streamline, DLSS's render
     sizes for the output are listed, and DLSS runs three times on stand-in textures.
   - `victory upscaler-check --present`: as `--device`, then three seconds of frame generation in
     a window of ours, with how many frames it showed.
3. Play with `scripts\play.ps1 -Dlss quality` (or `dlaa`, `balanced`, `performance`), and with
   frame generation `-Dlss quality -FrameGen` (`-FrameGenX 2` to `6`). The in-game F1 menu
   sets DLSS, its mode and frame generation for the next start, and the frames shown per frame
   rendered (2x to 6x) at once while frame generation runs.

Needed: a GeForce RTX card (frame generation: RTX 40 or later), Windows 10 2004 or later,
hardware-accelerated GPU scheduling on (frame generation needs it).

Before the DLLs are loaded, each `sl.*.dll` and `nvngx_*.dll` in the folder must carry a valid
Authenticode signature by NVIDIA Corporation; one that does not stops Streamline from being used.

## Switches

All are in `SWITCHES.md`; the ones that matter here:

| Switch | Default | What |
|---|---|---|
| `VICTORY_DLSS=1` | off | DLSS takes the title's upscaling |
| `VICTORY_DLSS_MODE` | `quality` | `dlaa`, `quality`, `balanced`, `performance`, `ultraperformance`: the scene at 1, 2/3, 0.58, 1/2, 1/3 of the output |
| `VICTORY_DLSS_PRESET` | `default` | the DLSS model: `default`, `e`, `f`, `j`, `k`, `l`, `m` |
| `VICTORY_DLSS_REACTIVE=0` | on | the title's reactive mask not given to DLSS |
| `VICTORY_FRAMEGEN=1` | off | frame generation; needs DLSS, `VICTORY_THIN=1` (or `VICTORY_ASYNC_SUBMIT=1`), and `VICTORY_UI_PLANE=1` at a `VICTORY_OUTPUT` smaller than 3840x2160 |
| `VICTORY_FRAMEGEN_X` | 2 | frames shown per frame rendered, 2 to 6 |
| `VICTORY_FRAMEGEN_HUDLESS=0` | on | the scene plane not tagged as the frame without the interface |
| `VICTORY_FRAMEGEN_ROOM_MB` | 1024 | MiB the texture room line is lowered by while frame generation is ready |
| `VICTORY_STREAMLINE_DIR` | see above | the folder with the DLLs |
| `VICTORY_STREAMLINE_PRESENT=1` | off | DLSS without frame generation: our swap chain is Streamline's proxy, so it sees the presents |
| `VICTORY_FSR_PATH=1` | off | the title's FidelityFX path at `VICTORY_DLSS_MODE`'s scale with its own FSR, no Streamline: DLSS's comparison |

Diagnostics builds only: `VICTORY_DLSS_JITTER` and `VICTORY_DLSS_MV` (`x,y` multipliers, `1,1`:
`-1,1` flips x, for a sign fix), `VICTORY_STREAMLINE_LOG` (a folder for Streamline's own log),
`VICTORY_FRAMEGEN_CYCLE` (every N presents 2x, 4x, off, 6x in turn) and `VICTORY_CHECK_TEARDOWN=0`
(`upscaler-check` without the shutdown at its end).

## How it works

- **The title's FidelityFX path.** The title imports `ffxCreateContext`, `ffxDestroyContext`,
  `ffxConfigure`, `ffxQuery` and `ffxDispatch` from `amd_fidelityfx_loader_xbox.dll`. With DLSS
  available, the executable's imports of it are bound to ours
  (`victory_hle::modules::fidelityfx`), which pass everything to the title's DLL except an
  upscale `ffxDispatch` (type 0x10001): that goes to `victory_gpu::streamline::upscale`, which
  puts a step into the title's command list at that place and records `slEvaluateFeature` into
  the host list with the dispatch's colour, depth, motion vectors, exposure, reactive mask and
  output. The title's own upscaler stays made and runs on any frame DLSS does not take.
- **Series X takes another path by itself.** Its renderer picks the title's own temporal pass
  (6); only Series S picks FidelityFX (7). When DLSS is available (and only then),
  `victory_game::resolution` patches the picker to 7, the FidelityFX path's sizing to note the
  mode's FidelityFX quality and take its scaled branch at the mode's scale, and the temporal
  scale to the same; each place is compared byte for byte first, and if one differs none is
  patched. `VICTORY_RENDER_SCALE`, `VICTORY_FFX_SCALE` and `VICTORY_UPSCALER` are left out then.
- **The title's FSR4 at the shipping sizes.** The title's FSR4 only makes its upscaler at
  1280x720 in and 1920x1080 out; its context is made at those sizes whatever the title asks, or
  it divides by zero at its first upscale (4fcaebd).
- **The camera.** The title passes no camera matrices: the projection is made of the field of
  view, aspect and depth range, and the previous frame's clip is the identity (the motion is in
  the motion vectors). DLSS reads none of them; frame generation may.
- **Frame generation.** Our device is handed to Streamline, the direct queue and our swap chain
  are made through its proxies (manual hooking: nothing is hooked globally). The depth and
  motion vectors of the last upscaling submitted before a present are tagged for it, on the
  line's thread. The frame without the interface is the scene plane of `VICTORY_UI_PLANE`; the
  interface plane (3840x2160 over a smaller output) is not tagged. A present without a new
  upscaling (menus, loads) switches frame generation off for it. Alt+Enter is not taken by the
  system for the window while frame generation runs.

## Limits

- Frame generation wants 60 or more frames rendered a second; VIctory-NVIDIA measured 26-30 on an
  RTX 5070 at 1440p with DLSS Quality. It adds latency.
- The camera matrices are made up, so generated frames may be worse in camera motion than in a
  game that passes its own. Motion vectors are wrong inside character silhouettes and in heat
  haze.
- No HDR with frame generation.
- Frame generation alone (without DLSS) is refused: in VIctory-NVIDIA it stopped the title at
  1440p.
- The process may fail fast at its exit (0xC0000409): Streamline is shut down when `victory run`
  returns, not on the title's own exit, which ends the process at once while its threads may
  still be presenting.
- Measured on the RTX 5070 here: nothing yet.
