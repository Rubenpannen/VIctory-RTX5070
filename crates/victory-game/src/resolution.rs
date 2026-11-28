//! The title's output size and render scale (`VICTORY_OUTPUT`,
//! `VICTORY_RENDER_SCALE`, `VICTORY_FFX_SCALE`, `VICTORY_UPSCALER`; each off
//! without it) and its FidelityFX path for DLSS (when the app asks): places
//! in the title's code changed at load, each compared byte for byte first
//! and left alone, said so, if it differs (another build of the title).
//!
//! What the title does (game_scarlett_master.exe 1.0.2.1, image offsets):
//!
//! - **Output size.** Its settings (`settings.xml` in its storage,
//!   `video/resolutionFullscreen`, 3840x2160 as a Series X wrote it; read
//!   into the live settings at +0x204/+0x208, the defaults' 1920x1080 at
//!   +0x5b37a20 overwritten) are applied by +0x22434a0: from +0x224352b it
//!   hands them to +0x2478130, the one writer of the screen size
//!   (+0xa9438cc/+0xa9438d0), and when the height changes calls +0xd8c3a0.
//!   The back buffers (+0x247ad10, by +0x501f4a0 and +0xcc2780) take that
//!   size through the copy +0x5eb24f0 makes (+0x75e6fe0). The display mode
//!   list it matches against is its own (+0x5a38910: 1280x720, 1664x936,
//!   1920x1080, 2560x1440, 3840x2160 at 60 Hz, table +0x615c0c0), not the
//!   DXGI output's, which is why `VICTORY_DISPLAY_MODE` changes nothing.
//!   Here that block becomes: write the size asked for into the settings
//!   (+0x204/+0x208), then the title's own else-path (screen size set,
//!   +0xd8c3a0 if the height changed). Everything that reads the settings
//!   after it (the device parameters +0x1e0fb30, the render size +0x7b4e10)
//!   sees the same size. The back buffers' pool (64 MB on Series X) is
//!   checked to be used up but for 2 MB (+0x247af25, an `int3`); for another
//!   size that check is jumped over.
//! - **Render scale.** The upscaler the title picks (+0x7e93f0: 6, its own,
//!   except on Series S with a flag, 7, AMD FidelityFX) gives the render
//!   size from the output size (+0x7e92f0 → the upscaler's +0x50 method):
//!   its own (+0x7e9440) multiplies by a scale at +0x90 of it, set to 2/3 at
//!   +0x28f1e78 (so 2560x1440 for 4K, 1280x720 for 1080p);
//!   `VICTORY_RENDER_SCALE` replaces that constant (Series X; run res-4 at
//!   1.0). FidelityFX's (+0x28ef680, Series S) multiplies by 1.0 on Series X,
//!   2/3 on Series S; `VICTORY_FFX_SCALE` points its two loads at a constant
//!   in its padding (as pull request #53 did). That one is not safe: at 0.5
//!   on Series S (run res-6) FidelityFX's jitter phase count came back 0 and
//!   the title divided by it (+0x28f03e8) 49 s in. That may be the cause
//!   VIctory-NVIDIA found in 4fcaebd (unverified here): the shipping FSR4
//!   makes its context at [`SHIPPING_FSR4_RENDER`] in and
//!   [`SHIPPING_FSR4_OUTPUT`] out only, 960x540 fails its creation, and the
//!   title, which does not look at the result, divides by zero at its first
//!   upscaling.
//! - **Upscaler.** `VICTORY_UPSCALER=<kind>` makes the picker return
//!   `kind` (pull request #53): 6 the title's own, 7 FidelityFX.
//! - **The FidelityFX path for DLSS.** With DLSS or frame generation
//!   available (NVIDIA Streamline loaded, `VICTORY_DLSS=1` /
//!   `VICTORY_FRAMEGEN=1`) or `VICTORY_FSR_PATH=1`, the app passes an
//!   [`Upscaler`] (`VICTORY_DLSS_MODE`'s FidelityFX quality mode and scale)
//!   and three places are changed together, or none if one differs: the
//!   picker returns 7 on either console (Series S picks it already when its
//!   flag is set); FidelityFX's render size method notes the mode at +0x90
//!   of the upscaler (`[r14+90h]`, where the title notes the console
//!   check's result: 0 on Series X, 1, quality, on Series S) and takes the
//!   scaled branch, whose load reads the mode's scale from the padding (as
//!   `VICTORY_FFX_SCALE` writes it); the title's own upscaler's scale is the
//!   mode's too, as the stock image pairs the two (2/3 each). The title's
//!   upscale dispatches then reach `victory_hle::modules::fidelityfx`, where
//!   DLSS takes them. `VICTORY_RENDER_SCALE`, `VICTORY_FFX_SCALE` and
//!   `VICTORY_UPSCALER` are left out then (said so). Without Streamline (its
//!   DLLs missing, a card it does not serve) and without `VICTORY_FSR_PATH`
//!   nothing of this is changed. The title's own FSR4 (no DLSS) makes its
//!   context only at the shipping sizes ([`SHIPPING_FSR4_RENDER`]), so
//!   `VICTORY_FSR_PATH=1` alone is expected to run only where the title asks
//!   for those (a 1920x1080 output at the quality scale); at other sizes the
//!   title may divide by zero at its first upscaling, as above. From VIctory-NVIDIA d11233a and 4fcaebd
//!   (readmet3xt-lab; there from DexNom's fork, `prepare_fsr_4k`), which
//!   change the same instructions in the file before it is mapped. A
//!   possible later place: the Settings menu's viewport load (+0x58048d4),
//!   which reads the 4K interface size where VIctory-NVIDIA's 1440p output
//!   points it at the display mode's.
//! - **Interface plane.** On Series X the interface is laid out in 4K
//!   pixels: the settings menu, subtitles and button prompts keep their 4K
//!   size at a smaller output (at 1080p twice too large, cut off at the
//!   right and bottom). The title's UI is sized by the screen (+0x1de4030
//!   writes it to +0x92d7924/+0x92d7928, +0x5c61a0 scales by it against
//!   3840x2160), but its Flash menus are not. The settings' video
//!   `uiOverlay` (+0x1e8) and `uiResolution` (+0x1ec/+0x1f0), read by the
//!   render size +0x7b4e10 and the UI's start-up step +0x26192a0, put the
//!   interface on a plane of its own at that size, which the display (here
//!   `compose`, `VICTORY_PLANES`) scales over the scene, as on Series S.
//!   `VICTORY_UI_PLANE=1` sets them (overlay on, 3840x2160) where the output
//!   block writes the output size, through a routine in the padding at
//!   +0x55ff290 (nothing refers to +0x55ff284..+0x55ff2c0); without it the
//!   block clears `uiOverlay`, which the title keeps in its settings file.
//!   The back buffers' pool (`mov ecx` at +0x247ad4a; Series S takes 82 MB
//!   from the `mov eax` before it) is made room for both planes. From
//!   avatarmanz/VIctory-7900XTX pull request #295 (e55673a0, UwUCammie),
//!   which did it for 1280x720.

