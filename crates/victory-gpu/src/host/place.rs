//! The title's buffer copies, done on the card at their place in the list
//! (`VICTORY_COPIES_IN_PLACE`, on by default; `0`: all at submission, on the
//! CPU, as before).
//!
//! A buffer copy was done in title memory when its list was submitted, before
//! any draw of the submission ran, and the host copies refreshed after it.
//! That is another order than the list's: draws ahead of the copy read what
//! it put; of two copies to the same bytes with a draw behind each, both
//! draws read the second; a source that a dispatch of the same submission
//! writes first was read as it was before it; and a destination the
//! submissions in the air still read changed under them (where its host copy
//! has one side only, `crate::shadow::card_all`).
//!
//! Now the list copies, in the destination's host copy: out of the source's
//! host copy where shaders write that one, else out of upload memory of the
//! list that takes the source's bytes when the list is submitted (a source
//! shaders never wrote is the title's CPU's, or a copy's put there earlier in
//! the submission). What is written there is a shader's write from then on:
//! the bytes are declared, and what the card made of them is put into title
//! memory before the title is told of the submission.
//!
//! Title memory takes the copy at submission as well, as it did: fills of
//! upload memory, textures and host copies that lie over the same bytes read
//! it there. The reference of the destination's host copy takes the same
//! bytes ([`Shadow::placed_copy`]), so what the landing finds is what the card
//! made of them afterwards (a counter copied to zero and counted up; a source
//! that a shader wrote in between), and nothing is sent up over it. Where
//! flights in the air may write the source or the destination this is left to
//! the landing: what they wrote is not in title memory yet, and what they
//! will put there must not stand over this copy's bytes. If a step of the
//! submission reads the destination on the CPU, it waits for them instead
//! ([`Placed::waits`]).
//!
//! Copies to memory that has no host copy yet are done as before: nothing on
//! the card has read or written it. So are those of more than 8 MiB out of
//! memory shaders do not write (streaming) into a host copy with two sides,
//! whose card side takes them in the submission's order anyway; into one with
//! one side they are placed too.

use super::*;
use crate::shadow::Shadow;

pub(super) fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_COPIES_IN_PLACE").is_none_or(|v| v != "0"))
}

/// The most bytes copied out of memory shaders do not write.
const MOST: usize = 8 << 20;

/// Since the last frame-rate line: copies at their place, their bytes, those
/// out of a host copy, and texture copies out of one.
pub(super) static STATS: [std::sync::atomic::AtomicU64; 4] = [const { std::sync::atomic::AtomicU64::new(0) }; 4];

/// Since the last frame-rate line: constant buffers filled on the card at
/// their place behind a copy at its place ([`HostList::placed_constants`]).
pub(super) static STATS_CONSTANTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Whether the card buffers of copies at their place
/// ([`HostList::between`]) are used again by later lists (on by default;
/// `VICTORY_BETWEEN_POOL=0`: a new buffer for each, let go when its list is
/// reset, as before). The title's constant buffers behind such copies
/// ([`HostList::placed_constants`]) took one of 64 KiB each: 6.8 a frame in
/// the run of 2026-11-27 on the RTX 5070, all 24,300 card buffers made in
/// its five minutes of play, each about a millisecond of the translating
/// thread's time (the driver's creation, then the waits of
/// `VICTORY_WAIT_RESIDENT`).
fn between_pool() -> bool {
    crate::toggled!("VICTORY_BETWEEN_POOL")
}

/// The bytes of buffers [`BETWEEN_POOL`] keeps, and how many of the oldest
/// a taking looks at.
const BETWEEN_POOL_MOST: u64 = 64 << 20;
const BETWEEN_POOL_LOOKS: usize = 32;

/// Buffers of [`HostList::between`] whose list was reset, the oldest first,
/// each with its size; and the bytes of them all. One that a descriptor
/// slot still shows (`VICTORY_SLOT_HOLDS`) waits here until the slot shows
/// something else: only a buffer nothing but the pool holds is taken.
#[allow(clippy::type_complexity)]
static BETWEEN_POOL: Mutex<(u64, std::collections::VecDeque<(ID3D12Resource, u64)>)> =
    Mutex::new((0, std::collections::VecDeque::new()));

