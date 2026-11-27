//! Mesh draws launched by the card: an amplification shader of ours
//! (`launch.hlsl`) in front of every translated mesh shader
//! (`VICTORY_MESH_LAUNCH=1`; off by default: the clamp of `crate::indirect`
//! before each record, alone or batched, as before). From fxceless b1d777a8
//! and 0e006f00, after Kelvaroq patch 0041.
//!
//! The title draws its meshes one indirect record a call, each from the
//! next slot of an argument buffer its own compute passes wrote: thousands
//! of records a frame in heavy scenes. Each is bounded by a compute
//! dispatch of ours between two waits for every earlier shader write
//! (`crate::indirect::bound`), or by a batched clamp
//! (`VICTORY_INDIRECT_BATCH`), so the card goes from drawing to compute and
//! back. In the other forks' runs that was most of the frame's full waits;
//! Kelvaroq measured a heavy scene's frame at 198 ms with the clamp and
//! 120 ms with this launch (RTX 5080), fxceless the wait for the card
//! 21 ms -> 2 ms with its masked patch.
//!
//! Here the record is read by the draw itself: the pipeline's amplification
//! stage loads its three words where the title's passes wrote them, bounds
//! them as the clamp does (65,535 groups, one-dimensional; a slot of more
//! is empty), charges the list's budget as the clamp does (a record that
//! does not fit runs empty), and launches the mesh groups. The title's own
//! barrier (a host UAV barrier, `crate::list`) orders its passes before the
//! draw, as on the console; nothing of ours runs between draws. A mesh
//! dispatch the CPU knows (`DispatchMesh`) gives its count as a root
//! constant and is launched the same way, unbounded by the budget as before.
//!
//! The draws' amplification groups may run in another order than the
//! list's (the card overlaps draws; nothing orders them now), so when the
//! list's budget runs out, which of its records run empty is the card's
//! order and no longer the list's: the bound on the list's total work is
//! the same.
//!
//! An indirect call with a count buffer or more than one record keeps the
//! clamp and runs on the same pipeline made without the amplification
//! stage (`HostPipeline::plain`, made the first time one needs it): the
//! translated mesh shader declares the payload either way
//! (`victory_shader::emit::mesh_payload_bytes`, under the same switch),
//! which the driver takes with or without an amplification shader.
//!
//! The stage is compiled from `launch.hlsl` when the first mesh pipeline is
//! made, by the PC's `dxc.exe` (`VICTORY_PC_DXC`, else beside the `dxil.dll`
//! of `VICTORY_DXIL` or the newest Windows SDK), and validated and signed by
//! that `dxil.dll`. Without them mesh pipelines are made without the stage
//! and every record is clamped, as with the switch off.
//!
//! A/B (diagnostics builds): `VICTORY_MESH_LAUNCH=1` together with
//! `VICTORY_AB=<seconds>,VICTORY_MESH_LAUNCH`: the pipelines have the stage
//! throughout, and in the phases without it their draws run on the plain
//! pipelines behind the clamp.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use windows::Win32::Graphics::Direct3D12::*;
use windows::core::Interface;

use crate::host::HostList;

/// The amplification shader's source.
const SOURCE: &str = include_str!("launch.hlsl");

/// The root constant that has the card read the record.
const DIRECT_NONE: u32 = u32::MAX;

/// Where the amplification shader binds: `b0`, `t0`, `u0` in this space.
const SPACE: u32 = victory_shader::emit::LAUNCH_SPACE;

/// Whether mesh pipelines get the amplification stage and indirect mesh
/// records are launched by it (`VICTORY_MESH_LAUNCH=1`; unset or `0`: not,
/// as before). The translated mesh shaders declare the payload the stage
/// needs under the same switch (`victory_shader::emit::mesh_payload_bytes`).
pub(crate) fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        crate::switch::on("VICTORY_MESH_LAUNCH") && victory_shader::emit::mesh_payload_bytes() > 0
    })
}

/// Whether a draw of a pipeline with the stage is launched by it now:
/// always, unless `VICTORY_AB` names `VICTORY_MESH_LAUNCH` (diagnostics
/// builds), whose phases without run the plain pipelines behind the clamp.
pub(crate) fn now() -> bool {
    #[cfg(feature = "diagnostics")]
    if let Some(on) = crate::ab::pick("VICTORY_MESH_LAUNCH") {
        return on;
    }
    true
}

/// The amplification stage's code, compiled once ([`compile`]); `None` if
/// it cannot be.
pub(crate) fn code() -> Option<D3D12_SHADER_BYTECODE> {
    static CODE: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    let code = CODE
        .get_or_init(|| match compile() {
            Ok(code) => {
                tracing::info!(
                    bytes = code.len(),
                    "mesh launch stage compiled (VICTORY_MESH_LAUNCH)"
                );
                Some(code)
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "no mesh launch stage (VICTORY_MESH_LAUNCH): mesh records are clamped as before"
                );
                None
            }
        })
        .as_deref()?;
    Some(D3D12_SHADER_BYTECODE {
        pShaderBytecode: code.as_ptr().cast(),
        BytecodeLength: code.len(),
    })
}

