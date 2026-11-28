# VIctory on an RTX 5070

A fork of avatarmanz's VIctory-7900XTX (no longer on GitHub), itself a
fork of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer that
runs an Xbox Series title natively on Windows from files the user supplies.
This one is about **speed on NVIDIA Blackwell, the GeForce RTX 5070**: the world
at a playable frame rate on a 12 GB card, without giving up the
picture. It is not there yet.

**This repository contains no game files, code or assets.** You need your own
copy of the game. It is not affiliated with or endorsed by the game's developer
or publisher.

## The target machine

Windows 11, AMD Ryzen 9 5900X (12 cores, 24 threads), 64 GB RAM, GeForce RTX 5070
12 GB.

What differs from the machine the parent fork was tuned on, and what this fork
has to plan around:

| | Parent fork (RX 7900 XTX) | This fork (RTX 5070) |
|---|---|---|
| GPU memory | 24 GB | **12 GB**: half the room for texture copies, render targets and card copies of title buffers |
| System memory | 64 GB | 64 GB (32 GB until 2026-11-26), room for the 35-46 GB of private memory the parent measured |
| Waves | AMD RDNA 3, wave32 and wave64 | NVIDIA, 32 lanes only: programs written for 64-lane waves are emulated |
| CPU | i9-12900K, 8 fast and 8 slow cores | Ryzen 9 5900X, 12 equal cores over two dies |
| Driver | AMD | NVIDIA: its own pipeline creation times and shader cache |

## Where it stands on the RTX 5070 (2026-11-27)

Medians of the frame times, of the 5-second frame lines with 300 draws a frame or
more. The first four rows are the same scene of the prologue each time
(2026-11-26); the last two are longer runs played by hand on 2026-11-27.

| | |
|---|---|
| The parent fork's defaults, 2560x1440 output | about 420 ms a frame |
| Copies of single buffer views found under a read lock, their buffers reused (18ae3bd) | 190 ms |
| On 12 GB cards, title buffers from 1 MiB up (up to 2 GB) kept on the card (a65c925) | 114 ms at 2560x1440, 96 ms at 1920x1080 |
| 2-minute runs at 1920x1080, `scripts\play.ps1` defaults | 112-138 ms (about 7-9 fps), 125 ms with patch 2 in the same runs |
| 10 minutes at 1920x1080, the night of 2026-11-26 to 27 | 103 ms (about 10 fps), best line 68 ms |
| 5 minutes at 1920x1080 after the first part of the VIctory-NVIDIA port (66f54796) | 105 ms, best line 64 ms; the card 41 ms, the wait for it 37 ms |

The ports of 2026-11-27 were fixes first: they did not move the median. The first
run of a new build is not one to measure: with the shader cache refilling and the
driver making pipelines, two such runs gave 380 and 713 ms.

The parent's defaults were tuned on a card whose driver makes buffers cheaply; the
NVIDIA driver takes about a millisecond for each, so anything that makes buffers
under a lock or per call costs whole frames here. The scene's own variation
(1,000 to 6,000 draws in 5 seconds) hides differences under about 10 %.

What sets the frame now, at 1920x1080:

- **One wait of 30-36 ms a frame on the submitting thread.** The title copies a few
  hundred bytes out of memory a dispatch still in flight writes, into constant
  buffers that draws of later lists read; those are filled on the CPU at
  submission, so the submission waits for the card. `VICTORY_PLACED_ORDER=1`, which
  `scripts\play.ps1` sets since 2026-11-28, ends the wait (34.6 -> 2.5 ms a frame in
  an A/B run), but about 22 ms of it come back as work of the same thread (the
  title's bytes put under flights' writes, copies in the card's memory), so the
  frame is only about 4 ms shorter (96 -> 92 ms).
- **The card itself**, about 37-41 ms a frame, the next wall after that.

New on this fork:

- The 12 GB card-buffer defaults above
  (`VICTORY_CARD_BUFFERS_MIN_MB=16 VICTORY_CARD_BUFFERS_MB=512` restores the parent's).
- On by default, with `=0` restoring the parent's way: `VICTORY_PLACED_CONSTANTS` and
  `VICTORY_PLACED_EXACT` (constant buffers and copies a copy earlier in the list
  writes, filled on the card), `VICTORY_TEXTURE_VOLATILE` (the intro video no longer
  jumps back in time), `VICTORY_TARGET_VARIANTS` (a pipeline per render-target
  format a draw really uses).
- **The console's 64-lane compute programs run on 32-lane waves** instead of being
  dropped: the card refuses the 64-lane form, so they take the wave-wide branches,
  and in groups of several console waves the two host waves of one console wave
  meet each other alone (`VICTORY_WAVE_WIDE=0`: not). One program whose
  approximate form hung the card is left out by `scripts\play.ps1`
  (`VICTORY_SKIP_PIPELINES`).
- A dispatch whose compute pipeline is not made yet waits for it with a budget of
  its own (`VICTORY_PIPELINE_COMPUTE_WAIT_MS`), so the title's GPU watchdog no
  longer ends a run at the intro's first frame.
- Without ray tracing, reflections are held to the light around
  (`VICTORY_AMBIENT_REFLECT`), and glass no longer sparkles or draws over what
  stands in front of it.
- Fixes and speed work ported by hand on 2026-11-27 from upstream VIctory,
  avatarmanz, fxceless, Belobug, Kelvaroq and VIctory-NVIDIA (which runs on the
  same card): shader translation fixes, placed views that no longer leak (16.7 GB
  after 70 minutes there), TVs, phone calls and reverb that are heard, and for
  `-RayTracing` the builder's memory. Not all of it has been played yet.
- The code only an AMD card ran is removed (the RDNA float forms, the 2^21-slot
  view heap).

## Where the parent fork stood (2026-11-26, RX 7900 XTX)

i9-12900K, 64 GB RAM, Radeon RX 7900 XTX, the Series X profile, ray tracing off.

| | |
|---|---|
| City street, Series S profile, renderer patches 1, 2, 5, 6 and 7 on | about 45 ms a frame (about 22 fps, 44.9 ms in run merge-5) |
| Same, patches 1, 2, 5 and 6 | 45.6 ms, p95 47 ms |
| Same, patches 1, 2 and 5 | 47.3 ms (about 21 fps), p95 49 ms |
| Series X profile, patches 1+2 | about 53 ms (about 19 fps) |
| Same scene before the patches | 57-75 ms (13-17 fps) |
| Shaders the title uses that translate | all of them (25,790 of 25,985; the rest are unused geometry shaders) |
| Memory | about 30 GB of private memory, 10.5 GB of VRAM: **64 GB of RAM recommended** for now |

It started at 280 ms a frame in the same scenes three days earlier. What sets
the frame now is the one thread that submits the title's work to the card
(about 40 ms); the card itself needs about 36 ms and now waits about 7 ms a frame with nothing queued.

### Its renderer patches (off by default while they are tested)

| Switch | What it does |
|---|---|
| `VICTORY_PORT_DESCRIPTORS=1` | The title's descriptor-table stores report the slots they write, instead of a scan of ~600,000 slots a frame |
| `VICTORY_OUTPUT=<w>x<h>`, `VICTORY_RENDER_SCALE` | The title's output size (it renders for 4K otherwise) and its upscaler's render scale |
| `VICTORY_UI_PLANE=1` | Below 4K output, the interface on a 3840x2160 plane of its own, scaled over the scene: the settings menu, subtitles and prompts no longer keep their 4K size (from avatarmanz pull request #295) |
| `VICTORY_FRAME_INTERVAL_MS` | The frame rate past the title's 30 fps, game clock unchanged |

The parent's patches 1 (the constants heap read in place), 5 (constant buffers
read as raw bytes), 6 (the write guards' shared lock) and 7 (the journal in parts,
heaps' records asked for at submit) are not in this fork. Measured on the
RTX 5070 at 1080p (2-minute runs, 2026-11-26; median ms a frame, baseline 138):
patch 1 258 and patches 1+5 206 (the card reads the title's constants over PCIe,
and patch 5 put no constant buffer in place in this build of the title); 6 and 7
showed no difference within the runs' noise. Patch 2 (125, its heap compare
4 -> 1 ms a frame) stays.

What was known to be wrong there:

- Trees, signs and far objects flicker in the distance in some places.
- Lighting is too bright or too dark after walking indoors, until the title has
  run for a while; rooms with little direct light are too dark without `-RayTracing`.
- Textures of what just came into view appear late while the title streams.
- The first run of a new build stutters while the driver makes pipelines
  (`victory warm` makes them ahead).
- Recording a frame in the world makes the title's own watchdog end it.

## How to run it

You need Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader
model 6.6 and mesh shaders, Rust (the toolchain is pinned), the MSVC build
tools, and your own dump of the title. With 32 GB of RAM, give Windows a large
paging file: the title needs several GiB of commit at startup, and Windows error
1455 in `crashes.log` means it ran out.

```powershell
git clone https://github.com/Rubenpannen/VIctory-RTX5070
cd VIctory-RTX5070
copy victory.example.toml victory.toml      # then set your paths, outside this folder
cargo build --locked --profile diag -p victory --features diagnostics
scripts\play.ps1 -Exe <path of your copy's executable>
```

`scripts\play.ps1 -Safe` leaves out the newest, faster ways; try it first when
something is wrong. `-RayTracing` turns the title's ray-traced indirect light on.
`-NvidiaDefaults` also sets the switches VIctory-NVIDIA's launcher sets and this
script does not (listed in the script with what each does; not measured on this PC yet).
The title renders for 2560x1440 at full scale by default (`VICTORY_OUTPUT=2560x1440`,
`VICTORY_RENDER_SCALE=1.0`; left alone it renders 2560x1440 and scales it up to
3840x2160). `-Display` picks the output size: `1920x1080`, `1664x936`, `1280x720`,
`3840x2160`, `monitor` (this PC's main screen, if the title has a mode for it) or
`console` (the title's own). `-RenderScale` picks the scale its upscaler renders at
(0.5 to 1.0, or `title` for its own 2/3). Below 4K the interface goes on a plane of
its own at 3840x2160 (`VICTORY_UI_PLANE=1`, not with `-Safe`), since the title
lays its menus out in 4K pixels. `-Console series-s` runs the Series S version.
`-Log` also writes the whole log to `<data_dir>\logs\play-<date>-<time>.log`
(`-LogFile <path>` elsewhere, outside this folder: it holds the title's data).

On a hybrid-graphics PC, the runtime and offline pipeline checks prefer a
high-performance adapter with Shader Model 6.6. The runtime logs its adapter
name, PCI vendor and device IDs, and dedicated VRAM. Set
`VICTORY_D3D12_ADAPTER` to a zero-based DXGI index or part of the adapter's
name to select it explicitly. With the launcher, pass
`-More @{VICTORY_D3D12_ADAPTER='NVIDIA'}`.

Before the first run, ahead of time (both can be stopped and go on later):

```powershell
target\diag\victory.exe shaders <the shader archives>   # every shader translated, minutes
target\diag\victory.exe warm --used                     # pipelines earlier runs drew with, for a new driver or build
```

Commands, debugging switches and upstream's own description:
[docs/README-fxceless.md](docs/README-fxceless.md). Every `VICTORY_...` switch is
listed in [SWITCHES.md](SWITCHES.md) with its default and values, and described
in a comment where it is read.

## What this fork inherits

From the RX 7900 XTX fork; the reasons and the measurements, all of that card,
are in [docs/findings.md](docs/findings.md). Of its renderer patches only patch 2
paid on the RTX 5070; patches 1, 5, 6 and 7 were measured and removed (above).

- **List calls are written down, not carried out, on the title's threads**
  (`VICTORY_THIN`): eight threads of ours translate them.
- **Submissions are prepared by a thread of ours and joined**
  (`VICTORY_ASYNC_SUBMIT`, `VICTORY_JOIN_PARTS`), with what can be spread over
  threads spread: refreshes, compares (AVX2), descriptor records, landings.
- **What shaders wrote comes back beside the write guards**, by the dwords that
  changed, on a thread of its own (`VICTORY_RELAY`, `VICTORY_LAND`).
- **Buffer fills that need bytes still on the card are done on the card**
  (`VICTORY_CARD_STEPS`) instead of waiting for it.
- **The title's writes are caught more cheaply**: one view opened at a fault,
  blocks that rest, places in the title's code that look before they write.
- **Textures go up on a copy queue of their own** and a slot shows one only after
  its upload ran, which also ended a device loss that came every 15 minutes or so.
- **Boats float**: textures that shaders write and the title's CPU reads go back
  to title memory (from fxceless and Geckomacabre, on by default).
- **Frame record and replay** with per-pass times and shader variants compared
  turn by turn ([docs/replay.md](docs/replay.md)).

## Credits

This is a tree of other people's work with more on top. The history is kept, and
commits taken from elsewhere name the commit and its author.

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors: the loader, the runtime services, the Direct3D 12 back end and
  the shader recompiler.
- **avatarmanz/VIctory-7900XTX** (no longer on GitHub): the fork this one
  started from, and everything listed under "What this fork inherits".
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)**: the fork
  that one started from, and the people whose pull requests it carries.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes,
  cube gradients, sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**:
  the ambient light idea, pipeline workers, and work by romainhedouin.
- **[trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA)**
  (on foxhound's tree, by codec_d and readmet3xt-lab): fixes and speed work for
  the same card, ported by hand; each commit names its source commits and authors.
- **Kelvaroq**: fence and texture-range fixes, and the idea behind the mesh launch stage.
- **[FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)**.
- **[myokex/victory-amd](https://github.com/myokex/victory-amd)**: the guard over
  every view of a page (with Spexivus, joroyt and tightfall), and Joshua Odom's
  reads into title memory.
- **Geckomacabre** and **cxy0072019-sketch**: the texture read-back.
- **[XCL](https://github.com/ivall/xcl)** (GPL-2.0-only) was read for how the same
  title behaves under another layer; no code of it is here.
- This fork's changes were made with [Claude Code](https://claude.com/claude-code).

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE); the one file that is not
covered is named in [docs/asset-policy.md](docs/asset-policy.md).