/// Since the start: buffers taken from [`BETWEEN_POOL`], and made for it.
static BETWEEN_POOL_STATS: [std::sync::atomic::AtomicU64; 2] = [const { std::sync::atomic::AtomicU64::new(0) }; 2];

/// A buffer of `size` bytes that no list and no slot holds any more.
fn between_take(size: u64) -> Option<ID3D12Resource> {
    let mut pool = BETWEEN_POOL.lock().unwrap_or_else(|e| e.into_inner());
    let at = pool.1.iter().take(BETWEEN_POOL_LOOKS).position(|(b, s)| *s == size && references(b) == 1)?;
    let (buffer, _) = pool.1.remove(at)?;
    pool.0 -= size;
    Some(buffer)
}

/// A list was reset: its buffers of [`HostList::between`] are kept for the
/// next lists, while there is room. Its last submission has run by then
/// (`Deferred`'s drop), and a buffer is in the common state again once the
/// submission that used it has run, as a new one is.
pub(super) fn between_give(buffers: impl Iterator<Item = (ID3D12Resource, u64)>) {
    let mut freed = Vec::new();
    {
        let mut pool = BETWEEN_POOL.lock().unwrap_or_else(|e| e.into_inner());
        for (buffer, size) in buffers {
            if !between_pool() || pool.0 + size > BETWEEN_POOL_MOST {
                freed.push(buffer);
                continue;
            }
            pool.0 += size;
            pool.1.push_back((buffer, size));
        }
    }
    // Let go outside the lock.
    drop(freed);
}

/// With the frame line: what [`between_pool`] did since the start. Nothing
/// while it has done nothing.
pub(super) fn between_line() {
    let [taken, made] = [0, 1].map(|i| BETWEEN_POOL_STATS[i].load(Ordering::Relaxed));
    if taken + made == 0 {
        return;
    }
    let (kept, mb) = {
        let pool = BETWEEN_POOL.lock().unwrap_or_else(|e| e.into_inner());
        (pool.1.len(), pool.0 >> 20)
    };
    tracing::info!(
        "card buffers of copies at their place used again (VICTORY_BETWEEN_POOL=0: a new one each time): {taken} taken from the pool, {made} made for it, {kept} in it now ({mb} MB)"
    );
}

/// What the CPU does at submission for a copy at its place in a list.
pub(super) struct Placed {
    pub(super) dst: usize,
    pub(super) src: usize,
    pub(super) len: usize,
    /// The destination's host copy.
    pub(super) shadow: std::sync::Arc<Shadow>,
    /// Upload memory the list copies from, to be given the source's bytes.
    from: Option<usize>,
}

impl Placed {
    /// Whether flights that write `written` are waited for.
    pub(super) fn waits(&self, written: &[(usize, usize)]) -> Waits {
        let source = touches(written, self.src, self.src + self.len);
        if source && self.from.is_some() {
            #[cfg(feature = "diagnostics")]
            {
                static SEEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                if SEEN.fetch_add(1, Ordering::Relaxed) < 20 {
                    let (start, n) = self.shadow.span();
                    tracing::info!(
                        dst = format_args!("{:#x}", self.dst),
                        src = format_args!("{:#x}", self.src),
                        len = self.len,
                        dst_copy = format_args!("{start:#x}+{n:#x}"),
                        src_title_buffer = ?crate::shadow::peek(self.src as u64, self.len as u64).map(|s| (s.span(), s.written(), s.in_place())),
                        src_exact = ?crate::shadow::peek_exact(self.src as u64, self.len as u64).map(|s| s.span()),
                        "a copy at its place takes its source out of upload memory, and flights write that source: it waits for them"
                    );
                }
            }
            return Waits::Yes;
        }
        match source || touches(written, self.dst, self.dst + self.len) {
            true => Waits::ForReaders,
            false => Waits::No,
        }
    }