use crate::frame_rate::Patched;

/// Sizes the title has a display mode for (its own list, +0x615c0c0).
pub const MODES: [(u32, u32); 5] = [(1280, 720), (1664, 936), (1920, 1080), (2560, 1440), (3840, 2160)];

/// `VICTORY_OUTPUT=<w>x<h>`: one of [`MODES`]. `None` if unset or not one
/// (said why).
pub fn asked_output() -> Option<(u32, u32)> {
    let spec = std::env::var("VICTORY_OUTPUT").ok()?;
    let size = parse_output(&spec);
    if size.is_none() {
        eprintln!("VIctory: VICTORY_OUTPUT={spec}: expected one of the title's modes: 1920x1080, 2560x1440, 1280x720, 1664x936, 3840x2160");
    }
    size
}

pub fn parse_output(spec: &str) -> Option<(u32, u32)> {
    let (w, h) = spec.trim().split_once(['x', 'X'])?;
    let size = (w.trim().parse().ok()?, h.trim().parse().ok()?);
    MODES.contains(&size).then_some(size)
}

/// `VICTORY_RENDER_SCALE=<s>`, 0.5 to 1.0 (0.5: half the output's width and
/// height): the title's own upscaler's scale (Series X; 2/3 without it).
/// Tested at 1.0 (run res-4); below 2/3 the title's largest render size
/// (+0x94, max(2/3, scale)) stays at 2/3. `None` if unset or out of range
/// (said why).
pub fn asked_render_scale() -> Option<f32> {
    let spec = std::env::var("VICTORY_RENDER_SCALE").ok()?;
    let scale = parse_render_scale(&spec);
    if scale.is_none() {
        eprintln!("VIctory: VICTORY_RENDER_SCALE={spec}: expected 0.5 to 1.0");
    }
    scale
}

pub fn parse_render_scale(spec: &str) -> Option<f32> {
    spec.trim().parse::<f32>().ok().filter(|s| (0.5..=1.0).contains(s))
}

/// `VICTORY_FFX_SCALE=<s>`, 0.5 to 1.0: FidelityFX's render scale (Series
/// S). Crashed at 0.5 (run res-6); see the module's notes.
pub fn asked_ffx_scale() -> Option<f32> {
    let spec = std::env::var("VICTORY_FFX_SCALE").ok()?;
    let scale = parse_render_scale(&spec);
    if scale.is_none() {
        eprintln!("VIctory: VICTORY_FFX_SCALE={spec}: expected 0.5 to 1.0");
    }
    scale
}

/// `VICTORY_UPSCALER=<kind>`: 6 (the title's own) or 7 (FidelityFX).
pub fn asked_upscaler() -> Option<u32> {
    let spec = std::env::var("VICTORY_UPSCALER").ok()?;
    let kind = spec.trim().parse::<u32>().ok().filter(|k| matches!(k, 6 | 7));
    if kind.is_none() {
        eprintln!("VIctory: VICTORY_UPSCALER={spec}: expected 6 (the title's own) or 7 (FidelityFX)");
    }
    kind
}

/// The size the title lays its interface out at on Series X.
pub const UI_SIZE: (u32, u32) = (3840, 2160);

/// `VICTORY_UI_PLANE=1`: the interface on a plane of its own at
/// [`UI_SIZE`], scaled over a smaller output (see the module's notes; off
/// without it). Says why when it cannot be.
pub fn asked_ui_plane(output: (u32, u32)) -> bool {
    let asked = std::env::var("VICTORY_UI_PLANE").ok().as_deref() == Some("1");
    let planes_off = std::env::var_os("VICTORY_PLANES").is_some_and(|v| v == "0");
    if asked && planes_off && output != UI_SIZE {
        eprintln!("VIctory: VICTORY_UI_PLANE=1 needs the plane compositor, which VICTORY_PLANES=0 turns off: the interface stays on the scene");
    }
    ui_plane_applies(output, asked, planes_off)
}

/// Whether an interface plane is made: asked for, an output smaller than
/// the interface, and a compositor to show it.
pub fn ui_plane_applies(output: (u32, u32), asked: bool, planes_off: bool) -> bool {
    asked && !planes_off && output != UI_SIZE
}

/// The title's video settings, as offsets from them: `uiOverlay` (a byte)
/// and `uiResolution`'s width and height.
pub const UI_OVERLAY: u32 = 0x1e8;
pub const UI_WIDTH: u32 = 0x1ec;
pub const UI_HEIGHT: u32 = 0x1f0;

