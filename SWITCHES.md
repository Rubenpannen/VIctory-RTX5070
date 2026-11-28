# VICTORY switches

Every `VICTORY_*` environment variable the code of this fork reads (crates/ and apps/), with what it
does, its default (what applies when the variable is not set) and the values it takes. Generated
on 2026-11-27 from the code, phase 1 of the VIctory-NVIDIA port included. Where this file and the
comment where a switch is read disagree, the comment is right.

- **Diagnostics build only**: the switch exists only in a build with `--features diagnostics`
  (`target\diag`); a release build ignores it.
- **Set by `scripts\play.ps1`**: the launcher sets it (some only with `-Safe`, `-RayTracing`,
  `-Display` or `-RenderScale`); `-More @{...}` overrides it.
- **Test only / build time**: read only by tests, or stamped in when building; not a switch to set
  when playing.

| Name | Description | Default | Possible values |
|---|---|---|---|
| `VICTORY_AB` | Flips the listed switches on and off by turns while the title runs, measuring both settings in one scene. *Diagnostics build only.* | off | &lt;seconds, number > 1>,&lt;switch name>,... (1 to 8 switches); the named switches flip every period |
| `VICTORY_ALIAS_COPY` | Lets a copy aimed at a texture that views another texture's memory reuse that texture instead of making a new one. | on | 0 = off; anything else = on |
| `VICTORY_ALLOCATOR_RENEW` | Replaces a host command list's allocator with a fresh one at reset once it recorded more than this many calls. | 512 | &lt;calls, integer>; 0 = never renew |
| `VICTORY_ALLOCATOR_TRIM` | Replaces a host list's (and its prelude's) command allocator at reset once it is `VICTORY_ALLOCATOR_TRIM_S` old, destroys idle pooled lists not taken for that long, and frees spare scratch chunks nobody took, so memory a heavy frame's commands took is given back. | off | 1 = on; unset/other = off |
| `VICTORY_ALLOCATOR_TRIM_S` | Sets the age, in seconds, at which `VICTORY_ALLOCATOR_TRIM` replaces an allocator or destroys an idle list. | 30 | &lt;seconds, integer>, clamped to 5-3600; needs VICTORY_ALLOCATOR_TRIM=1 |
| `VICTORY_AMBIENT` | Turns on the stand-in for ray-traced indirect light, estimated from the frame or a constant level, while the ray-tracing builder is off. *Set by `scripts\play.ps1`.* | off | &lt;level, number> = estimate from the frame; constant:&lt;level> = fixed level; unparseable = off |
| `VICTORY_AMBIENT_AIR` | Caps how much the air scatters light in the ambient stand-in, as a multiple of the light around. *Set by `scripts\play.ps1`.* | the title's own | &lt;multiple, number > 0>; 0 or unparseable = the title's own |
| `VICTORY_AMBIENT_REFLECT` | Caps how bright a reflection may get in the ambient stand-in, as a multiple of the light around. | built-in cap | &lt;multiple, number>; 0 or off = the title's own |
| `VICTORY_AMBIENT_SETTLE` | Sets how slowly the ambient light's running average follows the frames, so light does not pop in. | 250 ms, jump factor 3, fresh 0 s | &lt;time constant ms>[,&lt;jump factor>[,&lt;fresh seconds>]]; 0 = old fixed per-frame blend |
| `VICTORY_AMBIENT_SHADE` | Sets the strength and radius of the darkening in creases and corners added by the ambient stand-in. | strength 1.0, radius 0.6 m | &lt;strength>,&lt;radius in metres>; either part may be omitted |
| `VICTORY_ASK_EARLY` | Makes a thread ask the kernel about written title memory between a submission's own askings, at a fixed interval. | off | &lt;ms, number> between rounds; unparseable = 2 ms; 0 = off |
| `VICTORY_ASK_NORESET` | Walks title pages for writes without the kernel's reset and resets only the pages found written. | off | 1 = on; unset/other = off |
| `VICTORY_ASK_ONCE` | Asks the kernel once, before the first refresh, about every range a submission's lists refresh, instead of per range. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_ASK_SPREAD` | Splits a submission's written-memory askings over several threads at once. | off | 1 = on; unset/other = off |
| `VICTORY_ASYNC_SUBMIT` | Prepares and sends submissions on a thread of ours, in order, instead of on the title's submitting thread. | off (on when thin lists are wanted) | 1 = on; unset/other = off |
| `VICTORY_AUDIO_CAPTURE` | Records what the title plays to a WAV file (48 kHz stereo 16-bit) through process loopback. *Diagnostics build only.* | off | &lt;path of .wav file> = record to it; unset = off |
| `VICTORY_AUDIO_CAPTURE_OF` | Internal: tells the audio recorder child process which process id to record. *Diagnostics build only.* | not set (set by us for the recorder) | &lt;process id, integer> of the process to record |
| `VICTORY_B` | Placeholder switch name used only in the test of the A/B switch parser; no effect. | not set (test only) | no effect; name used in a parser test |
| `VICTORY_BAKE_EXACT` | Makes a draw of the title's bake see textures picked by index as title memory holds them at that moment. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_BAKE_TEXTURES_MS` | Sets the milliseconds a frame the texture-upload pass may take, during a bake, before a shader that picks by index asks. | 1500 ms | &lt;milliseconds, integer> |
| `VICTORY_BARRIER_TEST` | Timing experiment that drops some of the title's or our barrier waits for earlier shader writes; can give wrong pictures. *Diagnostics build only.* | off | groups = drop shader-write waits of barrier groups; indirect = drop the two waits of the indirect pass; both = both; other = off |
| `VICTORY_BATCH_POOL` | Lets command lists share the card scratch buffers of their batched indirect clamps, taking one when first needed and giving all back on reset, instead of each list keeping its own. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_BCPACK_CASES` | Test only: directory of packed BCPack chunks with reference outputs to compare the decoder against. | not set (test only) | &lt;directory path> of packed chunks with .out references |
| `VICTORY_BCPACK_TABLES` | Test only: the BCPack format table file used when comparing against reference outputs. | not set (test only) | &lt;file path> of the BCPack table file |
| `VICTORY_BENCH_CS` | Benchmark test only: times a different compute shader source for the card-compare pass. | not set (test only) | &lt;file path> of alternative compute shader source |
| `VICTORY_BENCH_FILE` | Benchmark test only: reads this file unbuffered to time what a store of detiled texture copies on disk would cost. | not set (test only) | &lt;file path> read unbuffered to time disk reads |
| `VICTORY_BENCH_HOLES` | Benchmark test only: views of watched memory without the holes the real title has. | on (test only) | 0 = views without holes; anything else = holes |
| `VICTORY_BENCH_N` | Benchmark test only: how many runs of each way of making new texture blocks. | 3 (test only) | &lt;runs per way, integer> |
| `VICTORY_BENCH_S` | Benchmark test only: how many seconds each measurement runs. | 5, or 30 in the block test (test only) | &lt;seconds, integer> |
| `VICTORY_BENCH_THREADS` | Benchmark test only: how many threads make texture blocks while long dispatches read them. | 4 (test only) | &lt;threads, integer> |
| `VICTORY_BENCH_WARP` | Benchmark test only: runs the card measurements on the software (WARP) adapter. | off (test only) | any value = use the software adapter; unset = off |
| `VICTORY_BETWEEN_POOL` | Uses the card buffers of copies done at their place in a list (and of constant buffers filled behind them) again in later lists, instead of making a new one each time. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_BIND_CACHE` | Skips setting render targets and depth on the host list again when they are the same as already set. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_BLOCK_WAY` | Benchmark test only: which way of making new texture blocks to run (one per process). | not set (test only) | one of: made; waited; enqueued; fresh-made; fresh-waited; fresh-enqueued; never |
| `VICTORY_BOOK` | Keeps a log of descriptor and memory ownership to name the cause of a device loss. *Diagnostics build only.* | on, but only active when device-removed data or checks are on | 0 = off; anything else (or unset) = on |
| `VICTORY_BUDGET_PACK` | Makes a list's mapped indirect budget a 256-byte slot of a shared 64 KiB buffer instead of a 64 KiB buffer of its own. | off | 1 = on; unset/other = off |
| `VICTORY_BUFFERS_AHEAD` | Has a low-priority thread make this many 64 KiB buffers (CPU-mapped, and card ones once wanted) ahead of the title's need, keeping up to 1024 in store afterwards. | 0 (none; every buffer is made when asked for) | &lt;count, integer>, at most 16384 |
| `VICTORY_BURY_LAG` | Sets how many submissions later than possible a released object is dropped and a run host list is reset. | 16 on a PC with limited RAM, else 64 | &lt;submissions, integer>; 0 = no lag |
| `VICTORY_BVH_BOUNDS` | Keeps the ray-tracing structure builder's accesses near their root views and treats an impossible size in a header as none. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_BVH_BUILDER` | Lets the translated ray-tracing structure builder run, so rays can find geometry. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/other = off |
| `VICTORY_BVH_FENCES` | Puts the structure builder's buffer atomics between two fences in translated shaders. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_BVH_QUARANTINE` | Keeps what the structure builder stores in host copies and does not copy it back into title memory. | off | 1 = on; unset/other = off |
| `VICTORY_BVH_SIZES_WAIT` | Waits for and lands a submission that asks for structure sizes before the title goes on. | off | 1 = on; unset/other = off |
| `VICTORY_BVH_TRACE` | Logs the structure sizes the builder is asked for and when title memory holds them. *Diagnostics build only.* | off | 1 = on; unset/other = off |
| `VICTORY_CACHE_CORES` | On a processor with one class of cores, treats the cores with the largest cache as the fast ones for our threads. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_CACHE_DIR` | Folder for derived data such as the shader cache; must lie outside the repository. | the config file's value, else the default cache folder | &lt;folder path>; must be outside the repository; empty = unset |
| `VICTORY_CALL_PARTS` | Splits the time of the costliest list calls (draws, dispatches, indirect executions) into parts on per-frame log lines, with the locks those calls take. *Diagnostics build only.* | on | 0 = off; anything else (or unset) = on |
| `VICTORY_CAPTURE` | Writes presented frames to PNG files in this folder, for frames numbered a power of two, to see what reaches the screen. *Diagnostics build only.* | off | &lt;folder path>; frames with a power-of-two number are written there as PNG |
| `VICTORY_CAPTURE_FRAMES` | Also saves these frame numbers as PNG and traces their draws at debug level. *Diagnostics build only.* | none | &lt;frame numbers, comma-separated> |
| `VICTORY_CAPTURE_WORLD` | Also captures the 2nd and 6th frames after the first frame with 500 or more draws. *Diagnostics build only.* | off | any value = on; unset = off |
| `VICTORY_CAPTURE_WORLD_ALSO` | Captures more frames counted from the first world frame, besides the default ones. *Diagnostics build only.* | none | &lt;frame offsets, comma-separated> |
| `VICTORY_CARD_ACCOUNTING` | Prints a line saying how the card's memory is used by texture slots when it has no room. | off | 1 = on; unset/other = off |
| `VICTORY_CARD_ALL` | Gives every title buffer a copy in the card's memory as well, however small. | off | 1 = on; unset/other = off |
| `VICTORY_CARD_AWAKE` | Keeps the card's clocks up between submissions so it does not idle during frames. *Set by `scripts\play.ps1`.* | off | 1 = on; turns = alternate on and off; other = off |
| `VICTORY_CARD_BUFFERS` | Keeps host copies of big title buffers in the card's own memory. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_CARD_BUFFERS_MB` | Sets the largest title buffer, in MiB, that gets a copy in the card's memory. | chosen from the adapter's VRAM budget | &lt;MiB, integer>; largest buffer given a card copy |
| `VICTORY_CARD_BUFFERS_MIN_KB` | Sets the smallest title buffer, in KiB, that gets a card-memory copy; can go under 1 MiB. *Set by `scripts\play.ps1`.* | not set (MIN_MB or the VRAM budget decides) | &lt;KiB, integer > 0>; smallest buffer given a card copy |
| `VICTORY_CARD_BUFFERS_MIN_MB` | Sets the smallest title buffer, in MiB, that gets a copy in the card's memory. | chosen from the adapter's VRAM budget | &lt;MiB, integer>; smallest buffer given a card copy |
| `VICTORY_CARD_HASH_LEN` | Limits the card's hashes of a buffer copy to the title buffer's own size rounded to 16 bytes. | on | 0 = hash the whole card buffer; anything else (or unset) = own size rounded to 16 bytes |
| `VICTORY_CARD_REHASH_RUNS` | Hashes pieces re-hashed after ranges are sent up in runs of touching pieces, so each is hashed once. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/other = off |
| `VICTORY_CARD_ROOM_FOR_BUFFERS` | Makes the texture queue keep room on the card for card copies of title buffers not yet made. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_CARD_ROOM_FOR_TARGETS_MB` | Sets how much card memory the texture queue leaves free for render targets and other not-yet-made host copies. | 3072 MiB | &lt;MiB, integer>; capped at 8192 |
| `VICTORY_CARD_ROOM_FOR_TARGETS_SETTLE_MS` | Gives up the card memory kept back for render targets once they hold half of it and have not grown 64 MiB for this long. | 20000 | &lt;milliseconds, integer>; 0 = keep the reserve all run, as before |
| `VICTORY_CARD_SPANS` | Measures how long the card itself works on our submissions, using its own timestamps, and logs it. *Diagnostics build only.* | off | 1 = on; unset/other = off |
| `VICTORY_CARD_STEPS` | Does steps of a submission that read in-flight title memory on the card, in the flights' order, instead of on the CPU. *Set by `scripts\play.ps1`.* | 0 (off) | 1 = stage 1; 2 = stage 2; other = off |
| `VICTORY_CARD_STEPS_MORE` | Lets a submission have up to 49,152 steps and 16 MiB of reads done on the card (not 12,288 and 4 MiB) instead of waiting for flights, with up to 16,384 changed runs (not 4,096) sent up under flights' writes. | off | 0 or unset = off; anything else = on |
| `VICTORY_CARD_WINDOWS_MB` | Sets the most bytes of memory windows outside buffers that may have copies in the card's memory; `auto` sizes it from the card's budget. | 0 (none) | &lt;MiB, integer>; auto = 9 % of the budget, at most 1024 MiB (about 1,000 MiB on a 12 GB card) |
| `VICTORY_CHECK_TEARDOWN` | Lets `victory upscaler-check` shut Streamline down at its end. *Diagnostics build only.* | on | 0 = not shut down; anything else = on |
| `VICTORY_CLZERO_ASK` | Makes the console block-zero instruction ask Windows about memory every time instead of remembering answers per thread. | off (answers are remembered per thread) | 1 = ask Windows every time; other = remember |
| `VICTORY_COMMITTED_QUERY` | Checks whether title pages are readable with VirtualQuery, as before, instead of the faster kernel page query. | off | 1 = on; unset/other = off |
| `VICTORY_CONFIG` | Names the configuration file to use instead of ./victory.toml. | ./victory.toml | &lt;file path>; empty = unset |
| `VICTORY_CONSOLE_DXC` | Test only: path to the console's shader compiler, for comparing console-built and PC-built shaders drawn alike. | not set (test only) | &lt;path of the console's shader compiler> |
| `VICTORY_COPIES_IN_PLACE` | Does the title's buffer copies on the card at their place in the list, not all at submission on the CPU. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_COPY_ASK` | Asks Windows (VirtualQuery) whether a title copy destination is fully mapped before copying into it. | off | 1 = on; unset/other = off |
| `VICTORY_COPY_BESIDE` | Does the title's memcpy copies without page-guard faults, noting them beside instead. | off | 1 = on; unset/other = off |
| `VICTORY_COPY_MODE` | Experiment on how submission copies of title memory run: normal, plain, or skipped. *Diagnostics build only.* | guarded copies | plain = unguarded copies; none = skip copies; other = guarded |
| `VICTORY_COPY_OVER_TITLE` | Chooses what a texture the title first copies into holds where the copy writes nothing: zeros, or title memory. | default handling (zeros where the copy wrote nothing) | 0 = off; title = take the title's bytes; other = default |
| `VICTORY_CRASH_SAVE` | On a crash, writes the listed title memory ranges to files in the cache folder for study. | none | &lt;hex rva>:&lt;hex length>,... title ranges to save on a crash |
| `VICTORY_CUT_WAITS` | Honours the title's waits list by list, cutting its lists at each wait, instead of holding whole lists. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_CUT_WRITES` | Treats the title's write-value commands at the bottom of the pipe as cut points that split lists. | off | 1 = on; unset/other = off |
| `VICTORY_D16_BITS` | Keeps the 16 bits of 16-bit image loads and gathers through views of integer formats in translated shaders. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_D3D12_ADAPTER` | Picks the graphics adapter by DXGI index or by name, instead of the best performance one. | automatic (best performance adapter) | &lt;DXGI adapter index, integer> or &lt;adapter name> |
| `VICTORY_D3D12_BACKGROUND` | Switches off the D3D12 runtime's background shader recompilation threads. | left on | 0 = switch off background work; other = leave on |
| `VICTORY_D3D12_DEBUG` | Turns on the D3D12 debug layer and prints its messages. *Diagnostics build only.* | off | 0 = off; anything else = on |
| `VICTORY_D3D12_DRED` | Turns on device-removed data to find which command a removed device stopped at. *Diagnostics build only.* *Set by `scripts\play.ps1`.* | off | light = breadcrumbs only; 0 = off; other = full |
| `VICTORY_D3D12_GBV` | Turns on GPU-based validation so bad descriptor indices become messages naming the command. *Diagnostics build only.* | off | 0 = off; anything else = on |
| `VICTORY_D3D12_WARP` | Runs the host on the software (WARP) adapter, so a wild shader access crashes with an address. *Diagnostics build only.* | off | 0 = off; anything else = on |
| `VICTORY_DATA_DIR` | Folder for data the program writes such as logs and crash reports; must lie outside the repository. | the config file's value, else the default data folder | &lt;folder path>; must be outside the repository; empty = unset |
| `VICTORY_DECLARE_ONCE` | Declares the ranges a list's shaders write only once, not at every use, so they copy back into title memory. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_DEPTH_TWINS` | Makes a colour twin for a depth texture the title also uses for unordered access, where the host cannot do both. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_DESC_WHY` | Prints why a descriptor trace in a translated shader could not be followed. | off | any value = on; unset = off |
| `VICTORY_DIFF_ONLY` | Test only: runs only the differential test cases whose name matches this text. | not set (test only) | &lt;text>; only cases whose name contains it run |
| `VICTORY_DIRECT_WRITES` | Writes into title memory with the kernel's own call instead of WriteProcessMemory, skipping its slow page query. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_DISPATCH_SPLIT` | Splits over-limit dispatches into pieces for 32-lane GPUs, as an experiment. | off | 1 = on; unset/other = off |
| `VICTORY_DISPLAY_MODE` | Sets the display size the console reports to the title. | 3840x2160 | &lt;width>x&lt;height> in pixels, 640x360 to 7680x4320; out of range = default |
| `VICTORY_DLSS` | NVIDIA DLSS takes the title's upscaling (docs/dlss.md): the title is put on its FidelityFX path and its upscales go to DLSS through NVIDIA Streamline; without the files or a card that has DLSS, nothing changes. *Set by `scripts\play.ps1`* (`-Dlss`). | off | 1 = on; unset/other = off |
| `VICTORY_DLSS_JITTER` | Multiplies the jitter handed to DLSS, to find a sign or scale fix. *Diagnostics build only.* | 1,1 | &lt;x,y>, decimals |
| `VICTORY_DLSS_MODE` | The DLSS mode, which sets the scene's render scale. *Set by `scripts\play.ps1`* (`-Dlss`). | quality | dlaa (or native) = 1.0; quality = 2/3; balanced = 0.58; performance = 1/2; ultraperformance = 1/3; other = refused with a message |
| `VICTORY_DLSS_MV` | Multiplies the motion-vector scale handed to DLSS, to find a sign or scale fix. *Diagnostics build only.* | 1,1 | &lt;x,y>, decimals |
| `VICTORY_DLSS_PRESET` | The DLSS model preset. | default (NVIDIA's choice) | default, e, f, j, k, l, m; other = refused with a message |
| `VICTORY_DLSS_REACTIVE` | Gives DLSS the title's reactive mask. | on | 0 = not given; anything else = on |
| `VICTORY_DSP_CONVOLVE` | Computes the audio DSP's mono convolutions of 1024-frame blocks, so TVs, phone calls and reverb are heard instead of silent. | on | 0 = off (the DSP's outputs are zeros, as before); anything else = on |
| `VICTORY_DSP_GAIN` | Scales the computed DSP convolutions (the hardware's own scaling is not known). | 2 | &lt;factor, number> |
| `VICTORY_DSTORAGE_HALF` | Makes a file-streaming queue submit itself once half full, as DirectStorage for PC does. | off | 1 = on; unset/other = off |
| `VICTORY_DSTORAGE_IO` | Chooses how file-streaming reads files: one handle, a handle per worker, overlapped, or unbuffered. | the built-in default | one = one handle; handles = a handle per worker; overlapped = overlapped reads; direct = unbuffered handle per worker |
| `VICTORY_DSTORAGE_LOG` | Writes a streaming log with a line per request, file and mapping cost to this file. | off | &lt;file path> to write the streaming log |
| `VICTORY_DSTORAGE_RETRIES` | Sets how many times a streaming write into not-yet-ready pages is retried before the request fails. | 12 | &lt;retries, integer>; waits grow from 2 ms to 128 ms |
| `VICTORY_DSTORAGE_STAGING_MB` | Raises the streaming staging buffer to at least this many MiB, above what the title asks for. | 0 (what the title asks for) | &lt;MiB, integer>; staging buffer is at least this big |
| `VICTORY_DSTORAGE_TRACE` | Traces the streaming system: per batch and unexpected changes, or per request. | 0 (off) | 0 = off; 1 = line per batch and per faulty request; 2 = line per request |
| `VICTORY_DUALSENSE` | Reads PlayStation pads (DualShock 4, also through the 0x0BA0 adapter; DualSense; DualSense Edge) over HID, USB or Bluetooth, with rumble; looked for every 2 seconds while no XInput pad is there. | auto (only while XInput lists no pad) | 0 = never; 1 = beside XInput pads too; unset = auto |
| `VICTORY_DUMP_MATCH` | Dumps shader containers whose description line contains one of the texts to a folder, as the title has them. | off | &lt;folder>;&lt;text> or several texts separated by a vertical bar; a text is a name or #hash |
| `VICTORY_DUMP_TARGETS` | Saves the title textures that captured frames render to or write, in first-use order. *Diagnostics build only.* | off | on when set and not 0; needs VICTORY_CAPTURE |
| `VICTORY_DXIL` | Path of dxil.dll, the validator that checks translated shaders in tests and the offline batch. | newest Windows SDK's dxil.dll | &lt;path of dxil.dll> |
| `VICTORY_DYNAMIC_SLOTS` | Sets the most descriptor slots a heap may use for the dynamic tables of the title's slots kept in place. | 262144 | &lt;slots, integer> |
| `VICTORY_EARLY_ALL` | Starts every title pipeline at creation, whether requested before or not, so the driver's cache holds it. | off | 1 = on; unset/other = off |
| `VICTORY_EARLY_DEPTH` | Lets translated pixel shaders declare early depth testing where the console's did. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_EARLY_TEXTURES_MS` | Sets the milliseconds a frame the pass uploading index-picked textures takes before a shader asks for them. | 12 ms | &lt;milliseconds, integer> |
| `VICTORY_EFFICIENCY_MODE` | Lets Windows slow this process down when its window is not in front, instead of preventing that. | off (Windows is told not to throttle) | 1 = let Windows throttle; other = prevent |
| `VICTORY_EMIT_DUMP` | Writes every translated shader to this folder as a file, for study. | off | &lt;folder path outside the repository> |
| `VICTORY_EXACT_BACK` | Writes what shaders wrote into title memory run by run, not as one stretch from first to last changed byte. | on | 0 = off; anything else (or unset) = on |
| `VICTORY_EXACT_POOL` | Uses the newer pooled lookup of exact buffer copies, with separate locks for hits and new copies. | on | 0 = off; anything else = on |
| `VICTORY_EXIT_CHILD` | Test only: marks the child process of the thread-exit test, which ends other threads but one. | not set (test only) | any value = run as the child process |
| `VICTORY_F16_NEAREST` | Rounds 16-bit float operations to nearest like the console, adding instructions to each. | off | 1 = on; unset/other = off |
| `VICTORY_FAIL_DUMP` | Saves the first 3 containers of each translation failure reason to this folder, to study gaps offline. | off | &lt;folder path> for the dumps |
| `VICTORY_FAIR_DUE` | Serves texture slots that are due for upload in turns, alternating between ones seen before and ones never seen. | on | 0 = off; anything else or unset = on |
| `VICTORY_FALLBACK_HEAPS` | Always binds a view heap and a sampler heap on a list, with a one-slot heap of ours if unset. | off | 1 = on; unset or anything else = off |
| `VICTORY_FAST_CORES` | Chooses which of the renderer's threads ask Windows to run on the processor's fast cores. | 2 | integer 0-3: 0 = no thread asks; 1 = threads a frame waits for; 2 = plus threads sharing their work; 3 = plus background workers |
| `VICTORY_FAULT_BOOK` | Writes a book of the title's write faults on guarded memory (block, frame time, cause) to the named file. *Diagnostics build only.* | off | &lt;file path>; empty = off |
| `VICTORY_FAULT_BOOK_AT` | Sets the frames at which the fault book is written out, when VICTORY_FAULT_BOOK is set. *Diagnostics build only.* | 150,210,270 | &lt;frame numbers, comma-separated> |
| `VICTORY_FETCH_CHECK` | Hashes pieces of card-to-PC copies to catch ones that came down while a later flight was still writing them. | off | 1 = on; unset or anything else = off |
| `VICTORY_FFX_SCALE` | Sets the render scale used with the title's FidelityFX upscaler (Series S mode). | not set (title's own) | &lt;render scale, decimal 0.5 to 1.0>; other values are refused with a message |
| `VICTORY_FILL_TITLE` | Lets a small fill or buffer copy proceed without waiting for in-flight work when the title itself wrote that memory. | 0 (off) | 0 = off; 1 = on, also checks for title writes since the page was taken; 2 = on without that check |
| `VICTORY_FIRSTLANE_OWN` | Makes the shader translator take the host wave's own first lane where the console reads its first lane. | off | 1 = on; unset or anything else = off |
| `VICTORY_FIRST_HALF` | Lets the translator evaluate a first-half interpolation as the full pair when a first attempt at the shader was refused. | on | 0 = off; anything else or unset = on |
| `VICTORY_FRAMEGEN` | DLSS frame generation (docs/dlss.md); needs DLSS on, `VICTORY_THIN=1` (or `VICTORY_ASYNC_SUBMIT=1`) and the interface plane (`VICTORY_UI_PLANE=1` at a `VICTORY_OUTPUT` smaller than 3840x2160), and is refused with a log line otherwise. *Set by `scripts\play.ps1`* (`-FrameGen`). | off | 1 = on; unset/other = off |
| `VICTORY_FRAMEGEN_CYCLE` | Every N presents, asks frame generation for 2x, 4x, off and 6x in turn (an unattended test). *Diagnostics build only.* | 0 (off) | &lt;presents, integer> |
| `VICTORY_FRAMEGEN_HUDLESS` | With frame generation, tags the scene plane (the frame without the interface, `VICTORY_UI_PLANE`) for it. | on | 0 = not tagged; anything else = on |
| `VICTORY_FRAMEGEN_ROOM_MB` | Lowers the texture queue's room line by this much while frame generation is ready, for what frame generation allocates. | 1024 | &lt;MiB, integer>; 0 = not lowered |
| `VICTORY_FRAMEGEN_X` | Frames shown per frame rendered with frame generation; the F1 menu changes it while the title runs once frame generation is ready. *Set by `scripts\play.ps1`* (`-FrameGen`). | 2 | 2 to 6; 0 = made ready but off |
| `VICTORY_FRAME_INTERVAL_MS` | Sets the title's frame interval, patched into the title's code and used by our frame clock. | not set (title's own) | &lt;milliseconds, decimal 0 to 1000>; 0 = no wait at all |
| `VICTORY_FRESH_OPEN` | Leaves the host list made for a title's new list open for its next reset instead of closing it at once. | on | 0 = closed at once, as before; anything else = on |
| `VICTORY_FSR_PATH` | Puts the title on its FidelityFX path at `VICTORY_DLSS_MODE`'s scale with its own FSR, without Streamline, to compare with DLSS (the title's FSR4 only works at 1280x720 in, 1920x1080 out). | off | 1 = on; unset/other = off |
| `VICTORY_FULLSCREEN` | Shows the title's window borderless over the whole screen. The F1 menu sets it (next start). | off | 1 = on; unset/other = off |
| `VICTORY_FULLSCREEN_RES` | The size of the borderless window `VICTORY_FULLSCREEN=1` makes. | the main screen's size | &lt;width>x&lt;height> |
| `VICTORY_FULL_CARD_PASS` | When the card is full, holds back texture slots needing a new host copy and carries on with the slots behind them. | on | 0 = off; anything else or unset = on |
| `VICTORY_GAME_DIR` | Sets the folder holding the title's files, overriding the one in victory.toml. | from victory.toml | &lt;folder path> |
| `VICTORY_GPU_LOAD_MS` | Makes the title's four per-frame GPU timestamps report the given load, so its GPU time looks non-zero. | 0 (timestamps as they are) | &lt;milliseconds, integer> of GPU time to report per frame |
| `VICTORY_GPU_TIMING` | Runs and times each list of a submission on its own and logs the slow ones with their commands. *Diagnostics build only.* | off | set to anything except 0 = on; unset or 0 = off |
| `VICTORY_GROUP_SYNC` | Chooses which of a translated shader's group barriers actually wait, for finding a barrier that hangs the card. | 2 | 0 = no barrier waits; 1 = title's barriers wait only in safe blocks; 2 or other = all barriers wait |
| `VICTORY_GUARD_ASK` | Asks Windows about every memory range's state each time, instead of trusting ranges already known to be guarded. | off | 1 = on; unset or anything else = off |
| `VICTORY_GUARD_HOLES` | Checks the pages around unmapped holes in a range when deciding if it is a view, not just the first address. | on | 0 = off; anything else or unset = on |
| `VICTORY_GUARD_REST` | Sets how long a block the title just opened by a write stays open before being guarded again. | 3000 | &lt;microseconds, integer above 1>; 0 = no rest time |
| `VICTORY_HEAP_PASS_ASIDE` | Runs the pass over the descriptor heap's buffer views on a helper thread while the title records. | on | 0 = off; anything else or unset = on |
| `VICTORY_HEAP_RECHECK` | Looks again at textures picked by index after their upload to catch later changes. | on | 0 = off; anything else or unset = on |
| `VICTORY_HEAP_SPREAD` | Looks at the slots re-checked for no write a part at a time every pass, rather than all every sixteenth pass. | on | 0 = off; anything else or unset = on |
| `VICTORY_HEAP_SYNC` | Mirrors changed descriptor heap pages into the host heap before lists are sent to the queue. | on | 0 = off; anything else or unset = on |
| `VICTORY_HEAP_SYNC_ASIDE` | Does a submission's descriptor heap sync on its own thread beside the rest of the preparing. | on | 0 = off; anything else or unset = on |
| `VICTORY_HEAP_SYNC_EARLY` | Sets how long the heap sync thread waits for a submission before making a round of its own. | 4 | &lt;milliseconds, integer>; 0 = none |
| `VICTORY_HEAP_SYNC_FIRST` | Starts a submission's round of the heaps' records as soon as it holds the submission lock. *Set by `scripts\play.ps1`.* | off | unset or 0 = off; anything else = on |
| `VICTORY_HEAP_TEXTURES` | Uploads textures placed in memory that is not fully mapped yet with what is there, and re-checks them later. | on | 0 = off; anything else or unset = on |
| `VICTORY_HELD_LEVELS` | Reads, keeps and uploads a placed texture only from the first mip level its views actually show. | on | 0 = off; anything else or unset = on |
| `VICTORY_HELPER_OWN` | Makes a pixel shader's helper lane read its own value where the console reads the first lane's. | on | 0 = off; anything else or unset = on |
| `VICTORY_HELPER_PRIORITY` | Runs the threads the submitting thread waits for at the submitting thread's priority. | on | 0 = off; anything else or unset = on |
| `VICTORY_HOST_LOCK_CHILD` | Internal marker for a loader-lock unit test, which re-runs itself as a child process. | not set (test only) | set by the unit test itself; not for users |
| `VICTORY_HOST_MAX_WAVE` | Older name for VICTORY_HOST_WAVE: the host's wave size that offline shader translation is done for. | from this PC's card | &lt;lanes, integer>; used when VICTORY_HOST_WAVE is not set |
| `VICTORY_HOST_WAVE` | Sets the host wave size (lanes) that offline shader translation is done for. | from this PC's card | &lt;lanes, integer>, e.g. 32 or 64 |
| `VICTORY_IMAGE_WATCH` | Logs chosen words of the title's image once a second, to compare two launches' internal counters. | off | &lt;name=hexoffset:bytes, comma-separated>; bytes 1/2/4/8, default 8 |
| `VICTORY_IME_ENGLISH` | Switches the title's window to the English (en-US) keyboard layout when it is made and each time it gets the focus. The F1 menu sets it (next start). | off | 1 = on; unset/other = off |
| `VICTORY_INDIRECT_BATCH` | Clamps the executions of one indirect record in batches instead of one by one. *Set by `scripts\play.ps1`.* | 1 | 0 = each execution on its own; 1 = batched; 2 = batches continue across dispatches until the title waits or draws |
| `VICTORY_INDIRECT_CARD_BUDGET` | Keeps a list's indirect-work budget in the card's own memory rather than in CPU-mapped memory. | on | 0 = off; anything else or unset = on |
| `VICTORY_INDIRECT_REST` | Lets a record that overflows the list's remaining indirect budget take the rest, so later records run empty too. | on | 0 = off; anything else or unset = on |
| `VICTORY_INDIRECT_WIDE` | Lets an indirect dispatch run up to 2^20 groups in x on AMD cards instead of 65,535. | on (AMD cards only) | 0 = 65,535 groups max; anything else = 2^20 on AMD; other cards stay at 65,535 |
| `VICTORY_INDIRECT_WINDOW` | Clamps indirect draws that read record after record of one argument buffer a window at a time. *Set by `scripts\play.ps1`.* | off | 1 = on; unset or anything else = off |
| `VICTORY_INPLACE` | Uses windows of title memory directly as card memory instead of copying them, at the given level. | 0 (off) | integer level: 0 = off; 1 = windows outside buffers in place; 2 = also title buffers (not with late wait) |
| `VICTORY_INPLACE_MB` | Sets the most title memory held in place at once. | 4096 | &lt;megabytes, integer> |
| `VICTORY_INPLACE_PROBE` | Runs a probe that tests opening title memory as card memory, with a census afterwards. | 0 (off) | 0 = nothing; 1 = probe on own memory plus census; 2 = also opens part of title memory |
| `VICTORY_INPLACE_PROBE_MB` | Sets the size of the in-place probe's test memory. | 64 | &lt;megabytes, integer>; capped at 1024 |
| `VICTORY_JOIN_BARRIERS` | Holds back barrier calls on a host list and joins them into fewer calls. | off | 1 = on; unset or anything else = off |
| `VICTORY_JOIN_COPIES` | Joins a title buffer copy that continues the previous one in its list into a single step. *Set by `scripts\play.ps1`.* | off | 1 = on; unset or anything else = off |
| `VICTORY_JOIN_LISTS` | Sets how many lists the line's thread hands to the host as one batch. | 48 | &lt;lists, integer, at least 1> |
| `VICTORY_JOIN_PARTS` | Sets how many parts of submissions the line's thread hands to the host as one. *Set by `scripts\play.ps1`.* | 1 (off) | &lt;parts, integer, at least 1>; 1 = no joining |
| `VICTORY_KEEP_FREED` | Keeps memory given to the card with it while there is room, instead of giving it back. | on | 0 = off; anything else or unset = on |
| `VICTORY_KEEP_SHOWN` | Keeps a texture slot showing its old texture until the newly written view's texture is made and uploaded. | 1 | 0 = never keep; 1 = keep a texture of the same kind; 2 = keep any of the same dimension |
| `VICTORY_KEEP_SNAPSHOTS` | Keeps a copy's snapshot between submissions and re-takes only the pieces that changed. | off | 1 = on; unset or anything else = off |
| `VICTORY_KEEP_SNAPSHOTS_MB` | Caps the memory used by kept snapshots. | 256 | &lt;megabytes, integer> |
| `VICTORY_KEEP_VIEWS` | Keeps a slot showing the old view of a placed texture until the new view is made, rather than null at once. | on | 0 = off; anything else or unset = on |
| `VICTORY_KEYS` | Scripts keyboard presses held for 0.2 s at given times after input starts, also mapped to pad buttons. *Diagnostics build only.* | off (no scripted keys) | &lt;seconds:key, comma-separated> or &lt;from-to:key> in seconds; each key held 0.2 s |
| `VICTORY_KEYS_FILE` | Reads scripted key presses from a file as it grows, for driving the title from outside. *Diagnostics build only.* | off | &lt;file path>; keys read as the file grows |
| `VICTORY_KNOWN_HOLES` | Remembers which placeholder memory ranges are holes so they are not queried again at every asking. | on | 0 = off; anything else or unset = on |
| `VICTORY_LAND` | Puts what shaders wrote back into title memory on a few worker threads instead of on one thread. | on | 0 = off; anything else or unset = on |
| `VICTORY_LAND_ANYTIME` | Lets a landing write through one of our own views even while the runtime changes a mapping. | on | 0 = off; anything else or unset = on |
| `VICTORY_LAND_BESIDE` | Writes landed pieces through views of ours beside the guards instead of through the title's addresses. | on | 0 = off; anything else or unset = on |
| `VICTORY_LAND_ECHO` | Detects refreshes that only saw a landing's own writes and sends nothing up for them. *Set by `scripts\play.ps1`.* | on | 0 = off; anything else or unset = on |
| `VICTORY_LAND_SKIP` | Leaves out pieces that an earlier fetch already holds as they are now. | off | 1 = on; unset or anything else = off |
| `VICTORY_LAND_TAKEN` | Notes a copy a landing went through as up to date with it, so the next refresh skips that 64 KiB. | on | 0 = off; anything else or unset = on |
| `VICTORY_LAND_THREADS` | Sets how many threads write shader results back into title memory. | cores / 4, between 1 and 4 | &lt;threads, integer 1 to 16> |
| `VICTORY_LANE_CORE` | Gives the submitting thread a fast core to itself, keeping the core's sibling thread free. | off | 1 = on; unset or anything else = off |
| `VICTORY_LATE_WAIT` | Sends submissions to the card without waiting for each, settling them later in stages. | 3 when the relay is on, else 0 | 0 = off; 1, 2, 3 = stage; other = default; off if VICTORY_GPU_TIMING is set |
| `VICTORY_LATE_WAITS` | Makes a blocking wait for an already-happened event sleep to the next frame tick as before. | off | 1 = on; unset or anything else = off |
| `VICTORY_LATE_WAIT_LISTS` | Sets the most host lists left in flight before the flights are settled. | 256 | &lt;lists, integer, at least 1> |
| `VICTORY_LATE_WAIT_WRITES` | Lets a submission with bottom-of-pipe writes fly, telling the title later rather than at once. | on | 0 = off; anything else or unset = on |
| `VICTORY_LAZY_PRELUDE` | Makes a host list's prelude of texture transitions and copies only when the list first needs one. | on | 0 = off; anything else or unset = on |
| `VICTORY_LAZY_TEXTURES` | Makes a texture's host copy on first use rather than when the title creates it. | on | 0 = off; anything else or unset = on |
| `VICTORY_LDS_FLAGS` | Lets the translator handle shared-memory regions that only raise flags without full synchronisation. | on | 0 = off; anything else or unset = on |
| `VICTORY_LDS_INIT_HOIST` | Also stores accumulator initialisations at the start of a shader program. | on | 0 = off; anything else or unset = on |
| `VICTORY_LENT_HEAP` | Lends the title's biggest view heap to dispatches of lists that bound none, instead of dropping them. | on | 0 = off; anything else or unset = on |
| `VICTORY_LEVELS_MIN_KB` | Sets the smallest texture size from which texture levels are made on demand. | 1024 | &lt;kilobytes, integer>; minimum 64 |
| `VICTORY_LEVELS_ON_DEMAND` | Makes mip levels of large placed textures only when needed, using tiled resources memory. | off | 1 = on; unset or anything else = off |
| `VICTORY_LIBRARY_HANDLE` | Keeps the mapped pipeline library file's handle open so a save cannot zero the mapped pages. | on | 0 = off; anything else or unset = on |
| `VICTORY_LIBRARY_LOCK` | Serialises every pipeline library call under one lock of ours. | off | 1 = on; unset or anything else = off |
| `VICTORY_LIBRARY_MAP` | Maps the pipeline library file into memory and saves through the mapping instead of using buffers. | on | 0 = off; anything else or unset = on |
| `VICTORY_LIBRARY_STATIC_SAMPLERS` | Also loads and stores pipelines with static samplers in the pipeline library. | off | 1 = on; unset or anything else = off |
| `VICTORY_LIBRARY_TRIM` | Releases the mapped library file's pages from memory after ten seconds without a pipeline load. | on | 0 = off; anything else or unset = on |
| `VICTORY_LIGHT_PROBE` | Writes a per-frame CSV of the picture's light: exposure values and the indirect-light stand-in. | off | &lt;csv file path>; empty or 0 = off |
| `VICTORY_LIGHT_PROBE_LUX` | Also records the lux program's inputs and outputs in the light probe. | off | 1 = on; unset or anything else = off |
| `VICTORY_LIGHT_REFRESH` | Refreshes host copies the light way, skipping view memory that nothing wrote since it was last asked. | on | 0 = off; anything else or unset = on |
| `VICTORY_LIST_POOL` | Reuses host command lists that have run instead of making each new and discarding it. | on | 0 = off; anything else or unset = on |
| `VICTORY_LIST_POOL_IDLE_S` | Destroys an idle pooled list, with its command allocators, once nobody took it for this many seconds. | not set (never; with `VICTORY_ALLOCATOR_TRIM=1` its time) | &lt;seconds, integer > 0> |
| `VICTORY_LIST_SLOTS` | Lists the heap slots textures are picked from after draws whose pixel shader matches the given hashes. *Diagnostics build only.* | none | &lt;shader hash prefixes, comma-separated> |
| `VICTORY_LIST_VIEWS` | Makes a view that a list's tables hold once per list instead of once per table. | on | 0 = off; anything else or unset = on |
| `VICTORY_LOG` | Sets the log filter for every binary, falling back to RUST_LOG. | info (debug builds of tools may differ) | &lt;tracing filter>, e.g. victory_loader=trace,info; falls back to RUST_LOG |
| `VICTORY_LOG_FILE` | Writes the log to this file too, without colours; a path inside this repository is refused (the log holds the title's data). *Cleared by `scripts\play.ps1` unless given with `-More` (its `-Log` writes a file of its own).* | not set (no file) | &lt;file path>; empty = no file |
| `VICTORY_LOW_EARLY_MB` | Reserves this much address space in [4 GiB, 2^37) at host creation, in 64 MiB pieces, for the ray-tracing builder's memory arenas. | 3200 with `VICTORY_BVH_BUILDER=1`, else 0 | &lt;megabytes, integer>; 0 = none reserved |
| `VICTORY_MAPPED_BY_PAGE` | Measures how much title memory is mapped one 4 KiB page at a time, without walking ranges. | off | 1 = on; unset or anything else = off |
| `VICTORY_MAPPED_BY_VIEWS` | Asks the runtime's own views of physical pages how much title memory is mapped before asking Windows. | on | 0 = off; anything else or unset = on |
| `VICTORY_MAP_LEAN` | Makes a mapping change ask the host only for what it needs, skipping redundant splits and commits. | on | 0 = off; anything else or unset = on |
| `VICTORY_MAP_THREADS` | Sets how many threads map a group of pages side by side. *Set by `scripts\play.ps1`.* | 8 | &lt;threads, integer 1 to 32>; 1 = caller alone; auto = adaptive |
| `VICTORY_MASKED_PATCH` | Lets a copy's page whose title bytes sit under shader writes be patched with more copies of its own. | off | set to anything except 0 = on; unset or 0 = off |
| `VICTORY_MEMORY_CARD_MB` | Caps how much of the card's own memory holds side copies that shaders see. | 4096 | &lt;megabytes, integer> |
| `VICTORY_MEMORY_CATEGORIES` | Logs a periodic line showing where submission copies of title memory go, by category. *Diagnostics build only.* | off | set to anything except 0 = on; unset or 0 = off |
| `VICTORY_MEMORY_DUMP` | Writes the card's memory arenas to files in the named folder when a file named go appears there. *Diagnostics build only.* | off | &lt;folder path>; a file named go there triggers the dump |
| `VICTORY_MEMORY_MAP` | Sets how often the address space is walked to log the memory map line. *Diagnostics build only.* | 60 | &lt;memory lines (one per 5 s), integer> |
| `VICTORY_MENU_OPEN` | Shows the F1 settings menu from the start. *Diagnostics build only.* | off | 1 = on; unset/other = off |
| `VICTORY_MESH_LAUNCH` | Puts an amplification shader of ours in front of every translated mesh shader to launch mesh draws. | off | set to anything except 0 = on; unset or 0 = off |
| `VICTORY_MESH_OUTPUT_STRICT` | Refuses mesh shaders whose outputs pass the host's 32 KiB cap instead of emitting them with lower maxima. | off | set to any value (even 0) = on; unset = off |
| `VICTORY_MESH_TEST` | Skips mesh draws or shrinks them to one pixel, to tell a hang caused by shaders from one caused by coverage. | off | skip = issue no mesh draws; scissor = one-pixel draws; anything else = off |
| `VICTORY_METER_THREAD` | Makes the frame meter's five-second lines on a thread of ours so presenting is not held up. | on | 0 = meter lines on the presenting thread; anything else or unset = own thread |
| `VICTORY_MOUSE_CAPTURE` | Keeps the pointer inside the title's window and hides it while the window is in front, and gives the title a GameInput mouse, so the camera follows the mouse. The F1 menu sets it (next start). | off | 1 = on; unset/other = off |
| `VICTORY_MOUSE_WARP` | With `VICTORY_MOUSE_CAPTURE=1`, puts the pointer back in the window's centre after each reading, so that it never stops at the window's edge. | on | 0 = off (the pointer stays where it is); anything else (or unset) = on |
| `VICTORY_NAN_GUARD` | Guards translated shader maths against not-a-number results. | on | 0 = off; anything else or unset = on |
| `VICTORY_NONE_SUCH` | Not a real switch: a made-up name used by a unit test to check the A/B picker. *Diagnostics build only.* | not set (test only) | none; a made-up name used only by a unit test |
| `VICTORY_ONE_PAD` | Shows the title one always-present pad fed by the keyboard and every host pad. | on | 0 = each host pad shown separately; anything else or unset = one always-present pad |
| `VICTORY_OPEN_ONE_VIEW` | Opens only the written view on a write fault and leaves the page's other views read-only. | on | 0 = off; anything else or unset = on |
| `VICTORY_OUTPUT` | Sets the title's output size by patching its display mode. *Set by `scripts\play.ps1`.* | not set (title's own) | &lt;width>x&lt;height>: 1280x720, 1664x936, 1920x1080, 2560x1440 or 3840x2160 |
| `VICTORY_OWNERLESS_VIEWS` | Caps how many table entries of views without a resource are kept; past it the oldest is freed (stops a memory leak in long sessions). | 1048576 | &lt;entries, integer>; 0 = keep all, as before |
| `VICTORY_PAD_SECONDS` | Test only: how long the ignored pad test reads a real pad. | 20 (test only) | &lt;seconds, integer> |
| `VICTORY_PAGE_PARTS` | Copies only the 4 KiB parts of a page the kernel says were written, not the whole 64 KiB page. | on | 0 = off; anything else or unset = on |
| `VICTORY_PAGE_WRITES` | Chooses what happens when title memory is written without the page guards noticing: reopen guarded host copies and/or mark texture slots due. | 1 (guarded blocks reopened) | 0 = count only; 1 = blocks and texture slots; slots = texture slots only; all = everything; guards/unset/other = blocks only |
| `VICTORY_PAIRS` | In the offline shader translation walk, keeps the named vertex+pixel shader pairs (label substrings) so their pipelines are built afterwards. | none | comma-separated vs+ps pairs of label substrings |
| `VICTORY_PAIR_CHEAP` | Pair meetings of 32-lane host waves meet once per exchange (two buffers) and read their counter with one fenced load. Changes translations. | on | 0 = two meetings and an atomic per lane per read, as before; anything else = on |
| `VICTORY_PAIR_LDS_ORDER` | No pair meeting around LDS accesses whose order does not matter. Changes translations. | on | 0 = a meeting before every LDS access, as before; anything else = on |
| `VICTORY_PAIR_REGIONS` | With pair meetings, loop branches get their own lane region, and title barriers inside wave-wide branches become fences only when `VICTORY_GROUP_SYNC=1` (otherwise they still wait for the group). Changes translations. | on | 0 = as before; anything else = on |
| `VICTORY_PANEL` | Draws the run's numbers as white text on a black box in the picture's upper-left corner; F3 shows or hides it. In every build; F3 or the F1 menu turn it on and off while the title runs. | off | 0 or unset = off; anything else = on (F3 shows/hides) |
| `VICTORY_PATCH_CLZERO` | Replaces each faulting cache-zeroing loop in the title's code with plain stores the first time it faults, so it stops raising exceptions. | on | 0 = off; anything else = on |
| `VICTORY_PATCH_WAITS` | Replaces the title's console wait instructions (monitorx/mwaitx) with harmless ones the first time they fault, instead of emulating them each time. | on | 0 = off; anything else = on |
| `VICTORY_PC_DXC` | Path to the PC dxc.exe used to compile the helper shaders the host builds itself; otherwise it is looked for beside dxil.dll. | dxc.exe beside the validator's dxil.dll | &lt;path to dxc.exe> |
| `VICTORY_PICKED_MB` | Sets how many megabytes of buffers written through index-picked views are looked at per frame, in turn. | 32 | &lt;megabytes per frame, integer> |
| `VICTORY_PICKED_WRITES` | Brings what shaders write through index-picked buffer views back into title memory instead of leaving it only on the card. | 0 (off) | 0 = off; 1 = declare written ranges, not landed; 2 = declared and landed |
| `VICTORY_PIPELINE_BOOK` | Keeps a disk record of every pipeline the title creates so 'victory warm' can build their host pipelines ahead of a run. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/anything else = off |
| `VICTORY_PIPELINE_COMPUTE_WAIT` | Gives not-yet-made compute pipelines their own waiting budget, dropping the dispatch once spent, instead of waiting within the frame's budget. | on | 0 = off; anything else = on |
| `VICTORY_PIPELINE_COMPUTE_WAIT_MS` | Sets how long, per title frame and over all threads, dispatches may wait for compute pipelines to be made. | 5000 | &lt;milliseconds per title frame, integer> |
| `VICTORY_PIPELINE_LOWEST` | Runs the pipeline-making worker threads below the title's own and streaming threads in priority. | on | 0 = off; anything else = on |
| `VICTORY_PIPELINE_SLOW` | Limits how many workers build known slow pipelines that no draw has asked for yet, so other pipelines are not held up. | on (a quarter of the workers, at least 1) | 0 = no limit, in title order; anything else = limit on |
| `VICTORY_PIPELINE_WAIT_MS` | Sets how long a title frame may wait in total for pipelines to be made before their draws are dropped. | 500 | &lt;milliseconds per title frame, integer> |
| `VICTORY_PIPELINE_WAIT_ONCE` | Lets one pipeline be waited for at most one budget over the whole run, rather than a full budget in every frame. | on | 0 = off; anything else = on |
| `VICTORY_PIPELINE_WAIT_SHARED` | Makes the frame's pipeline-wait budget shared by all threads, so each thread only waits for what is left. | on | 0 = off; anything else = on |
| `VICTORY_PIPELINE_WORKERS` | Sets how many worker threads make pipelines. | two thirds of logical processors, 2 to 16 | &lt;worker threads, 1 to 64> |
| `VICTORY_PLACED_CONSTANTS` | Serves a constant buffer that an earlier copy in the list writes whole from a card-side buffer, not title memory. | on | 0 = off; anything else = on |
| `VICTORY_PLACED_EXACT` | Copies a shader-written source, held as one view's own bytes, from its copy on the card instead of CPU-filled upload memory. | on | 0 = off; anything else = on |
| `VICTORY_PLACED_FILLER` | Keeps the unused filler words of a texture description in the key that identifies a texture, so differing filler gives separate textures. | off (filler ignored in the key) | 1 = keep filler in the key; unset/other = ignored |
| `VICTORY_PLACED_ORDER` | Counts a copy at its place in a list as blocking card work only if that work stands behind it, and cuts a submission behind such a copy that later lists read on the CPU. *Set by `scripts\play.ps1` (not with `-Safe`).* | off | 0 or unset = off; anything else = on |
| `VICTORY_PLACED_READERS` | Counts readers standing ahead of a placed copy as waiting for it too, instead of only the ones behind it. | off (only later readers counted) | 0 = count readers ahead too; unset/other = only later readers |
| `VICTORY_PLANES` | Turns off composing the title's second presented plane over the first; also stops the separate UI-plane option from working. | on | 0 = off; anything else = on |
| `VICTORY_POKE` | Writes the given byte at each listed offset of the title's image twice a second, e.g. to keep a main-loop flag alive. *Diagnostics build only.* | none | &lt;hex rva>:&lt;hex byte>, comma-separated |
| `VICTORY_POOL_RECLAIM` | Frees empty 64 MiB texture blocks as soon as video memory use reaches 85 % of budget, even without the residency switch. | on | 0 = off; anything else = on |
| `VICTORY_POOL_SMALL` | Places textures of 64 KiB and less at 4 KiB multiples inside their block, where the host allows, not 64 KiB multiples. | off | 0 or unset = off; anything else = on |
| `VICTORY_POOL_STATS` | Logs every five seconds what the texture blocks hold: how full they are, how their free bytes lie, textures by piece size, and what small ones would hold at 4 KiB multiples (one more question of the host per texture made). | off | 1 = on; unset/other = off |
| `VICTORY_POP_ALL` | With the pop watcher, saves every watched frame's small picture instead of only the frames around detected blips. *Diagnostics build only.* | off | 1 = on; unset/anything else = off |
| `VICTORY_POP_FOLDER` | Folder where the pop watcher saves the 640-pixel-wide pictures around frames that show blips (at most 200). *Diagnostics build only.* | none (nothing saved) | &lt;existing or creatable folder path> |
| `VICTORY_POP_WATCH` | Reads back presented frames in the given time windows and compares neighbours to count popping textures (blips, steps, lone blocks). *Diagnostics build only.* | none (off) | &lt;from>-&lt;to> in seconds since start, comma-separated windows |
| `VICTORY_PORT_DESCRIPTORS` | Learns which descriptor slots the title wrote from its patched stores, instead of looking at every changed heap page each frame. | off | 1 = on; unset/anything else = off |
| `VICTORY_PORT_DESCRIPTORS_CHECK` | Sets how many heap pages the fallback checks per frame when descriptor porting is on. | 16 | &lt;pages per frame, integer>; 0 = none |
| `VICTORY_PORT_DESCRIPTORS_EVERY` | Sets how many frames pass between the fallback's questions to the write watch when descriptor porting is on. | 8 | &lt;frames, integer, minimum 1> |
| `VICTORY_PRELUDE_UPLOADS` | Decides title texture uploads at submission, after the check, and copies them on the list's prelude. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/other = off |
| `VICTORY_PROBE` | Test tool: runs each .dxil file in the folder on the card and prints what the host's compiler made of it. | not set (test only) | &lt;folder of .dxil files> |
| `VICTORY_PROCESS_PRIORITY` | Stops the process from raising itself to above-normal priority. | above normal priority | normal or 0 = leave priority as started; anything else = above normal |
| `VICTORY_PROFILE` | Samples the busiest threads' stacks at an interval during a time window and writes where the process's time goes to a file. *Diagnostics build only.* | off | &lt;from>,&lt;to>,&lt;ms>,&lt;file>: seconds window, sample interval in milliseconds, output file |
| `VICTORY_QUEUE_BATCH` | The texture pass tells the texture queue what came of its slots 16 at a time and takes slot state with the slots due. | on | 0 = one take of the queue's lock per slot, as before; anything else = on |
| `VICTORY_QUEUE_HOST_TEXTURES` | Queues records of textures the host writes for the texture pass. *Set by `scripts\play.ps1`.* | on | 0 = leave them out of the queue; anything else = on |
| `VICTORY_QUEUE_MARKS` | The texture queue's marks let notes that change nothing skip its lock. | on | 0 = the earlier take-out skip only (and `VICTORY_TABLED_REREAD` does nothing); anything else = on |
| `VICTORY_QUEUE_WAIT` | Makes queues honour fence waits; when off every submission runs as soon as it is made. | on | 0 = off; anything else = on |
| `VICTORY_QUICK_FIND` | Keeps a per-thread cache of the last copy found for an address, avoiding the shared map's lock. | on | 0 = off; anything else = on |
| `VICTORY_QUICK_MIRROR` | Tells a slot already made from its record so without taking the heap lock. | on | 0 = off; anything else = on |
| `VICTORY_RAW_VIEWS` | Keeps a raw twin of every buffer view of the title's slots so shaders picking buffers by a computed index read correctly. | on | 0 = off; anything else = on |
| `VICTORY_RAW_VIEW_CLAMP` | Bounds dynamic raw buffer views to the host buffer's real width; off uses the requested range unchecked. | on | 0 = off; anything else = on |
| `VICTORY_READY_ONCE` | Skips re-checking a table the list reads again when nothing in the list changed since. | on | 0 = off; anything else = on |
| `VICTORY_READ_BACK` | Allows texture-to-buffer copies as a read back through a host buffer; when off they are refused. | on | 0 = off; anything else = on |
| `VICTORY_RECHECK_LIVELY` | Sets how many one-frame-apart looks a texture found changed without notice gets after its upload before the looks space out. | 120 | &lt;looks, 0 to 255>; 0 = none |
| `VICTORY_RECHECK_SOON` | Looks again at a silently changed texture in the very next frame instead of after four. | on | 0 = off; anything else = on |
| `VICTORY_RECORD` | Records one frame of the D3D12 calls into the given folder for later replay; nothing is hooked without it. *Diagnostics build only.* | off (nothing hooked) | &lt;folder path>; empty = off |
| `VICTORY_RECORD_ALWAYS` | Makes the recorder write its capture even when no trigger fired. *Diagnostics build only.* | off | 1 = on; unset/anything else = off |
| `VICTORY_RECORD_AT` | Sets when the recorder captures: a time in seconds or a frame number. *Diagnostics build only.* | trigger (capture on the recorder's own trigger) | &lt;seconds, decimal> or f&lt;frame number> |
| `VICTORY_RECORD_FRAMES` | Sets how many frames the recorder writes down. *Diagnostics build only.* | 1 | &lt;frames, integer, minimum 1> |
| `VICTORY_REFRESH_AHEAD` | Brings the copies a submission's shaders write up to date together on several threads ahead of its steps. | on (needs wide refresh and light watch) | 0 = off; anything else = on |
| `VICTORY_REFRESH_CHECK` | After a refresh that trusted the write watch, compares 256 bytes with title memory and reports copies the card reads stale. | off | 1 = on; unset/anything else = off |
| `VICTORY_REFRESH_FRONT` | Brings everything a submission refreshes up to date in one go on several threads ahead of its steps. | off (on anyway when joins are asked for) | 0 or unset = off; anything else = on |
| `VICTORY_REF_PS` | Test tool: file of a reference pixel shader (DXIL) used instead of the generated one in the translator's comparison test. | not set (test only) | &lt;path to a DXIL file> |
| `VICTORY_REF_VS` | Test tool: file of a reference vertex shader (DXIL) used instead of the generated stub in the translator's comparison test. | not set (test only) | &lt;path to a DXIL file> |
| `VICTORY_RELAUNCHED_FROM` | Set by victory itself in a relaunched process: an inherited handle of the process it replaces, which it waits for. | not set (set by victory itself) | &lt;inherited process handle, set internally> |
| `VICTORY_RELAY` | Chooses the stage of the relay, a thread that lands flights once the card has run them; 0 turns it off. *Set by `scripts\play.ps1`.* | on, stage 2 | unset or 2 = stage 2; 1 = stage 1; anything else = off |
| `VICTORY_RELAY_COPY` | Fetches the pieces the relay needs from the card's memory over the copy queue instead of the drawing queue. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/anything else = off |
| `VICTORY_RELEASE_THREAD` | Drops released D3D12 resources on a thread of their own instead of on the submitting and landing threads. | on | 0 = off; anything else = on |
| `VICTORY_RENDER_SCALE` | Sets the title's own upscaler render scale relative to the output size. *Set by `scripts\play.ps1`.* | the title's own (2/3 without it) | &lt;scale, 0.5 to 1.0>; out of range = ignored with a message |
| `VICTORY_REPLAY_GBV` | In the frame replayer's debug device, turns on GPU-based validation of every shader access (very slow). *Diagnostics build only.* | off | any value = on |
| `VICTORY_REPLAY_PEEK` | In replay divergence search, always inspects each draw's results even when its kinds do not differ. *Diagnostics build only.* | off | any value = on |
| `VICTORY_RESIDENCY` | Turns on host-memory budgets that limit texture prefetch and let go of idle upload storage when system RAM or commit runs low. | off | 1 = on; unset/anything else = off |
| `VICTORY_RESIDENCY_LIMITED` | Forces the small-PC memory budgets of the residency system even when the PC has more than 32 GiB of RAM. | auto (on when the PC has 32 GiB RAM or less) | 1 = force small-PC budgets; unset/other = auto; needs VICTORY_RESIDENCY=1 |
| `VICTORY_REUPLOAD_WHEN_FULL` | Keeps bringing textures that already have a host copy up to date while the card has no room for new ones. | on | 0 = off; anything else = on |
| `VICTORY_RIPE_BLOCKS` | Sets how many texture blocks are made ahead of need so one has aged by the time its first piece is taken. | 12 | &lt;blocks, integer>; 0 = none |
| `VICTORY_ROOT_REACH_MS` | Sets how long an answer about how far committed memory reaches around a root view is reused. | 1000 | &lt;milliseconds, integer>; 0 = ask every time |
| `VICTORY_ROOT_SRV_DIRECT` | Binds the title's root SRVs as raw root descriptors again instead of bounded one-view tables whose reads past the end return zeros. | off (bounded one-view tables) | any value set = raw root descriptors; unset = off |
| `VICTORY_ROOT_UAV_MB` | Limits the size of a root unordered-access view of a title buffer's host copy to this many megabytes. | unlimited | &lt;megabytes, integer> |
| `VICTORY_ROOT_VIEW_MB` | Limits the size of a root view of a title buffer's host copy to this many megabytes. | unlimited | &lt;megabytes, integer> |
| `VICTORY_ROOT_WINDOW_MB` | Sets the most a root view without a stated length is given past its 4 MiB cell. | 32 | &lt;megabytes, 4 to 256>; 4 = the 4 MiB cell alone |
| `VICTORY_RT_LEAVE_OUT` | Leaves the ray-tracing passes out while the structure builder is off. | on (only while VICTORY_BVH_BUILDER is not 1) | 0 = make the ray-tracing passes; anything else = leave out |
| `VICTORY_SAFE_COPY` | Copies title memory in our own code rather than through the kernel, guarding against the title unmapping it meanwhile. | on | 0 = off; anything else = on |
| `VICTORY_SAME_RECORDS` | Skips writing bytes into title memory that already hold the same value, when putting a descriptor copy there. | on | 0 = off; anything else = on |
| `VICTORY_SAVE_CHILD` | Test-only: save folder passed to the save-game crash test's child process. | not set (test only) | &lt;save folder path>, set by the crash test |
| `VICTORY_SAVE_OP` | Test-only: which save operation the crash test's child process runs. | not set (test only) | &lt;operation name>, set by the crash test |
| `VICTORY_SAVE_STOP` | Test-only: makes the save-game crash test's child process stop for good at the named step. | not set (test only) | &lt;step number, integer> |
| `VICTORY_SCISSOR_TEST` | Experiment: widens all of the title's scissor rectangles to the whole render target. | off | full = widen all scissors; anything else = off |
| `VICTORY_SCRATCH_POOL` | Lets command lists share scratch upload chunks, taking one when first needed and giving all back on reset. | on | 0 = off; anything else = on |
| `VICTORY_SCREEN_WAIT_MS` | Sets the longest a frame waits for pipelines of screen-wide passes to be made. | 10000 | &lt;milliseconds, integer>; 0 = same budget as other pipelines |
| `VICTORY_SECTION_WINDOW` | Keeps one writable view of a memory section for DirectStorage writes instead of mapping a view for each write. | on | 0 = off; anything else = on |
| `VICTORY_SEND_BAR` | Keeps the copies on the card's drawing queue in memory the CPU writes through the bus (resizable BAR). | off | 1 = on; unset/anything else = off |
| `VICTORY_SEND_CHANGED` | Sends up only the dwords that title memory actually changed when refreshing a card-side copy, instead of whole written pages. | off | 0 or unset = off; anything else = on |
| `VICTORY_SEND_OWN` | Sends to card-side copies the bytes as they were when the submission was prepared. | on | 0 = off; anything else = on |
| `VICTORY_SHADER_CENSUS` | Appends one line per translated program counting the console and host instructions by kind to the given file. | off | &lt;output file path> |
| `VICTORY_SHADER_FRESH` | Makes a normal run translate shaders again when the cache entry came from an earlier translator build. | off | 1 = on; unset/anything else = off |
| `VICTORY_SHADER_KEY` | Chooses how cached translations are keyed: by translator version number or by a hash of its sources. | translator version number | source = key by hash of translator sources; anything else = version number |
| `VICTORY_SHADER_LOOP_LIMIT` | Sets how many iterations a translated shader loop may run per lane before it is left; only read in diagnostics builds. *Diagnostics build only.* | 1048576 (2^20) | &lt;iterations, decimal or 0x hex, at least 1>; diagnostics builds only |
| `VICTORY_SHADER_OPT` | Chooses which clean-up passes run over a translated program before it is written. | 63 (every pass but the last) | 0 = none; &lt;bitmask, decimal or 0x hex>: 1 dead code, 2 constants, 4 one instruction per value, 8 branch facts, 16 one read per place, 32 first-lane read, 64 four-dword loads |
| `VICTORY_SHADER_SAVE` | Saves each shader as the title gave it (.cdxil) and as translated (.dxil), named by its hash, into the folder. | off | &lt;folder path> |
| `VICTORY_SHADER_SOURCE_HASH` | Build-time value: hash of the translator's sources, mixed into the shader cache key; set by the build script. | set at build time | not read at run time; build-script value |
| `VICTORY_SHADER_SWITCHES` | Build-time value: the generated list of switches the translator reads, used to check each is keyed; set by the build script. | set at build time | not read at run time; build-script value |
| `VICTORY_SHOW_AFTER` | Shows a texture new to a slot only once the submission with its upload has finished. *Set by `scripts\play.ps1`.* | off | 0 or unset = off; anything else = on |
| `VICTORY_SHOW_NEW_LATER` | Holds back a slot's new texture made while earlier submissions may still run until those have finished. | on | 0 = off; anything else = on |
| `VICTORY_SKIP_BINK` | Skips the title's Bink movies while its own Bink DLL stays loaded. | off | 1 = skip movies that do not loop; all = skip every movie; 0 or empty = off |
| `VICTORY_SKIP_CPU_CHECK` | Lets the title start on a CPU without AVX2, which the startup check would otherwise refuse (e.g. under CPU emulation). | off (AVX2 required) | 1 = skip the check; anything else = checked |
| `VICTORY_SKIP_PIPELINES` | Leaves out pipelines of the listed shader hashes so their draws and dispatches are dropped. *Set by `scripts\play.ps1`.* | none | &lt;16-digit hex shader hashes, comma-separated> |
| `VICTORY_SLOT_CHECK` | Runs a thread that walks the view heap and reports slots naming memory that is already gone. *Diagnostics build only.* | off | 0 or unset = off; anything else = on |
| `VICTORY_SLOT_EVICT` | Under pressure on the card's memory, lets go of idle host copies of title-filled textures although title slots a table showed still name them; the slots show nothing until the title uses the texture again. | off | 1 = on; unset/other = off |
| `VICTORY_SLOT_EVICT_LOW_PERCENT` | Sets the share of the card's budget under which `VICTORY_SLOT_EVICT` stops evicting. | the high percent less 10 | &lt;percent, integer>, clamped to 40 .. high less 1 |
| `VICTORY_SLOT_EVICT_MB` | Caps the bytes `VICTORY_SLOT_EVICT` lets go in one sweep. | 256 | &lt;MiB, integer>, clamped to 16-2048 |
| `VICTORY_SLOT_EVICT_PERCENT` | Sets the share of the card's budget above which `VICTORY_SLOT_EVICT` starts evicting. | 85 | &lt;percent, integer>, clamped to 50-97 |
| `VICTORY_SLOT_HOLDS` | Makes a descriptor slot hold the host memory of a constant buffer view it points at, not only the list. | on | 0 = off; anything else = on |
| `VICTORY_SNAPSHOT_POOL` | Sets how many snapshot buffers of copies in PC memory are kept for reuse. | 1024 | &lt;buffers kept, integer> |
| `VICTORY_SPANS_QUICK` | Lets a slot write of no placed texture view leave the heap's placed-spans set alone when the slot is not in it. | on | 0 = off; anything else = on |
| `VICTORY_SPARES_BY_HEADROOM` | Keeps empty texture blocks according to the card's headroom below 85 % of budget instead of all-or-nothing. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/anything else = off |
| `VICTORY_SPILL_STRICT` | Follows spilled registers through lane moves with modifiers or lanes past the wave width only strictly; 0 restores the looser way. | on | 0 = off; anything else = on |
| `VICTORY_SPLIT_WAITS` | Cuts a submission at its waits for memory values so the parts before can run, instead of holding the whole submission. | on | 0 = off; anything else = on |
| `VICTORY_STACKS` | Prints every thread's stack as module+rva at each listed time after start. *Diagnostics build only.* | off | &lt;seconds after start>, comma-separated list |
| `VICTORY_STAGED_ASK_ALL` | Asks about every texture a pass over index-picked textures takes, through committed memory checks, instead of only some. | off | 1 = on; unset/anything else = off |
| `VICTORY_STALE_TEXTURES` | Chooses when a view the title writes makes older views of other memory stale, so their host texture copies are refreshed. | from the start of the block of smallest levels | 0 = never; overlap = wherever spans overlap; anything else = default rule |
| `VICTORY_STORE_BESIDE` | Lets the layer's own memcpy/memset replacements also handle the title's stores beside tracked memory, noticing them without a page fault. | off | 1 = on; unset/other = off |
| `VICTORY_STREAMLINE_DIR` | The folder with NVIDIA Streamline's DLLs (each checked to be signed by NVIDIA before loading). *Kept by `scripts\play.ps1` from the window it runs in.* | &lt;data_dir>\nvidia-sdk\Streamline\bin\x64 (`scripts\get-nvidia-sdks.ps1` puts them there) | &lt;folder path> |
| `VICTORY_STREAMLINE_LOG` | Writes Streamline's own verbose log into this folder. *Diagnostics build only.* | not set (no log) | &lt;folder path>; empty or 0 = no log |
| `VICTORY_STREAMLINE_PRESENT` | With DLSS and no frame generation, our swap chain is Streamline's proxy of it, so that Streamline sees the presents. | off | 1 = on; unset/other = off |
| `VICTORY_STREAM_NOTICE` | Tells the GPU side which bytes DirectStorage has just put into memory, so streamed data is picked up without scanning. | on | 0 = off; anything else = on |
| `VICTORY_STREAM_PRIORITY` | Raises the title's below-normal streaming thread to above normal where it maps and submits, so it gets a core. | on | 0 = off; anything else = on |
| `VICTORY_STRICT_JOINS` | Makes the shader translator stop on a value without a DXIL form at a join instead of silently turning it into zero. | off | any value (even empty) = on |
| `VICTORY_SUBMIT_DEPTH` | Limits how many queue submissions may wait in the line before a submit call returns. | 64 | &lt;count of submissions, integer>; minimum 1 |
| `VICTORY_SUBMIT_FRAMES` | Limits how many presents may wait in the line before a present call returns. *Set by `scripts\play.ps1`.* | 1 | &lt;count of presents, integer>; 0 = both the previous and the current frame are sent |
| `VICTORY_SWEEP` | Once a second, lets go of host copies, staging buffers and texture copies that nothing names any more. | on | 0 = off; anything else = on |
| `VICTORY_SWEEP_PER_HEAP` | Runs the heap buffer pass once per bound view heap each frame instead of once for all heaps. | off | 1 = on; unset/other = off |
| `VICTORY_SYNC_FINE` | Hands each thread fewer descriptor pages and records at a time when syncing heaps, so the work spreads across more threads. *Set by `scripts\play.ps1`.* | off | any value except 0 = on; 0/unset = off |
| `VICTORY_SYNC_IN_PLACE` | Reads a changed descriptor page's records in place and compares each with the last seen, instead of copying the page first. | on | 0 = off; anything else = on |
| `VICTORY_TABLED_REREAD` | A later table read in the same frame takes out a slot written after the frame's first read. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/other = off |
| `VICTORY_TABLED_TEXTURES` | Keeps textures that draws read through tables in the queue of textures picked by index, as before. | off (tabled slots leave the queue) | 1 = keep them in the queue as before; unset/other = leave the queue |
| `VICTORY_TAKE_PARTS` | When a write-watched page was taken before, copies only the parts written since where known, not the whole page. | on | 0 = off; anything else = on |
| `VICTORY_TARGET_VARIANTS` | Gives a draw into views of other formats than its pipeline declares a host pipeline built for those formats. | on | 0 = off; anything else = on |
| `VICTORY_TEST_ROUNDS` | Sets how many rounds a landing stress unit test runs. | 120 (tests only) | &lt;number of rounds, integer> |
| `VICTORY_TEST_TITLES` | Lists folders or files of real title data for tests that need them; those tests skip when it is unset. | none (tests needing game files skip) | &lt;paths separated by the OS path separator (; on Windows)> |
| `VICTORY_TEXBACK_ASYNC` | Writes texture read-backs into title memory on a thread of their own. | on | 0 = written on the landing thread, as before; anything else = on |
| `VICTORY_TEXTURE_AWAY_MS` | Waits this long before a texture slot whose memory is unmapped goes null, so surfaces do not flash black. | 3000 | &lt;milliseconds, integer>; 0 = at once |
| `VICTORY_TEXTURE_BACK` | Picks which shader-written texture kinds are copied back to title memory so the title's CPU can read them (the ocean maps). | the four water kinds (256x256f41, 256x256f16, 64x64f16, 128x128f36) | 1 = default kinds; 0 = none; &lt;WxHf&lt;format number>,... list of kinds> |
| `VICTORY_TEXTURE_BUDGET_MB` | Caps the bytes of host copies of title-filled textures made for index-picked shaders. | none (share of the card's budget; 4 GB without a budget) | &lt;megabytes, integer> = cap on bytes of host texture copies |
| `VICTORY_TEXTURE_CATCHUP` | Gives a large backlog of textures three times the usual bytes and time per frame until it drains. *Set by `scripts\play.ps1`.* | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_COPY` | Reads a fully mapped texture's memory through the kernel's copy instead of in place. | off | 1 = on; unset/other = off |
| `VICTORY_TEXTURE_FILL` | Lets a refresh that does not wait still fill a never-filled staging buffer from the pages present, instead of leaving zeros. | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_GUARD` | Uses a write guard on texture memory and hashes it only every eighth frame, instead of hashing it whole at every check. | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_HOT` | Treats textures the title rewrites every frame through another view as hot and checks them each frame, not every eighth. | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_KEEP` | Keeps idle host texture copies longer while the card has free memory, instead of releasing them after 5 seconds. | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_LOOK_MS` | Sets the longest gap between looks at texture slots whose memory has not arrived yet. | not set (about 4 s counted in passes) | &lt;milliseconds, integer>; 0 or invalid = not set |
| `VICTORY_TEXTURE_NO_WAIT` | Lets a texture refresh continue without waiting while the runtime is remapping title memory. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/other = off |
| `VICTORY_TEXTURE_ONCE` | Fills a staging buffer made for a single texture upload without consulting the write watch and guards. | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_POOL` | Makes blocks for textures ahead of time from a pool, so one is ready when a texture needs memory. | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_ROOM_PERCENT` | Sets the share of the card's budget up to which host copies of index-picked textures are made; unset, the line is the budget less max(768 MiB, 5 %), never under 90 %. | not set (about 93 % on a 12 GB card) | &lt;percent, integer>, clamped to 50-97; 90 = the line as before |
| `VICTORY_TEXTURE_SAME` | Skips an upload when title memory holds the same bytes the host texture copy was already filled from. | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_STAGE_MB` | Sets how many megabytes of index-picked textures are made and staged per frame pass. | not set (computed from the card's room) | &lt;megabytes per frame pass, integer> |
| `VICTORY_TEXTURE_STAGE_MS` | Sets the time limit per frame pass for making and staging index-picked textures. *Set by `scripts\play.ps1`.* | 250 | &lt;milliseconds per pass, integer> |
| `VICTORY_TEXTURE_STORE` | Keeps the larger levels of streamed textures in an on-disk store for reuse. | off | 1 = textures folder of the cache; &lt;folder path>; empty/0 = off |
| `VICTORY_TEXTURE_SWEEP` | Lets go of host texture copies that have been idle for several frames, before a submission starts. | on | 0 = off; anything else = on |
| `VICTORY_TEXTURE_VOLATILE` | Re-uploads a texture the title rewrites between recording and submission in every list that samples it, to stop flicker. | on | 0 = off; anything else = on |
| `VICTORY_THIN` | Makes command list calls only record their arguments, with threads of ours working out their meaning beside the title's. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/other = off |
| `VICTORY_THIN_THREADS` | Sets how many worker threads work out the recorded thin list calls. | 8 | &lt;thread count, integer>; clamped to 1-32 |
| `VICTORY_THREAD_DIMENSIONS` | Treats dispatch counts flagged as thread dimensions as threads, not groups, fixing an overrunning structure-builder stage. | on | 0 = off; anything else = on |
| `VICTORY_THREAD_TICKET` | Gives multi-wave groups on 32-lane hosts console thread ids from a group counter, so waves do not share ids. | on | 0 = off; anything else = on |
| `VICTORY_TILES_TEST_CARD` | Makes the tiled-resource unit tests use the default hardware adapter instead of the software one. | off | 1 = on; unset/other = off |
| `VICTORY_TIMELINE` | Writes a timeline file of a few frames, starting at the first present after a set time. *Diagnostics build only.* | off | &lt;output file path>, optionally with a time spec; empty = off |
| `VICTORY_TIMELINE_AT` | Sets the seconds into the run after which the timeline starts recording. *Diagnostics build only.* | 60 | &lt;seconds into the run, number> |
| `VICTORY_TIMELINE_FRAMES` | Sets how many frames the timeline records. *Diagnostics build only.* | 3 | &lt;number of frames> |
| `VICTORY_TIMER_LOG` | Logs how even the frame times are, every 5 seconds, in bands. | off | 1 = on; unset/other = off |
| `VICTORY_TITLE_FPS` | Overrides the frame rate the title is told to run at when the frame interval switch is set. | derived from VICTORY_FRAME_INTERVAL_MS | &lt;frames per second>; off = leave the title's own rate; only used with VICTORY_FRAME_INTERVAL_MS |
| `VICTORY_TITLE_PRIORITIES` | Lowers the title's lowest two thread priority levels to below normal, unless this is set to 0. *Set by `scripts\play.ps1`.* | on (lowest two priorities become below normal) | 0 = pass priorities through as the title sets them; anything else = on |
| `VICTORY_TRAIL` | Keeps a short trail of what the layer did to host objects and prints it when something goes wrong. | on | 0 = off; anything else = on |
| `VICTORY_TRANSLATE_ONLY` | Limits the offline shader translation tool to containers whose hash starts with one of these prefixes. | all containers | &lt;hash prefixes, hex, comma-separated> |
| `VICTORY_TRANSLATOR` | Build-time stamp of the translator's sources, used to key the shader cache; not set by hand. | set at build time by build.rs | not a runtime switch; BLAKE3 of the translator sources |
| `VICTORY_TRAP_LOG` | Names the crash log file used by a loader unit test of an unimplemented import trap. | not set (test only) | &lt;crash log file path> |
| `VICTORY_TWIN_EXACT` | Starts raw twins of buffer views that are not 16-byte aligned at the view's own dword instead of making them null. | on | 0 = off; anything else = on |
| `VICTORY_UI_PLANE` | Puts the title's interface on a plane of its own at 3840x2160, scaled over a smaller output. *Set by `scripts\play.ps1`.* | off | 1 = on; unset/other = off |
| `VICTORY_UNDER_FUSED` | Copies the title's bytes into the CPU side of a card copy while comparing, in 32-byte blocks, instead of finding runs first. | on | 0 = off; anything else = on |
| `VICTORY_UNDER_THREADS` | Sets how many threads handle the title's bytes that go in under in-flight shader writes. | 6 | &lt;thread count, integer>; clamped to 1-16 |
| `VICTORY_UNDER_WIDE` | Compares pages of in-flight shader writes on several threads instead of on the submitting thread. *Set by `scripts\play.ps1`.* | on | 0 = off; anything else = on |
| `VICTORY_UNDER_WORDS` | Merges the title's bytes under in-flight shader writes by whole words instead of byte by byte. | on | 0 = off; anything else = on |
| `VICTORY_UNWATCHED_FRAMES` | Sets how often memory without a write watch is refreshed whole, in frames. | 1 (every frame) | &lt;frames between refreshes, integer> |
| `VICTORY_UPCOPY_CHECK` | Checks whether bytes sent up to a card copy are older than what the card holds, and logs it. | off | 1 = on; unset/other = off |
| `VICTORY_UPLOAD_AHEAD` | Does first uploads of index-picked textures on a separate copy queue, read and detiled by threads of ours ahead of time. *Set by `scripts\play.ps1`.* | off | any value except 0 = on; 0/unset = off |
| `VICTORY_UPLOAD_LOW` | Keeps low-address upload memory in its own arena directory instead of as heap data. | on | 0 = off; anything else = on |
| `VICTORY_UPLOAD_POOL` | Reuses a list's shared-memory upload buffers instead of making and releasing one per upload. | off | 1 = on; unset/other = off |
| `VICTORY_UPSCALER` | Chooses which upscaler the title is told to use: its own or FidelityFX. | not set (title's own choice) | 6 = the title's own; 7 = FidelityFX; other = refused with a message |
| `VICTORY_UP_QUEUE` | Sends changes on the CPU side of card copies up on a copy queue of its own. | off | 1 = on; unset/other = off |
| `VICTORY_VIEW_BYTES` | Gives views made off a copy's elements a copy of their own bytes instead of leaving them null. | on | 0 = off; anything else = on |
| `VICTORY_VIEW_COPY` | Makes descriptor views on a staging area, written whole before use, rather than in place where they can be seen half written. | on | 0 = off; anything else = on |
| `VICTORY_VIEW_EXTENT` | Brings a buffer view up to date in the host copy its slot shows, not by range alone. | on | 0 = off; anything else = on |
| `VICTORY_VIEW_GUARD` | Guards all views of a guarded memory page together when one is mapped anew. | on | 0 = off; anything else = on |
| `VICTORY_VIEW_INLINE` | Names constant buffer and placed-memory views whole in their records instead of as table entries. | on | 0 = off; anything else = on |
| `VICTORY_VIEW_LIMIT` | Makes the offline translate command translate as for a host whose view heap is capped at this null slot. | not set | &lt;view heap slot number, integer> |
| `VICTORY_VIEW_THREADS` | Sets how many threads make descriptors for index-picked views (records use a separate larger pool). | 4 (6 for records) | &lt;thread count, integer>; clamped to 1-16; 1 = on the calling thread alone |
| `VICTORY_VRAM_BUDGET_MB` | Caps what is taken as the card's memory budget, whatever the system gives; the texture line, card copies of title buffers and the render-target reserve are all sized from it. | not set (the system's budget) | &lt;MiB, integer>; at least 2048 |
| `VICTORY_WAIT_ASK` | Asks Windows whether a fence value's memory is committed before reading it. | off | 1 = on; unset/other = off |
| `VICTORY_WAIT_FORCE` | Makes a present run work queues still hold, so a wait is not held across frames. | on | 0 = off; anything else = on |
| `VICTORY_WAIT_RESIDENT` | Makes new memory resident and waits for it before anything can name it, avoiding page faults on the card. | on | 0 = off; anything else = on |
| `VICTORY_WALLS_TRACE` | Logs what the title's CPU was given while it decided whether the camera is in a room. | off | 1 = trace; 2 = trace every request too; other = off |
| `VICTORY_WARM_SECONDS` | Stops the shader warm-up from taking new shaders after this many seconds. | no limit | &lt;seconds, integer> |
| `VICTORY_WARP` | Runs shader tests and host creation on the software WARP adapter instead of the hardware one. | off | 1 = on; unset/other = off |
| `VICTORY_WATCH` | Logs the 32-bit value at each listed title address whenever it changes, polled every 5 ms. *Diagnostics build only.* | off | &lt;hex RVAs of title addresses, comma-separated> |
| `VICTORY_WAVE_DEEP_SKIPS` | Allows the exact translation forms that cost group barriers for console wave64 programs. | on | 0 = off; anything else = on |
| `VICTORY_WAVE_DUMP` | Prints the LDS-flag analysis details of console wave64 programs while translating. | off | any value = on |
| `VICTORY_WAVE_LOCAL` | Translates wave operations as per-host-wave where exact; idiom refuses the approximate cases instead. | on | 0 = off; idiom = refuse approximate cases; anything else = on |
| `VICTORY_WAVE_OWN_BITS` | Accepts a wave OR where not every lane runs, where the lanes' own bits can be told apart. | on | 0 = off; anything else = on |
| `VICTORY_WAVE_WHY` | Prints why a wave64 region was refused translation, by its LDS accesses. | off | any value = on |
| `VICTORY_WAVE_WIDE` | Translates console wave64 programs as two host waves (wide form); unset widens only one narrow case. | unset: only one narrow case is widened | 1 = wide form on; 0 = never wide; unset = narrow case only |
| `VICTORY_WAVE_WIDE_VOLUMETRIC` | The volumetric light's two tiled programs and four more 64-thread LDS programs refused on RTX cards (VIctory-NVIDIA pull request 101) take the wave-wide form before any approximate one. Changes translations. | off | 1 = on (ignored while `VICTORY_WAVE_WIDE` is set); unset/other = off |
| `VICTORY_WC_BAR` | Gives a write-combined title buffer one copy in card memory the CPU maps. | off | 1 = on; unset/other = off |
| `VICTORY_WC_NO_BACK` | Leaves what shaders write in card copies of write-combined title memory on the card (no compare, fetch or merge back). *Set by `scripts\play.ps1`.* | off | 1 = on; unset/other = off |
| `VICTORY_WC_NO_BACK_MIN_KB` | The smallest copy `VICTORY_WC_NO_BACK` and `VICTORY_WC_BAR` apply to (smaller ones went near black in cutscenes). | 1024 | &lt;KiB, integer>; 0 = every copy |
| `VICTORY_WIDE_BACK` | Takes snapshots of shader-written ranges and compares them across several threads, one range per thread. *Set by `scripts\play.ps1`.* | on | 0 = off; anything else = on |
| `VICTORY_WIDE_FILLS` | Does a submission's upload memory fills together on several threads. | on (needs WIDE_REFRESH on) | 0 = off; anything else = on |
| `VICTORY_WIDE_MERGE` | Merges shader writes to card copies into title memory on several threads. | off | 1 = on; unset/other = off |
| `VICTORY_WIDE_REFRESH` | Runs a submission's refreshes of host copies on several threads, one copy per thread. | automatic from the core count | 0 = off; 1 = force on; unset/other = automatic |
| `VICTORY_WIDE_SPIN` | Lets worker threads wait awake for up to this many microseconds before sleeping. | 0 (sleep, no spinning) | &lt;microseconds, integer>; 0 = do not spin |
| `VICTORY_WIDE_TEXTURES` | Brings a submission's textures up to date on several threads. | on (needs WIDE_REFRESH on) | 0 = off; anything else = on |
| `VICTORY_WINDOWS_BAR_MB` | Caps the bytes of memory windows kept in the card's memory and filled by the CPU over the bus. | 0 (none) | &lt;megabytes, integer> |
| `VICTORY_WINDOWS_OWN` | Gives windows outside buffers a second side, so in-flight submissions read the bytes they were prepared with. | off | 1 = on; unset/other = off |
| `VICTORY_WINDOW_BUDGET_MB` | Caps loose title-window copies in the watch and high-RAM memory bands. | none (no cap) | &lt;megabytes, integer>; 0 = no cap |
| `VICTORY_WINDOW_KB` | Sets the grid that memory windows are cut on, in kilobytes. | 4096 (4 MiB) | &lt;kilobytes, power of two, at least 64>; smaller or invalid = default |
| `VICTORY_WINDOW_MAX` | Caps the title's top-level window size so it fits the host's screen. | none | &lt;width>x&lt;height> in pixels |
| `VICTORY_WINDOW_TAILS` | Makes a window the cell's tail from the ask's step on, for finer window grids. | off | 1 = on; unset/other = off |
| `VICTORY_WRITE_SITES` | Names a file where code addresses whose stores usually leave memory unchanged are recorded for the title-writes tracker. | off | &lt;file path> |
| `VICTORY_WRITE_SKIP` | Declares a shader-bound buffer range as written only if that shader's code can store to buffers. | on | 0 = off; anything else = on |
| `VICTORY_XMA_DUMP` | Saves the context words and input of the first 32 enabled XMA voices to a folder, for testing the decoder. *Diagnostics build only.* | off | &lt;output folder path> |
| `VICTORY_XMA_REFERENCE` | Points an ignored decoder test at a folder of reference XMA files and reference decodes. | not set (test only) | &lt;folder path with mono/stereo xmadata and ref files> |
| `VICTORY_XMA_REFILL` | Lets an ended XMA voice go on when the title gives it input again, publishing only whole blocks. | off | 1 = on; unset/other = off |
| `VICTORY_XMA_STREAMS` | Points an ignored decoder test at a folder of saved XMA input streams and context words. | not set (test only) | &lt;folder path of .xma files with .ctx files> |