    /// At submission, in the order of the steps: the list's upload memory
    /// takes the source's bytes and, if `free` (no flight in the air may write
    /// source or destination), title memory at the destination does. Gives
    /// the bytes if it took them.
    pub(super) fn prepare(&self, free: bool) -> Option<Vec<u8>> {
        if !free && self.from.is_none() {
            return None;
        }
        let mut bytes = vec![0u8; self.len];
        // (What is not there reads as zeros, as any copy's source.)
        copy_mapped(bytes.as_mut_ptr(), self.src as *const u8, self.len, false);
        if let Some(to) = self.from {
            // SAFETY: the list's own upload memory of that many bytes.
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), to as *mut u8, self.len) };
        }
        if !free {
            return None;
        }
        let whole = copy_mapped(self.dst as *mut u8, bytes.as_ptr(), self.len, true);
        crate::watch::wrote(self.dst, self.len);
        if whole {
            return Some(bytes);
        }
        // Title memory that is not all there was left alone, or written piece
        // by piece: what it holds now is read back. (Only then: the read and
        // compare of up to 8 MiB was done after every such copy.)
        let mut there = vec![0u8; self.len];
        (read_into(self.dst as *const u8, &mut there) && there == bytes).then_some(bytes)
    }
}

/// Whether a copy waits for flights before its submission is prepared: not;
/// yes, as it takes a source on the CPU that they may write; or only if
/// another step of the submission reads its destination on the CPU.
pub(super) enum Waits {
    No,
    ForReaders,
    Yes,
}

/// Whether a step of `runs` reads on the CPU what one of their copies at its
/// place, ahead of it, waits for its readers ([`Waits::ForReaders`]) with. (A
/// reader ahead of the copy wants what was there before it, and the steps are
/// done in this order: `VICTORY_PLACED_READERS=0` counts those as well, as
/// before.)
pub(super) fn read_by(runs: &[&HostList], written: &[(usize, usize)]) -> bool {
    static ALL: OnceLock<bool> = OnceLock::new();
    let all = *ALL.get_or_init(|| std::env::var_os("VICTORY_PLACED_READERS").is_some_and(|v| v == "0"));
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let waits = |p: &Placed| matches!(p.waits(written), Waits::ForReaders);
    if all {
        for list in runs {
            let steps = &list.deferred().copies;
            let placed = steps.iter().filter_map(|s| if let Step::Placed(p) = s { Some(p) } else { None });
            let placed: Vec<_> = placed.filter(|p| waits(p)).map(|p| (p.dst, p.dst + p.len)).collect();
            join_ranges(&mut ranges, placed);
        }
    }
    runs.iter().any(|list| {
        list.deferred().copies.iter().any(|step| {
            // What reads, by kind ([`READERS`]).
            let (read, kind) = match step {
                Step::Placed(p) => {
                    let read = p.from.is_some() && touches(&ranges, p.src, p.src + p.len);
                    if !all && waits(p) {
                        join_ranges(&mut ranges, [(p.dst, p.dst + p.len)]);
                    }
                    (read, 0)
                }
                _ if ranges.is_empty() => (false, 0),
                &Step::Copy(_, src, len) => (touches(&ranges, src, src + len), 1),
                &Step::Fill(_, src, len, _) => (touches(&ranges, src, src + len), 2),
                Step::Texture(staging) => {
                    let (at, len) = staging.memory();
                    (touches(&ranges, at, at + len), 3)
                }
                Step::Host(_, (from, to)) => (touches(&ranges, *from, *to), 4),
                _ => (false, 0),
            };
            #[cfg(feature = "diagnostics")]
            if read {
                READERS[kind].fetch_add(1, Ordering::Relaxed);
            }
            let _ = kind;
            read
        })
    })
}

/// Diagnostics: the steps found reading what a copy at its place writes
/// ([`read_by`]) since the last frame line: another such copy, a buffer copy,
/// a fill, a texture, a step of ours.
#[cfg(feature = "diagnostics")]
pub(super) static READERS: [std::sync::atomic::AtomicU64; 5] = [const { std::sync::atomic::AtomicU64::new(0) }; 5];