const SERIES_X_POOL: [u8; 4] = (64u32 << 20).to_le_bytes();

/// +0x247ad10: `mov ecx, 64 MB`'s constant, the back buffers' pool on
/// Series X (the `mov eax` before it holds Series S's 82 MB).
pub const POOL: Site = Site {
    at: 0x247ad4b,
    what: "back buffer pool's size (Series X)",
    bytes: &SERIES_X_POOL,
};

/// The pool with an interface plane: the 64 MB the title sizes for 4K (now
/// the interface's), the scene's share of that at `width` x `height`, and
/// 8 MB to spare, in whole 2 MB; at least Series S's 82 MB.
pub fn ui_plane_pool(width: u32, height: u32) -> u32 {
    const MB: u64 = 1 << 20;
    let scene = (64 * MB * u64::from(width) * u64::from(height)).div_ceil(3840 * 2160 * MB);
    let mb = (64 + scene + 8).next_multiple_of(2).max(82);
    (mb * MB) as u32
}

const PADDING: [u8; 40] = [0xcc; 40];

/// Padding between two functions (+0x55ff284 to +0x55ff2c0; no code or data
/// refers into it): the two routines the output block calls.
pub const SPARE: Site = Site {
    at: 0x55ff290,
    what: "padding, for the interface plane's settings",
    bytes: &PADDING,
};

/// Where in [`SPARE`] the routine that turns `uiOverlay` off begins.
pub const UI_OFF_AT: usize = 32;

/// What goes at [`SPARE`]: at its start `mov byte [rsi+uiOverlay], 1 ;
/// mov dword [rsi+uiResolution], 3840x2160 ; ret`, at [`UI_OFF_AT`]
/// `mov byte [rsi+uiOverlay], 0 ; ret` (rsi: the settings, in the output
/// block).
pub fn spare_code() -> Vec<u8> {
    let mut c = vec![0xc6, 0x86];
    c.extend(UI_OVERLAY.to_le_bytes());
    c.push(1);
    for (field, value) in [(UI_WIDTH, UI_SIZE.0), (UI_HEIGHT, UI_SIZE.1)] {
        c.extend([0xc7, 0x86]);
        c.extend(field.to_le_bytes());
        c.extend(value.to_le_bytes());
    }
    c.push(0xc3);
    c.resize(UI_OFF_AT, 0xcc);
    c.extend([0xc6, 0x86]);
    c.extend(UI_OVERLAY.to_le_bytes());
    c.extend([0, 0xc3]);
    c.resize(SPARE.bytes.len(), 0xcc);
    c
}

/// A place to change: its offset in the image and the bytes there
/// (compared first; the replacement is at most as long).
pub struct Site {
    pub at: usize,
    pub what: &'static str,
    pub bytes: &'static [u8],
}

/// The screen size setter, its screen height, and what +0x22434a0 calls when
/// the height changed.
pub const SET_SCREEN_SIZE: usize = 0x2478130;
pub const SCREEN_HEIGHT: usize = 0xa9438d0;
pub const HEIGHT_CHANGED: usize = 0xd8c3a0;

/// +0x22434a0 from +0x224352b to +0x2243568: the settings' size handed to
/// the screen size.
pub const OUTPUT: Site = Site {
    at: 0x224352b,
    what: "settings' output size applied",
    bytes: &[
        0x8b, 0x0d, 0x9b, 0x03, 0x70, 0x08, // mov ecx, [+0xa9438cc]
        0x8b, 0x86, 0x04, 0x02, 0x00, 0x00, // mov eax, [rsi+204h]
        0x39, 0xc1, 0x75, 0x0d, // cmp ecx, eax ; jne
        0x8b, 0x96, 0x08, 0x02, 0x00, 0x00, // mov edx, [rsi+208h]
        0xe8, 0xea, 0x4b, 0x23, 0x00, 0xeb, 0x20, // call +0x2478130 ; jmp +0x2243568
        0x8b, 0xbe, 0x08, 0x02, 0x00, 0x00, // mov edi, [rsi+208h]
        0x8b, 0x2d, 0x7c, 0x03, 0x70, 0x08, // mov ebp, [+0xa9438d0]
        0x89, 0xc1, 0x89, 0xfa, // mov ecx, eax ; mov edx, edi
        0xe8, 0xd3, 0x4b, 0x23, 0x00, // call +0x2478130
        0x39, 0xfd, 0x74, 0x07, 0xb1, 0x01, // cmp ebp, edi ; je ; mov cl, 1
        0xe8, 0x38, 0x8e, 0xb4, 0xfe, // call +0xd8c3a0
    ],
};

/// +0x247ad10, after the back buffers are made in their pool (64 MB on
/// Series X, 82 MB on Series S, from +0x247ad45): `int3` unless less than
/// 2 MB of the pool is left over. Sized for 3840x2160; at 1920x1080 49 MB
/// are left (run res-1 stopped here). With another output size the check
/// becomes a jump over the `int3`; the pool stays as large.
pub const POOL_CHECK: Site = Site {
    at: 0x247af25,
    what: "back buffer pool's left-over check",
    bytes: &[0x48, 0x3d, 0xff, 0xff, 0x1f, 0x00, 0x76, 0x01, 0xcc], // cmp rax, 1FFFFFh ; jbe +1 ; int3
};

/// What goes at [`POOL_CHECK`]: `jmp +1` in place of `jbe +1`.
pub fn pool_check_code() -> Vec<u8> {
    let mut c = POOL_CHECK.bytes[..8].to_vec();
    c[6] = 0xeb;
    c
}

