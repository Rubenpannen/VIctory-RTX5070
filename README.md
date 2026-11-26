# VIctory on an RTX 5070

A fork of [VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX), itself a
fork of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer that
runs an Xbox Series title natively on Windows from files the user supplies.
This one is about **speed on NVIDIA Blackwell, the GeForce RTX 5070**: the world
at a playable frame rate on a 12 GB card in a 32 GB PC, without giving up the
picture. It is not there yet.

**This repository contains no game files, code or assets.** You need your own
copy of the game. It is not affiliated with or endorsed by the game's developer
or publisher.

## The target machine

Windows 11, AMD Ryzen 9 5900X (12 cores, 24 threads), 32 GB RAM, GeForce RTX 5070
12 GB. No measurements of this fork on it yet; the figures below are those of the
fork it started from.

What differs from the machine the parent fork was tuned on, and what this fork
has to plan around:

| | Parent fork (RX 7900 XTX) | This fork (RTX 5070) |
|---|---|---|
| GPU memory | 24 GB | **12 GB**: half the room for texture copies, render targets and card copies of title buffers |
| System memory | 64 GB | **32 GB**, where the parent measured 40-46 GB of private memory |
| Waves | AMD RDNA 3, wave32 and wave64 | NVIDIA, 32 lanes only: programs written for 64-lane waves are emulated |
| CPU | i9-12900K, 8 fast and 8 slow cores | Ryzen 9 5900X, 12 equal cores over two dies |
| Driver | AMD | NVIDIA: its own pipeline creation times and shader cache |

## Where the parent fork stood (2026-11-26, RX 7900 XTX)

i9-12900K, 64 GB RAM, Radeon RX 7900 XTX, the Series X profile, ray tracing off.

| | |
|---|---|
| City street, renderer patches 1+2 on | about 53 ms a frame (about 19 fps) |
| Same scene before the patches | 57-75 ms (13-17 fps) |
| Shaders the title uses that translate | all of them (25,790 of 25,985; the rest are unused geometry shaders) |
| Memory | about 35 GB of private memory: **64 GB of RAM recommended** for now |

It started at 280 ms a frame in the same scenes three days earlier.

### Its renderer patches (off by default while they are tested)

| Switch | What it does |
|---|---|
| `VICTORY_PORT_CONSTANTS=1` | The title's 174 MB constants heap is read by the card where the title writes it: no uploads, no fills |
| `VICTORY_PORT_DESCRIPTORS=1` | The title's descriptor-table stores report the slots they write, instead of a scan of ~600,000 slots a frame |
| `VICTORY_PORT_CBV=1` | Translated shaders read constant buffers in place as raw bytes |
| `VICTORY_OUTPUT=<w>x<h>`, `VICTORY_RENDER_SCALE` | The title's output size (it renders for 4K otherwise) and its upscaler's render scale |
| `VICTORY_FRAME_INTERVAL_MS` | The frame rate past the title's 30 fps, game clock unchanged |

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
The title renders for 2560x1440 at full scale by default (`VICTORY_OUTPUT=2560x1440`,
`VICTORY_RENDER_SCALE=1.0`; left alone it renders 2560x1440 and scales it up to
3840x2160). `-Display` picks the output size: `1920x1080`, `1664x936`, `1280x720`,
`3840x2160`, `monitor` (this PC's main screen, if the title has a mode for it) or
`console` (the title's own). `-RenderScale` picks the scale its upscaler renders at
(0.5 to 1.0, or `title` for its own 2/3).
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
described in a comment where it is read.

## What this fork inherits

From the RX 7900 XTX fork; the reasons and the measurements, all of that card,
are in [docs/findings.md](docs/findings.md). Which of them pay on the RTX 5070 is
still to be measured.

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
- **[avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX)**:
  the fork this one started from, and everything listed under "What this fork
  inherits".
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)**: the fork
  that one started from, and the people whose pull requests it carries.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes,
  cube gradients, sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**:
  the ambient light idea, pipeline workers, and work by romainhedouin.
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