/// Everything recorded so far is done, and seen, before what follows.
///
/// # Safety
/// An open list.
unsafe fn wait_for_all(host: &Host, list: &ID3D12GraphicsCommandList) {
    static ENHANCED: OnceLock<bool> = OnceLock::new();
    let enhanced = *ENHANCED.get_or_init(|| {
        let mut o = D3D12_FEATURE_DATA_D3D12_OPTIONS12::default();
        // SAFETY: a feature structure of its size.
        let known = unsafe {
            host.device.CheckFeatureSupport(
                D3D12_FEATURE_D3D12_OPTIONS12,
                (&raw mut o).cast(),
                size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS12>() as u32,
            )
        };
        known.is_ok() && o.EnhancedBarriersSupported.as_bool()
    });
    // SAFETY: the caller's.
    unsafe {
        if enhanced && let Ok(list) = list.cast::<ID3D12GraphicsCommandList7>() {
            let all = [D3D12_GLOBAL_BARRIER {
                SyncBefore: D3D12_BARRIER_SYNC_ALL,
                SyncAfter: D3D12_BARRIER_SYNC_ALL,
                AccessBefore: D3D12_BARRIER_ACCESS_COMMON,
                AccessAfter: D3D12_BARRIER_ACCESS_COMMON,
            }];
            list.Barrier(&[D3D12_BARRIER_GROUP {
                Type: D3D12_BARRIER_TYPE_GLOBAL,
                NumBarriers: 1,
                Anonymous: D3D12_BARRIER_GROUP_0 {
                    pGlobalBarriers: all.as_ptr(),
                },
            }]);
            return;
        }
        let alias = D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_ALIASING,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Aliasing: std::mem::ManuallyDrop::new(D3D12_RESOURCE_ALIASING_BARRIER {
                    pResourceBefore: std::mem::ManuallyDrop::new(None),
                    pResourceAfter: std::mem::ManuallyDrop::new(None),
                }),
            },
        };
        let uav = D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                UAV: std::mem::ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER {
                    pResource: std::mem::ManuallyDrop::new(None),
                }),
            },
        };
        list.ResourceBarrier(&[alias, uav]);
    }
}

/// The host copy of `[src, src + len)` if shaders write it: what it holds at
/// a place in a list is not what title memory holds at submission.
fn shaders_own(src: usize, len: usize) -> Option<std::sync::Arc<Shadow>> {
    crate::shadow::peek(src as u64, len as u64)
        .filter(|s| s.written() && !s.in_place())
        .or_else(|| placed_exact().then(|| crate::shadow::peek_exact(src as u64, len as u64)).flatten())
}

/// Whether a copy's source that shaders write through a copy of a single
/// view's own bytes (`crate::shadow::exact`) is copied out of that copy on
/// the card (`VICTORY_PLACED_EXACT=0`: out of upload memory the CPU fills at
/// submission, as before). The title copies 240 bytes a frame out of such a
/// view that a dispatch in the air writes: from upload memory the submission
/// waited for every flight to land first, 36 ms a frame on the RTX 5070 at
/// 1080p (the card-steps line: "a copy at its place waits for flights").
fn placed_exact() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_PLACED_EXACT").is_none_or(|v| v != "0"))
}