/// The title's own upscaler, its method +0x28f1e70: `mov dword [rcx+90h],
/// 2/3` (the scale +0x7e9440 renders at).
pub const OWN_SCALE: Site = Site {
    at: 0x28f1e78,
    what: "the title's own upscaler's render scale",
    bytes: &[0xc7, 0x81, 0x90, 0x00, 0x00, 0x00, 0xab, 0xaa, 0x2a, 0x3f],
};

/// FidelityFX's render size method, +0x28ef680, and its padding: the scale
/// is 1.0 (+0x5fc6ed8) or, on Series S, 2/3 (+0x615d95c), loaded at +0x28
/// and +0x32.
pub const FFX_SCALE: Site = Site {
    at: 0x28ef680,
    what: "FidelityFX's render scale",
    bytes: &[
        0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x83, 0xec, 0x28, 0x44, 0x89, 0xcb, 0x4c, 0x89, 0xc6, 0x48, 0x89, 0xd7, 0x49, 0x89,
        0xce, 0xe8, 0xa6, 0x5a, 0x78, 0xfd, 0x0f, 0xb6, 0xc0, 0x41, 0x89, 0x86, 0x90, 0x00, 0x00, 0x00, 0x84, 0xc0, 0x75, 0x0a,
        0xc5, 0xfa, 0x10, 0x05, 0x28, 0x78, 0x6d, 0x03, 0xeb, 0x08, 0xc5, 0xfa, 0x10, 0x05, 0xa2, 0xe2, 0x86, 0x03, 0xc5, 0xf2,
        0x2a, 0x54, 0x24, 0x70, 0xc5, 0xf2, 0x2a, 0xcb, 0xc5, 0xfa, 0x59, 0xc9, 0xc5, 0xfa, 0x2c, 0xc1, 0x89, 0x07, 0xc5, 0xfa,
        0x59, 0xc2, 0xc5, 0xfa, 0x2c, 0xc0, 0x89, 0x06, 0x48, 0x83, 0xc4, 0x28, 0x5b, 0x5f, 0x5e, 0x41, 0x5e, 0xc3, 0xcc, 0xcc,
        0xcc, 0xcc, 0xcc, 0xcc,
    ],
};

/// The upscaler picker, +0x7e93f0, to its first `ret`.
pub const PICKER: Site = Site {
    at: 0x7e93f0,
    what: "upscaler picker",
    bytes: &[
        0x48, 0x83, 0xec, 0x28, 0x80, 0x3d, 0x6d, 0xf0, 0x3a, 0x07, 0x00, 0x75, 0x09, 0x83, 0x3d, 0x68, 0xf0, 0x3a, 0x07, 0x00,
        0x74, 0x28, 0xe8, 0x35, 0xbd, 0x88, 0xff, 0x8a, 0x0d, 0x5b, 0xf0, 0x3a, 0x07, 0x80, 0xe1, 0x40, 0x80, 0xf9, 0x01, 0xb9,
        0x07, 0x00, 0x00, 0x00, 0x83, 0xd9, 0x00, 0x84, 0xc0, 0xb8, 0x06, 0x00, 0x00, 0x00, 0x0f, 0x45, 0xc1, 0x48, 0x83, 0xc4,
        0x28, 0xc3,
    ],
};

/// `rel32` from the end of an instruction at image offset `next` to `to`.
fn rel(to: usize, next: usize) -> [u8; 4] {
    ((to as i64 - next as i64) as i32).to_le_bytes()
}

/// What goes at [`OUTPUT`] for `width` x `height` (the same length), first
/// calling `routine` (an image offset, one of [`SPARE`]'s) if given.
pub fn output_code(width: u32, height: u32, routine: Option<usize>) -> Vec<u8> {
    let at = OUTPUT.at;
    let mut c = Vec::with_capacity(OUTPUT.bytes.len());
    if let Some(routine) = routine {
        c.push(0xe8); // call routine (rsi: the settings; changes no register)
        c.extend(rel(routine, at + c.len() + 4));
    }
    c.push(0xb8); // mov eax, width
    c.extend(width.to_le_bytes());
    c.extend([0x89, 0x86, 0x04, 0x02, 0x00, 0x00]); // mov [rsi+204h], eax
    c.push(0xbf); // mov edi, height
    c.extend(height.to_le_bytes());
    c.extend([0x89, 0xbe, 0x08, 0x02, 0x00, 0x00]); // mov [rsi+208h], edi
    c.extend([0x8b, 0x2d]); // mov ebp, [screen height]
    c.extend(rel(SCREEN_HEIGHT, at + c.len() + 4));
    c.extend([0x89, 0xc1, 0x89, 0xfa]); // mov ecx, eax ; mov edx, edi
    c.push(0xe8); // call the setter
    c.extend(rel(SET_SCREEN_SIZE, at + c.len() + 4));
    let end = OUTPUT.bytes.len();
    c.extend([0x39, 0xfd, 0x74, (end - (c.len() + 4)) as u8]); // cmp ebp, edi ; je end
    c.extend([0xb1, 0x01, 0xe8]); // mov cl, 1 ; call
    c.extend(rel(HEIGHT_CHANGED, at + c.len() + 4));
    c.extend([0xeb, (end - (c.len() + 2)) as u8]); // jmp end
    c.resize(end, 0xcc);
    c
}

/// What goes at [`OWN_SCALE`] for `scale`.
pub fn own_scale_code(scale: f32) -> Vec<u8> {
    let mut c = OWN_SCALE.bytes[..6].to_vec();
    c.extend(scale.to_le_bytes());
    c
}