/// [`SOURCE`] compiled by the PC's `dxc.exe` (as the pipeline tests find
/// it: `VICTORY_PC_DXC`, else beside the validator's `dxil.dll`), then
/// validated and signed by that `dxil.dll` where it loads.
fn compile() -> Result<Vec<u8>, String> {
    use std::os::windows::process::CommandExt;
    use victory_shader::emit::validator::Validator;
    /// `CREATE_NO_WINDOW`: no console flashes up for the compiler.
    const NO_WINDOW: u32 = 0x0800_0000;
    let dxc = match std::env::var_os("VICTORY_PC_DXC") {
        Some(p) => std::path::PathBuf::from(p),
        None => Validator::default_path()
            .ok_or("no dxil.dll (VICTORY_DXIL or a Windows SDK) to find dxc.exe beside")?
            .with_file_name("dxc.exe"),
    };
    if !dxc.exists() {
        return Err(format!("no {}", dxc.display()));
    }
    let dir = std::env::temp_dir().join(format!("victory-launch-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let (hlsl, out) = (dir.join("launch.hlsl"), dir.join("launch.dxil"));
    std::fs::write(&hlsl, SOURCE).map_err(|e| e.to_string())?;
    let rest = format!("REST={}", crate::indirect::rest());
    let ran = std::process::Command::new(&dxc)
        .args(["-nologo", "-T", "as_6_5", "-E", "main", "-D"])
        .arg(&rest)
        .arg("-Fo")
        .arg(&out)
        .arg(&hlsl)
        .creation_flags(NO_WINDOW)
        .output();
    let code = match ran {
        Ok(o) if o.status.success() => std::fs::read(&out).map_err(|e| e.to_string()),
        Ok(o) => {
            let said = String::from_utf8_lossy(&o.stderr);
            Err(format!("{} failed: {said}", dxc.display()))
        }
        Err(e) => Err(format!("{}: {e}", dxc.display())),
    };
    let _ = std::fs::remove_dir_all(&dir);
    let mut code = code?;
    if let Ok(mut validator) = Validator::load(Validator::default_path().as_deref()) {
        validator.validate(&mut code)?;
    }
    Ok(code)
}

/// The three root parameters the stage reads, in order: the direct group
/// count (one constant), the record (a root SRV), the list's budget (a
/// root UAV). Five root words.
pub(crate) fn params() -> [D3D12_ROOT_PARAMETER1; 3] {
    let descriptor = |kind| D3D12_ROOT_PARAMETER1 {
        ParameterType: kind,
        Anonymous: D3D12_ROOT_PARAMETER1_0 {
            Descriptor: D3D12_ROOT_DESCRIPTOR1 {
                ShaderRegister: 0,
                RegisterSpace: SPACE,
                Flags: D3D12_ROOT_DESCRIPTOR_FLAG_DATA_VOLATILE,
            },
        },
        ShaderVisibility: D3D12_SHADER_VISIBILITY_AMPLIFICATION,
    };
    [
        D3D12_ROOT_PARAMETER1 {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Anonymous: D3D12_ROOT_PARAMETER1_0 {
                Constants: D3D12_ROOT_CONSTANTS {
                    ShaderRegister: 0,
                    RegisterSpace: SPACE,
                    Num32BitValues: 1,
                },
            },
            ShaderVisibility: D3D12_SHADER_VISIBILITY_AMPLIFICATION,
        },
        descriptor(D3D12_ROOT_PARAMETER_TYPE_SRV),
        descriptor(D3D12_ROOT_PARAMETER_TYPE_UAV),
    ]
}

/// What one launch reads.
#[derive(Clone, Copy)]
pub(crate) enum Launch {
    /// The record at this address on the card.
    Record(u64),
    /// A group count known on the CPU.
    Direct(u32),
}

/// Mesh draws launched since the last line: records read by the card, and
/// dispatches whose count the CPU gave.
static LAUNCHED: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];

/// Records the launch of one mesh draw, the mesh pipeline with its stage
/// bound and the stage's parameters at `roots`: the record's address on the
/// card (the card reads it), or the group count the CPU knows. False if the
/// list has no budget for a record.
pub(crate) fn record(list: &HostList, roots: [u32; 3], what: Launch) -> bool {
    let Ok(l6) = list.list.cast::<ID3D12GraphicsCommandList6>() else {
        return false;
    };
    let budget = list.budget();
    let (direct, record, budget) = match what {
        Launch::Record(address) => {
            let Some(budget) = budget else {
                return false;
            };
            (DIRECT_NONE, address, budget)
        }
        // The stage reads neither the record nor the budget; live
        // addresses all the same where the list has them.
        Launch::Direct(x) => (x, budget.unwrap_or(0), budget.unwrap_or(0)),
    };
    // SAFETY: an open list with the mesh pipeline and its root signature
    // set; the record and the budget are live buffers the list keeps.
    unsafe {
        list.list.SetGraphicsRoot32BitConstant(roots[0], direct, 0);
        list.list.SetGraphicsRootShaderResourceView(roots[1], record);
        list.list.SetGraphicsRootUnorderedAccessView(roots[2], budget);
        l6.DispatchMesh(1, 1, 1);
    }
    LAUNCHED[usize::from(matches!(what, Launch::Direct(_)))].fetch_add(1, Ordering::Relaxed);
    each_second();
    true
}

/// Says the counts once a second, from the launches themselves, per frame
/// begun meanwhile.
fn each_second() {
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    static SECOND: AtomicU64 = AtomicU64::new(0);
    static FRAME: AtomicU64 = AtomicU64::new(0);
    let now = START.get_or_init(std::time::Instant::now).elapsed().as_secs();
    let last = SECOND.load(Ordering::Relaxed);
    if now > last
        && SECOND
            .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    {
        let frame = crate::frame::current();
        let frames = frame.saturating_sub(FRAME.swap(frame, Ordering::Relaxed)).max(1) as f64;
        let [records, direct] = [0, 1].map(|i| LAUNCHED[i].swap(0, Ordering::Relaxed));
        tracing::info!(
            "mesh draws launched by the card per frame: {:.0} records read where the title wrote \
             them, {:.0} counts known on the CPU (VICTORY_MESH_LAUNCH)",
            records as f64 / frames,
            direct as f64 / frames
        );
    }
}