impl HostList {
    /// Copies `len` bytes of title memory at `src` to `dst`, at this place of
    /// the list (see the file). `false` if it is to be done at submission.
    pub(crate) fn place_copy(&self, dst: usize, src: usize, len: usize) -> bool {
        if !on() {
            return false;
        }
        let Some(host) = host() else {
            return false;
        };
        let Some(to) = crate::shadow::peek(dst as u64, len as u64).filter(|s| !s.in_place()) else {
            return false;
        };
        let own = shaders_own(src, len);
        let scratch = match &own {
            Some(_) => None,
            // Above `MOST` too where the destination's host copy has one side
            // (in the PC's memory, what the card reads): done at submission,
            // the copy would change it under the submissions in the air that
            // still read it. On cards with a budget of 12 GiB or less copies
            // under 16 MiB have one side (`shadow::card_limits`), so streaming
            // copies of 8 to 16 MiB came here.
            None if len <= MOST || !to.carded() => match self.scratch_at(len as u64) {
                Some(scratch) => Some(scratch),
                None => return false,
            },
            None => return false,
        };
        let at = to.offset(dst as u64);
        // What the title's CPU wrote around and under it so far goes first.
        to.refresh(self, dst as u64, len as u64);
        let (common, source, dest) =
            (D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_COPY_DEST);
        // SAFETY (all): an open list; live buffers (the list holds the
        // copies and its scratch memory), ranges inside them.
        // Host copies are in the common state here and promoted by the
        // copy's use of them; each is put back in it after, as the steps
        // behind expect. The destination was left in `COPY_DEST`, where the
        // next copy out of it, a constant buffer filled from it
        // (`placed_constants`) and the compare of host copies in the card's
        // memory after the lists (a root view, in the same submission) read
        // it: 19,500 debug-layer errors in a 7-minute run.
        unsafe { wait_for_all(host, &self.list) };
        match (&own, &scratch) {
            (Some(from), _) => {
                from.refresh(self, src as u64, len as u64);
                let offset = from.offset(src as u64);
                if from.buffer.as_raw() == to.buffer.as_raw() {
                    let Some(between) = self.between(host, len as u64) else {
                        return false;
                    };
                    unsafe {
                        self.list.CopyBufferRegion(&between, 0, &from.buffer, offset, len as u64);
                        self.list.ResourceBarrier(&[
                            transition(&between, dest, source),
                            // (A promoted read state is not promoted to a write.)
                            transition(&from.buffer, source, dest),
                        ]);
                        wait_for_all(host, &self.list);
                        self.list.CopyBufferRegion(&to.buffer, at, &between, 0, len as u64);
                    }
                } else {
                    unsafe {
                        self.list.CopyBufferRegion(&to.buffer, at, &from.buffer, offset, len as u64);
                        self.list.ResourceBarrier(&[transition(&from.buffer, source, common)]);
                    }
                }
                STATS[2].fetch_add(1, Ordering::Relaxed);
            }
            (None, Some((from, offset, _))) => unsafe {
                self.list.CopyBufferRegion(&to.buffer, at, from, *offset, len as u64);
            },
            (None, None) => return false,
        }
        unsafe {
            self.list.ResourceBarrier(&[transition(&to.buffer, dest, common)]);
            wait_for_all(host, &self.list);
        }
        STATS[0].fetch_add(1, Ordering::Relaxed);
        STATS[1].fetch_add(len as u64, Ordering::Relaxed);
        to.gpu_writes(self, dst as u64, len as u64);
        let mut deferred = self.deferred();
        deferred.copies.push(Step::Placed(Placed {
            dst,
            src,
            len,
            shadow: to,
            from: scratch.map(|s| s.2 as usize),
        }));
        deferred.title_written();
        self.changed_something();
        true
    }

    /// For a copy of `[src, src + len)` into a texture: a buffer (readable by
    /// copies) that holds what shaders wrote there, at this place of the list.
    /// `None` where shaders do not write it.
    pub(crate) fn place_source(&self, src: usize, len: usize) -> Option<ID3D12Resource> {
        if !on() {
            return None;
        }
        let host = host()?;
        let from = shaders_own(src, len)?;
        let between = self.between(host, len as u64)?;
        from.refresh(self, src as u64, len as u64);
        STATS[3].fetch_add(1, Ordering::Relaxed);
        // SAFETY: an open list; live buffers, ranges inside them.
        unsafe {
            wait_for_all(host, &self.list);
            self.list.CopyBufferRegion(&between, 0, &from.buffer, from.offset(src as u64), len as u64);
            self.list.ResourceBarrier(&[
                transition(&between, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE),
                transition(&from.buffer, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_COMMON),
            ]);
            wait_for_all(host, &self.list);
        }
        Some(between)
    }