/// What goes at [`FFX_SCALE`] for `scale`: both loads read `scale`, written
/// into the padding at +0x64.
pub fn ffx_scale_code(scale: f32) -> Vec<u8> {
    const LOADS: [usize; 2] = [0x28, 0x32];
    const SLOT: usize = 0x64;
    let mut c = FFX_SCALE.bytes.to_vec();
    c[SLOT..SLOT + 4].copy_from_slice(&scale.to_le_bytes());
    for l in LOADS {
        c[l + 4..l + 8].copy_from_slice(&((SLOT - (l + 8)) as i32).to_le_bytes());
    }
    c
}

/// Where in [`FFX_SCALE`] the console check's result is noted as the
/// FidelityFX quality mode and picks the branch (14 bytes): `movzx eax, al ;
/// mov [r14+90h], eax ; test al, al ; jne` to the scaled load.
pub const FFX_MODE_AT: usize = 0x1a;
/// Where in [`FFX_SCALE`] the scaled branch (Series S) loads its scale.
pub const FFX_SCALED_LOAD: usize = 0x32;

/// What goes at [`FFX_SCALE`] for the FidelityFX path at `scale` with
/// `ffx_mode` noted: [`ffx_scale_code`]'s loads of `scale`, and at
/// [`FFX_MODE_AT`] `mov eax, ffx_mode ; mov [r14+90h], eax ; jmp` to the
/// scaled load (the console check before it still runs; its result is not
/// used).
pub fn ffx_upscaler_code(ffx_mode: u32, scale: f32) -> Vec<u8> {
    let mut c = ffx_scale_code(scale);
    let mut noted = vec![0xb8];
    noted.extend(ffx_mode.to_le_bytes());
    noted.extend([0x41, 0x89, 0x86, 0x90, 0x00, 0x00, 0x00]); // mov [r14+90h], eax
    let end = FFX_MODE_AT + noted.len() + 2;
    noted.extend([0xeb, (FFX_SCALED_LOAD - end) as u8]); // jmp to the scaled load
    c[FFX_MODE_AT..end].copy_from_slice(&noted);
    c
}

/// The largest render and output sizes the title's shipping FSR4 makes an
/// upscaler context at: it carries only the Series S's fixed-resolution
/// quality tables. Other sizes fail the creation ("Shipping FSR4
/// configuration mismatch": the output must be 1920x1080, the render size
/// 1280x720 for a tight allocation or 1920x1080 for the legacy one), and
/// the title, which does not look at the result, divides by zero at its
/// first upscaling (VIctory-NVIDIA 4fcaebd, readmet3xt-lab, on a 4070
/// SUPER). With DLSS on, `victory_hle::modules::fidelityfx` makes the
/// context at these and gives DLSS the sizes asked.
pub const SHIPPING_FSR4_RENDER: [u32; 2] = [1280, 720];
/// See [`SHIPPING_FSR4_RENDER`].
pub const SHIPPING_FSR4_OUTPUT: [u32; 2] = [1920, 1080];

/// The picker's FidelityFX.
pub const FIDELITYFX: u32 = 7;

/// The FidelityFX path the app asks for (DLSS, frame generation or
/// `VICTORY_FSR_PATH=1`; see the module's notes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Upscaler {
    /// What it is for, in the log (`dlss quality`, say).
    pub name: &'static str,
    /// The FidelityFX quality mode the title notes: 0 native AA, 1 quality,
    /// 2 balanced, 3 performance, 4 ultra performance.
    pub ffx_mode: u32,
    /// The render size over the output size.
    pub scale: f32,
}

/// `mov eax, kind ; ret`.
pub fn picker_code(kind: u32) -> Vec<u8> {
    let mut c = vec![0xb8];
    c.extend(kind.to_le_bytes());
    c.push(0xc3);
    c
}

/// Compares `site`'s bytes in the image of `size` bytes at `base`.
///
/// # Safety
/// The image is mapped.
pub unsafe fn matches(base: usize, size: usize, site: &Site) -> Patched {
    if site.at.checked_add(site.bytes.len()).is_none_or(|end| end > size) {
        return Patched::Failed("outside the image");
    }
    // SAFETY: inside the mapped image (checked above).
    let there = unsafe { std::slice::from_raw_parts((base + site.at) as *const u8, site.bytes.len()) };
    if there == site.bytes { Patched::Done } else { Patched::Differs }
}

/// Writes `code` over `site` if its bytes are the expected ones.
///
/// # Safety
/// The image stays mapped and no thread runs its code yet.
#[cfg(windows)]
pub unsafe fn write(base: usize, size: usize, site: &Site, code: &[u8]) -> Patched {
    use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
    use windows_sys::Win32::System::Memory::{PAGE_EXECUTE_READWRITE, VirtualProtect};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    if code.len() > site.bytes.len() {
        return Patched::Failed("replacement longer than the bytes compared");
    }
    // SAFETY: the caller's promise.
    let found = unsafe { matches(base, size, site) };
    if found != Patched::Done {
        return found;
    }
    let at = base + site.at;
    let mut old = 0;
    // SAFETY: inside the image (checked); made writable for this and put
    // back; nothing runs it yet.
    unsafe {
        if VirtualProtect(at as *const _, code.len(), PAGE_EXECUTE_READWRITE, &mut old) == 0 {
            return Patched::Failed("Windows would not make it writable");
        }
        std::ptr::copy_nonoverlapping(code.as_ptr(), at as *mut u8, code.len());
        VirtualProtect(at as *const _, code.len(), old, &mut old);
        FlushInstructionCache(GetCurrentProcess(), at as *const _, code.len());
    }
    Patched::Done
}

/// Applies what the switches ask for, and the FidelityFX path if `upscaler`
/// is given; a line each in the log and on standard output. Whether the
/// FidelityFX path was put in (all three of its places).
///
/// # Safety
/// The image of `size` bytes at `base` stays mapped and no thread runs its
/// code yet.
#[cfg(windows)]
pub unsafe fn apply_asked(base: usize, size: usize, upscaler: Option<Upscaler>) -> bool {
    // The FidelityFX path's three places, all or none.
    let path_sites = [&PICKER, &FFX_SCALE, &OWN_SCALE];
    let upscaler = upscaler.filter(|u| {
        let differ: Vec<&str> = path_sites
            .iter()
            // SAFETY: the caller's promise.
            .filter(|s| unsafe { matches(base, size, s) } != Patched::Done)
            .map(|s| s.what)
            .collect();
        if !differ.is_empty() {
            tracing::warn!(?differ, "resolution: the FidelityFX path is not patched in (another build?)");
            println!(
                "resolution: the FidelityFX path ({}) is not patched in, the bytes are not the ones expected at: {}; the title upscales as before",
                u.name,
                differ.join(", ")
            );
        }
        differ.is_empty()
    });
    let mut asked: Vec<(&Site, Vec<u8>, String)> = Vec::new();
    if let Some((w, h)) = asked_output() {
        let plane_asked = asked_ui_plane((w, h));
        // The routines the output block calls go in first; the block calls
        // one only if they did (else it is as it was before them).
        // SAFETY: the caller's promise.
        let spare = unsafe { write(base, size, &SPARE, &spare_code()) };
        report(&SPARE, spare, if plane_asked { "interface plane on" } else { "interface plane off" });
        let ready = spare == Patched::Done;
        // SAFETY: as above.
        let plane = plane_asked && ready && unsafe { matches(base, size, &POOL) } == Patched::Done;
        if plane_asked && !plane {
            println!("resolution: no interface plane (the bytes at its places are not the ones expected): the interface stays on the scene");
        }
        let routine = ready.then_some(if plane { SPARE.at } else { SPARE.at + UI_OFF_AT });
        let value = if plane {
            format!("{w}x{h}, the interface at {}x{} on a plane of its own", UI_SIZE.0, UI_SIZE.1)
        } else {
            format!("{w}x{h}")
        };
        asked.push((&OUTPUT, output_code(w, h, routine), value));
        if plane {
            let pool = ui_plane_pool(w, h);
            asked.push((&POOL, pool.to_le_bytes().to_vec(), format!("{} MB for both planes", pool >> 20)));
        }
        if (w, h) != (3840, 2160) {
            asked.push((&POOL_CHECK, pool_check_code(), format!("{w}x{h}")));
        }
    }
    if let Some(u) = upscaler {
        // The FidelityFX path sets the picker and both scales.
        for switch in ["VICTORY_RENDER_SCALE", "VICTORY_FFX_SCALE", "VICTORY_UPSCALER"] {
            if std::env::var_os(switch).is_some() {
                tracing::warn!(switch, "resolution: left out for the FidelityFX path");
                println!("resolution: {switch} is left out: the FidelityFX path ({}) sets the upscaler and its scale", u.name);
            }
        }
    } else {
        if let Some(s) = asked_render_scale() {
            asked.push((&OWN_SCALE, own_scale_code(s), format!("{s}")));
        }
        if let Some(s) = asked_ffx_scale() {
            asked.push((&FFX_SCALE, ffx_scale_code(s), format!("{s}")));
        }
        if let Some(kind) = asked_upscaler() {
            asked.push((&PICKER, picker_code(kind), format!("returns {kind}")));
        }
    }
    for (site, code, value) in asked {
        // SAFETY: the caller's promise.
        let done = unsafe { write(base, size, site, &code) };
        report(site, done, &value);
    }
    let Some(u) = upscaler else {
        return false;
    };
    let path = [
        (&PICKER, picker_code(FIDELITYFX), format!("returns {FIDELITYFX}, {}", u.name)),
        (
            &FFX_SCALE,
            ffx_upscaler_code(u.ffx_mode, u.scale),
            format!("{}, mode {} noted, {}", u.scale, u.ffx_mode, u.name),
        ),
        (&OWN_SCALE, own_scale_code(u.scale), format!("{}, {}", u.scale, u.name)),
    ];
    let mut all = true;
    for (site, code, value) in path {
        // SAFETY: the caller's promise.
        let done = unsafe { write(base, size, site, &code) };
        report(site, done, &value);
        all &= done == Patched::Done;
    }
    all
}