    /// For a table's constant buffer of `size` bytes at `address` that a copy
    /// at its place earlier in this list writes all of: a buffer on the card
    /// that holds those bytes at this place of the list, and its address
    /// (`VICTORY_PLACED_CONSTANTS=0`: none, the constant buffer is filled out
    /// of title memory at submission, as before).
    ///
    /// The title copies 560 bytes out of memory a dispatch writes (its big
    /// heap) to a constant buffer a few draws read, a few times a frame. The
    /// copy is done here; the fill of the constants was done on the CPU at
    /// submission, so it read what the copy writes before the card had
    /// written it, and the whole submission waited for every flight to land
    /// (`card_steps`: "a copy at its place waits for flights"): 36 ms a frame
    /// of the submitting thread on the RTX 5070 at 1080p.
    ///
    /// shortcut: kept per list for the copy that wrote it; a shader that
    /// writes the constants between two draws that read them is not seen.
    pub(crate) fn placed_constants(&self, address: u64, size: u32) -> Option<(ID3D12Resource, u64)> {
        static ON: OnceLock<bool> = OnceLock::new();
        if !on() || !*ON.get_or_init(|| std::env::var_os("VICTORY_PLACED_CONSTANTS").is_none_or(|v| v != "0")) {
            return None;
        }
        let (at, n) = (address as usize, size as usize);
        let (step, shadow) = {
            let deferred = self.deferred();
            let (step, p) = deferred.copies.iter().enumerate().rev().find_map(|(i, s)| match s {
                Step::Placed(p) if p.dst < at + n && at < p.dst + p.len => Some((i, p)),
                _ => None,
            })?;
            // Partly: as before.
            if at < p.dst || at + n > p.dst + p.len {
                return None;
            }
            if let Some(made) = deferred.placed_constants.get(&(at, size, step)) {
                // SAFETY: a live buffer.
                return Some((made.clone(), unsafe { made.GetGPUVirtualAddress() }));
            }
            (step, p.shadow.clone())
        };
        let host = host()?;
        let between = self.between(host, u64::from(size).next_multiple_of(256))?;
        // SAFETY: an open list; live buffers (the list holds both), ranges
        // inside them.
        unsafe {
            wait_for_all(host, &self.list);
            self.list.CopyBufferRegion(&between, 0, &shadow.buffer, shadow.offset(address), u64::from(size));
            self.list.ResourceBarrier(&[
                transition(&between, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER),
                transition(&shadow.buffer, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_COMMON),
            ]);
        }
        STATS_CONSTANTS.fetch_add(1, Ordering::Relaxed);
        self.deferred().placed_constants.insert((at, size, step), between.clone());
        // SAFETY: a live buffer.
        let gpu = unsafe { between.GetGPUVirtualAddress() };
        Some((between, gpu))
    }

    /// A buffer on the card for this recording alone.
    fn between(&self, host: &Host, len: u64) -> Option<ID3D12Resource> {
        let size = len.next_multiple_of(1 << 16);
        if !between_pool() {
            let buffer = host.uav_buffer(size)?;
            name(&buffer, || "bytes between copies".into());
            self.keep(&buffer);
            return Some(buffer);
        }
        // One of a list reset earlier, or a new one, made with no lock held.
        let buffer = match between_take(size) {
            Some(taken) => {
                BETWEEN_POOL_STATS[0].fetch_add(1, Ordering::Relaxed);
                taken
            }
            None => {
                let made = host.uav_buffer(size)?;
                name(&made, || "bytes between copies".into());
                BETWEEN_POOL_STATS[1].fetch_add(1, Ordering::Relaxed);
                made
            }
        };
        // Held until the list is reset, then given to the pool.
        self.deferred().between.push((buffer.clone(), size));
        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    /// What copies with two sides cost to make (ignored: a measurement for
    /// `VICTORY_CARD_ALL`): buffers of 64 KiB on the card and in the PC's
    /// memory.
    #[test]
    #[ignore]
    fn small_buffers_cost() {
        let Some(host) = super::host() else {
            return;
        };
        const N: usize = 1000;
        let used = || host.video_memory().map_or(0, |m| m[1] >> 20);
        let before = used();
        let t = std::time::Instant::now();
        let card: Vec<_> = (0..N).map(|_| host.uav_buffer(1 << 16)).collect();
        let card_us = t.elapsed().as_micros() as f64 / N as f64;
        let after = used();
        let t = std::time::Instant::now();
        let pc: Vec<_> = (0..N).map(|_| host.shared_buffer(1 << 16)).collect();
        let pc_us = t.elapsed().as_micros() as f64 / N as f64;
        println!(
            "{N} buffers of 64 KiB: on the card {card_us:.0} us each, {} MB of its memory; in the PC's memory {pc_us:.0} us each",
            after.saturating_sub(before)
        );
        drop((card, pc));
    }
}