/// A line in the log and on standard output for a site written or not.
#[cfg(windows)]
fn report(site: &Site, done: Patched, value: &str) {
    let offset = format!("{:#x}", site.at);
    match done {
        Patched::Done => tracing::info!(site = %offset, what = site.what, value, "resolution: patched"),
        Patched::Differs => tracing::warn!(site = %offset, what = site.what, "resolution: not patched, the title's bytes there are not the ones expected (another build?)"),
        Patched::Failed(why) => tracing::warn!(site = %offset, what = site.what, why, "resolution: not patched"),
    }
    println!("resolution: {} ({value}) at title+{offset}: {done:?}", site.what);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switches_parse() {
        assert_eq!(parse_output("1920x1080"), Some((1920, 1080)));
        assert_eq!(parse_output(" 2560X1440 "), Some((2560, 1440)));
        assert_eq!(parse_output("1600x900"), None);
        assert_eq!(parse_output("1080p"), None);
        assert_eq!(parse_render_scale("0.67"), Some(0.67));
        assert_eq!(parse_render_scale("0.5"), Some(0.5));
        assert_eq!(parse_render_scale("1"), Some(1.0));
        assert_eq!(parse_render_scale("0.4"), None);
        assert_eq!(parse_render_scale("1.5"), None);
    }

    /// The calls and loads in the bytes compared are the ones the
    /// replacement keeps.
    #[test]
    fn the_output_block_is_the_titles() {
        let b = OUTPUT.bytes;
        assert_eq!(b.len(), 61);
        let target = |op_end: usize| (OUTPUT.at + op_end) as i64 + i64::from(i32::from_le_bytes(b[op_end - 4..op_end].try_into().unwrap()));
        assert_eq!(target(27), SET_SCREEN_SIZE as i64);
        assert_eq!(target(50), SET_SCREEN_SIZE as i64);
        assert_eq!(target(61), HEIGHT_CHANGED as i64);
        assert_eq!(target(41), SCREEN_HEIGHT as i64);
        assert_eq!(target(6), (SCREEN_HEIGHT - 4) as i64, "the width before it");
        // jne to the else path at 29; jmp +0x20 and je +7 both land at the end.
        assert_eq!((b[14], 16 + b[15] as usize), (0x75, 29));
        assert_eq!((b[27], 29 + b[28] as usize), (0xeb, 61));
        assert_eq!((b[52], 54 + b[53] as usize), (0x74, 61));
    }

    #[test]
    fn the_output_code_keeps_the_titles_else_path() {
        // Without a routine, and after a call to either of the two.
        for (routine, k) in [(None, 0), (Some(SPARE.at), 5), (Some(SPARE.at + UI_OFF_AT), 5)] {
            let c = output_code(1920, 1080, routine);
            assert_eq!(c.len(), OUTPUT.bytes.len());
            let target = |op_end: usize| (OUTPUT.at + op_end) as i64 + i64::from(i32::from_le_bytes(c[op_end - 4..op_end].try_into().unwrap()));
            if let Some(routine) = routine {
                assert_eq!(c[0], 0xe8);
                assert_eq!(target(5), routine as i64, "the call reaches the routine");
            }
            assert_eq!(c[k], 0xb8);
            assert_eq!(c[k + 1..k + 5], 1920u32.to_le_bytes());
            assert_eq!(c[k + 5..k + 11], [0x89, 0x86, 0x04, 0x02, 0x00, 0x00]);
            assert_eq!(c[k + 12..k + 16], 1080u32.to_le_bytes());
            assert_eq!(c[k + 16..k + 22], [0x89, 0xbe, 0x08, 0x02, 0x00, 0x00]);
            assert_eq!(c[k + 22..k + 24], [0x8b, 0x2d]);
            assert_eq!(target(k + 28), SCREEN_HEIGHT as i64);
            assert_eq!(c[k + 32], 0xe8);
            assert_eq!(target(k + 37), SET_SCREEN_SIZE as i64);
            assert_eq!(c[k + 37..k + 40], [0x39, 0xfd, 0x74]);
            assert_eq!(k + 41 + c[k + 40] as usize, 61, "je lands at the end");
            assert_eq!(c[k + 43], 0xe8);
            assert_eq!(target(k + 48), HEIGHT_CHANGED as i64);
            assert_eq!(c[k + 48], 0xeb);
            assert_eq!(k + 50 + c[k + 49] as usize, 61, "jmp lands at the end");
            assert!(c[k + 50..].iter().all(|&x| x == 0xcc));
        }
    }

    /// The two routines: overlay on with the interface's size, overlay off;
    /// each ends in `ret` and stays inside the padding.
    #[test]
    fn the_spare_routines_set_the_interface_plane() {
        let c = spare_code();
        assert_eq!(c.len(), SPARE.bytes.len());
        assert!(SPARE.at + c.len() <= 0x55ff2c0, "inside the padding");
        assert_eq!(c[..7], [0xc6, 0x86, 0xe8, 0x01, 0x00, 0x00, 0x01], "mov byte [rsi+uiOverlay], 1");
        assert_eq!(c[7..13], [0xc7, 0x86, 0xec, 0x01, 0x00, 0x00]);
        assert_eq!(c[13..17], 3840u32.to_le_bytes());
        assert_eq!(c[17..23], [0xc7, 0x86, 0xf0, 0x01, 0x00, 0x00]);
        assert_eq!(c[23..27], 2160u32.to_le_bytes());
        assert_eq!(c[27], 0xc3);
        assert!(c[28..UI_OFF_AT].iter().all(|&x| x == 0xcc));
        assert_eq!(c[UI_OFF_AT..UI_OFF_AT + 8], [0xc6, 0x86, 0xe8, 0x01, 0x00, 0x00, 0x00, 0xc3], "mov byte [rsi+uiOverlay], 0 ; ret");
        assert!(c[UI_OFF_AT + 8..].iter().all(|&x| x == 0xcc));
    }

    #[test]
    fn the_pool_holds_both_planes() {
        assert_eq!(POOL.bytes, (64u32 << 20).to_le_bytes());
        assert_eq!(ui_plane_pool(1280, 720) >> 20, 82, "Series S's at least");
        assert_eq!(ui_plane_pool(1920, 1080) >> 20, 88);
        assert_eq!(ui_plane_pool(2560, 1440) >> 20, 102);
        for (w, h) in MODES {
            assert!(ui_plane_pool(w, h) >= 82 << 20);
        }
    }

    #[test]
    fn the_interface_plane_only_below_its_size_and_with_the_compositor() {
        assert!(ui_plane_applies((1920, 1080), true, false));
        assert!(ui_plane_applies((2560, 1440), true, false));
        assert!(!ui_plane_applies((1920, 1080), false, false), "off unless asked");
        assert!(!ui_plane_applies((1920, 1080), true, true), "VICTORY_PLANES=0");
        assert!(!ui_plane_applies(UI_SIZE, true, false), "4K needs none");
    }

    #[test]
    fn the_scale_codes() {
        let own = own_scale_code(0.5);
        assert_eq!(own[..6], OWN_SCALE.bytes[..6]);
        assert_eq!(own[6..], 0.5f32.to_le_bytes());
        assert_eq!(OWN_SCALE.bytes[6..], (2.0f32 / 3.0).to_le_bytes());
        let ffx = ffx_scale_code(0.75);
        assert_eq!(ffx.len(), FFX_SCALE.bytes.len());
        assert_eq!(ffx[0x64..0x68], 0.75f32.to_le_bytes());
        for at in [0x28, 0x32] {
            assert_eq!(ffx[at..at + 4], [0xc5, 0xfa, 0x10, 0x05], "vmovss xmm0, [rip+d]");
            let disp = i32::from_le_bytes(ffx[at + 4..at + 8].try_into().unwrap());
            assert_eq!(at as i32 + 8 + disp, 0x64);
        }
        // Only the two displacements and the padding change.
        let changed: Vec<usize> = (0..ffx.len()).filter(|&i| ffx[i] != FFX_SCALE.bytes[i]).collect();
        assert!(changed.iter().all(|&i| (0x2c..0x30).contains(&i) || (0x36..0x3a).contains(&i) || (0x64..0x68).contains(&i)));
        assert_eq!(picker_code(7), [0xb8, 7, 0, 0, 0, 0xc3]);
        assert_eq!(pool_check_code(), [0x48, 0x3d, 0xff, 0xff, 0x1f, 0x00, 0xeb, 0x01]);
    }

    /// The FidelityFX path: the mode noted and a jump to the scaled load in
    /// place of the console check's branch, which then reads the scale.
    #[test]
    fn the_fidelityfx_path_code() {
        let b = FFX_SCALE.bytes;
        // movzx eax, al ; mov [r14+90h], eax ; test al, al ; jne scaled.
        assert_eq!(
            b[FFX_MODE_AT..FFX_MODE_AT + 14],
            [0x0f, 0xb6, 0xc0, 0x41, 0x89, 0x86, 0x90, 0x00, 0x00, 0x00, 0x84, 0xc0, 0x75, 0x0a]
        );
        assert_eq!(FFX_MODE_AT + 14 + b[FFX_MODE_AT + 13] as usize, FFX_SCALED_LOAD);
        assert_eq!(b[FFX_SCALED_LOAD..FFX_SCALED_LOAD + 4], [0xc5, 0xfa, 0x10, 0x05]);
        for (mode, scale) in [(0, 1.0f32), (1, 2.0 / 3.0), (3, 0.5), (4, 1.0 / 3.0)] {
            let c = ffx_upscaler_code(mode, scale);
            assert_eq!(c.len(), b.len());
            assert_eq!(c[FFX_MODE_AT], 0xb8, "mov eax, mode");
            assert_eq!(c[FFX_MODE_AT + 1..FFX_MODE_AT + 5], mode.to_le_bytes());
            assert_eq!(c[FFX_MODE_AT + 5..FFX_MODE_AT + 12], b[FFX_MODE_AT + 3..FFX_MODE_AT + 10]);
            assert_eq!(c[FFX_MODE_AT + 12], 0xeb);
            assert_eq!(FFX_MODE_AT + 14 + c[FFX_MODE_AT + 13] as usize, FFX_SCALED_LOAD);
            let disp = i32::from_le_bytes(c[FFX_SCALED_LOAD + 4..FFX_SCALED_LOAD + 8].try_into().unwrap());
            let slot = (FFX_SCALED_LOAD as i32 + 8 + disp) as usize;
            assert_eq!(c[slot..slot + 4], scale.to_le_bytes(), "the scaled load reads the scale");
            // Only the branch, the two displacements and the padding change.
            let changed: Vec<usize> = (0..c.len()).filter(|&i| c[i] != b[i]).collect();
            assert!(changed.iter().all(|&i| (FFX_MODE_AT..FFX_MODE_AT + 14).contains(&i)
                || (0x2c..0x30).contains(&i)
                || (0x36..0x3a).contains(&i)
                || (0x64..0x68).contains(&i)));
        }
        assert_eq!(picker_code(FIDELITYFX), [0xb8, 7, 0, 0, 0, 0xc3]);
    }

    #[cfg(windows)]
    #[test]
    fn a_site_is_written_only_over_its_bytes() {
        use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READ, VirtualAlloc, VirtualFree};
        // SAFETY: fresh memory of our own, read-only to code as the image's is.
        unsafe {
            let place = VirtualAlloc(std::ptr::null(), 0x1000, MEM_COMMIT | MEM_RESERVE, 0x40) as *mut u8;
            assert!(!place.is_null());
            let site = Site { at: 0x100, what: "test", bytes: OWN_SCALE.bytes };
            std::ptr::copy_nonoverlapping(site.bytes.as_ptr(), place.add(0x100), site.bytes.len());
            let mut old = 0;
            windows_sys::Win32::System::Memory::VirtualProtect(place.cast(), 0x1000, PAGE_EXECUTE_READ, &mut old);
            assert_eq!(write(place as usize, 0x1000, &site, &own_scale_code(0.5)), Patched::Done);
            assert_eq!(std::slice::from_raw_parts(place.add(0x106), 4), 0.5f32.to_le_bytes());
            // Already changed: its bytes differ now, nothing is written again.
            assert_eq!(write(place as usize, 0x1000, &site, &own_scale_code(0.75)), Patched::Differs);
            assert_eq!(std::slice::from_raw_parts(place.add(0x106), 4), 0.5f32.to_le_bytes());
            let far = Site { at: 0xffa, what: "test", bytes: OWN_SCALE.bytes };
            assert!(matches!(write(place as usize, 0x1000, &far, &own_scale_code(0.5)), Patched::Failed(_)));
            VirtualFree(place.cast(), 0, MEM_RELEASE);
        }
    }
}
