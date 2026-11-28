//! Diagnostics: where the time of the costliest list calls goes
//! (`VICTORY_CALL_PARTS=0`: not timed).
//!
//! The translating threads' time by kind of call (`list::thin`, "list calls
//! translated per frame by kind") is of a whole call: on the RTX 5070 at
//! 1080p some 8,500 indirect executions a frame at 29 us and 5,100 dispatches
//! at 19 us (run of 2026-11-28), 57 to 68 ms of each of eight threads, and
//! nothing said where in a call. Here the thread that translates a draw, a
//! dispatch or an indirect execution keeps the part of the call it is in; a
//! mark in the code ([`lap!`](crate::lap)) ends that part and begins another,
//! so every nanosecond of the call goes to one part, and what no mark names
//! is "the rest". The parts are printed with the frame line, in microseconds
//! a call, with the locks those calls take every time and how often another
//! thread held them.
//!
//! What it costs: a read of the clock at each mark that changes the part,
//! about a dozen in a call.

/// Diagnostics: the part `$part` (a constant of `crate::laps`) of the list
/// call this thread translates begins here. Nothing in other builds.
#[macro_export]
#[doc(hidden)]
macro_rules! lap {
    ($part:ident) => {
        #[cfg(feature = "diagnostics")]
        $crate::laps::mark($crate::laps::$part);
    };
}

/// `$work`, all of it counted as the part `$part`, whatever marks it meets
/// on its way (see [`lap!`](crate::lap)).
#[macro_export]
#[doc(hidden)]
macro_rules! lap_held {
    ($part:ident, $work:expr) => {{
        #[cfg(feature = "diagnostics")]
        let _held = $crate::laps::hold($crate::laps::$part);
        $work
    }};
}

#[cfg(feature = "diagnostics")]
pub(crate) use timed::*;

#[cfg(feature = "diagnostics")]
mod timed {
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
    use std::sync::{Mutex, MutexGuard, OnceLock, TryLockError};
    use std::time::Instant;

    /// The kinds of calls that are split.
    pub(crate) const DRAW: u8 = 0;
    pub(crate) const DRAW_INDEXED: u8 = 1;
    pub(crate) const DISPATCH: u8 = 2;
    pub(crate) const DISPATCH_MESH: u8 = 3;
    /// An indirect execution, until its kind is known ([`kind`]).
    pub(crate) const INDIRECT: u8 = 4;
    pub(crate) const INDIRECT_DRAW: u8 = 5;
    pub(crate) const INDIRECT_MESH: u8 = 6;
    pub(crate) const INDIRECT_DISPATCH: u8 = 7;
    const KINDS: usize = 8;
    const KIND_NAMES: [&str; KINDS] = [
        "draws",
        "indexed draws",
        "dispatches",
        "mesh dispatches",
        "indirect executions that went no further",
        "indirect draws",
        "indirect mesh dispatches",
        "indirect dispatches",
    ];

    /// The parts of a call.
    pub(crate) const REST: u8 = 0;
    pub(crate) const PIPELINE: u8 = 1;
    pub(crate) const ARGUMENTS: u8 = 2;
    pub(crate) const SIGNATURE: u8 = 3;
    pub(crate) const BATCH_PLACE: u8 = 4;
    pub(crate) const BATCH_OPEN: u8 = 5;
    pub(crate) const CLAMP: u8 = 6;
    pub(crate) const NOTES: u8 = 7;
    pub(crate) const HOST_CALL: u8 = 8;
    pub(crate) const AFTER: u8 = 9;
    pub(crate) const BUFFERS: u8 = 10;
    pub(crate) const PASSES: u8 = 11;
    pub(crate) const BIND_STATE: u8 = 12;
    pub(crate) const TABLES_OWN: u8 = 13;
    pub(crate) const TABLES_KEYS: u8 = 14;
    pub(crate) const TABLES_AGAIN: u8 = 15;
    pub(crate) const TABLES_MADE: u8 = 16;
    pub(crate) const USER_DATA: u8 = 17;
    pub(crate) const ROOT_VIEWS: u8 = 18;
    pub(crate) const TWINS: u8 = 19;
    const PARTS: usize = 20;
    const PART_NAMES: [&str; PARTS] = [
        "the rest",
        "the host pipeline",
        "the arguments' host copy",
        "the command signature",
        "a place in the open batch",
        "a batch opened",
        "a window or a clamp of its own",
        "our notes",
        "the host's call",
        "what follows it",
        "vertex and index copies and the targets",
        "passes of ours",
        "heaps, root signature and pipeline",
        "the title's own tables' entries",
        "the fresh tables' keys",
        "fresh tables gone through again",
        "fresh tables made",
        "user data",
        "root views",
        "twins written",
    ];

    /// The locks counted ([`locked`]): the command signatures made
    /// (`draw::host_signature`), the textures the host copied into
    /// (`texture::upload_from`), a list's steps (`HostList::deferred`).
    pub(crate) const SIGNATURES: usize = 0;
    pub(crate) const HOST_WRITTEN: usize = 1;
    pub(crate) const STEPS: usize = 2;
    const LOCKS: usize = 3;
    const LOCK_NAMES: [&str; LOCKS] = ["the command signatures'", "the host-written textures'", "their lists' steps'"];

    /// A thread's call under way, and what it has not given to the shared
    /// counters yet ([`flush`]).
    struct Thread {
        armed: bool,
        kind: u8,
        part: u8,
        /// Parts held under way ([`hold`]): marks are passed over.
        held: u32,
        /// When `part` began.
        at: Instant,
        /// The call's nanoseconds by part, and its tables found ready.
        call: [u64; PARTS],
        ready: u64,
        sums: [[u64; PARTS]; KINDS],
        calls: [u64; KINDS],
        readies: [u64; KINDS],
        /// By lock: taken, of those waited for, the nanoseconds waited.
        locks: [[u64; 3]; LOCKS],
        dirty: bool,
    }

    thread_local! {
        static THREAD: RefCell<Thread> = RefCell::new(Thread {
            armed: false,
            kind: 0,
            part: REST,
            held: 0,
            at: Instant::now(),
            call: [0; PARTS],
            ready: 0,
            sums: [[0; PARTS]; KINDS],
            calls: [0; KINDS],
            readies: [0; KINDS],
            locks: [[0; 3]; LOCKS],
            dirty: false,
        });
    }

    /// Since the last frame-rate line, all threads: nanoseconds by kind and
    /// part, calls and tables found ready by kind, and the locks as in
    /// [`Thread`].
    static SUMS: [[AtomicU64; PARTS]; KINDS] = [const { [const { AtomicU64::new(0) }; PARTS] }; KINDS];
    static CALLS: [AtomicU64; KINDS] = [const { AtomicU64::new(0) }; KINDS];
    static READIES: [AtomicU64; KINDS] = [const { AtomicU64::new(0) }; KINDS];
    static LOCKED: [[AtomicU64; 3]; LOCKS] = [const { [const { AtomicU64::new(0) }; 3] }; LOCKS];

    /// `VICTORY_CALL_PARTS=0` (diagnostics builds): the calls are not split.
    fn on() -> bool {
        static ON: OnceLock<bool> = OnceLock::new();
        *ON.get_or_init(|| std::env::var_os("VICTORY_CALL_PARTS").is_none_or(|v| v != "0"))
    }

    /// A call that is timed until this is dropped.
    pub(crate) struct Call(bool);

    /// This thread begins a call of `kind`: its time counts, by part, until
    /// what is returned is dropped.
    pub(crate) fn begin(kind: u8) -> Call {
        if !on() {
            return Call(false);
        }
        THREAD.with_borrow_mut(|t| {
            // (A call inside a call is part of it.)
            if t.armed {
                return Call(false);
            }
            (t.armed, t.kind, t.part, t.held, t.at) = (true, kind, REST, 0, Instant::now());
            Call(true)
        })
    }

    impl Drop for Call {
        fn drop(&mut self) {
            if !self.0 {
                return;
            }
            THREAD.with_borrow_mut(|t| {
                t.call[t.part as usize] += t.at.elapsed().as_nanos() as u64;
                let kind = (t.kind as usize).min(KINDS - 1);
                for (sum, part) in t.sums[kind].iter_mut().zip(&mut t.call) {
                    *sum += std::mem::take(part);
                }
                t.calls[kind] += 1;
                t.readies[kind] += std::mem::take(&mut t.ready);
                (t.armed, t.dirty) = (false, true);
            });
        }
    }

    /// The call under way is of `kind` after all (an indirect execution's,
    /// known once it is looked at).
    pub(crate) fn kind(kind: u8) {
        THREAD.with_borrow_mut(|t| {
            if t.armed {
                t.kind = kind;
            }
        });
    }

    /// The part `part` of the call under way begins.
    #[inline]
    pub(crate) fn mark(part: u8) {
        THREAD.with_borrow_mut(|t| {
            if !t.armed || t.held != 0 || t.part == part {
                return;
            }
            let now = Instant::now();
            t.call[t.part as usize] += (now - t.at).as_nanos() as u64;
            (t.part, t.at) = (part.min(PARTS as u8 - 1), now);
        });
    }

    /// A table of the call under way was found ready, with nothing to do.
    pub(crate) fn ready() {
        THREAD.with_borrow_mut(|t| {
            if t.armed {
                t.ready += 1;
            }
        });
    }

    /// A part that marks do not end, until this is dropped; it goes on
    /// until the next mark after that.
    pub(crate) struct Held;

    /// The part `part` begins, and holds whatever marks come.
    pub(crate) fn hold(part: u8) -> Held {
        mark(part);
        THREAD.with_borrow_mut(|t| t.held += 1);
        Held
    }

    impl Drop for Held {
        fn drop(&mut self) {
            THREAD.with_borrow_mut(|t| t.held = t.held.saturating_sub(1));
        }
    }

    /// `lock`, taken. In a call that is timed it is counted as `which`, and
    /// the wait for it timed if another thread held it.
    pub(crate) fn locked<T>(which: usize, lock: &Mutex<T>) -> MutexGuard<'_, T> {
        // (Lists are let go by threads that are ending too: no call of theirs
        // is timed.)
        if !THREAD.try_with(|t| t.borrow().armed).unwrap_or(false) {
            return lock.lock().unwrap_or_else(|e| e.into_inner());
        }
        let (guard, waited) = match lock.try_lock() {
            Ok(guard) => (guard, None),
            Err(TryLockError::Poisoned(e)) => (e.into_inner(), None),
            Err(TryLockError::WouldBlock) => {
                let began = Instant::now();
                let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
                (guard, Some(began.elapsed().as_nanos() as u64))
            }
        };
        THREAD.with_borrow_mut(|t| {
            let counts = &mut t.locks[which.min(LOCKS - 1)];
            counts[0] += 1;
            if let Some(ns) = waited {
                counts[1] += 1;
                counts[2] += ns;
            }
        });
        guard
    }

    /// What this thread counted goes to the shared counters (after a piece
    /// of a recording: the eight threads would pass their cache lines around
    /// at every mark).
    pub(crate) fn flush() {
        THREAD.with_borrow_mut(|t| {
            if !std::mem::take(&mut t.dirty) {
                return;
            }
            for kind in 0..KINDS {
                let calls = std::mem::take(&mut t.calls[kind]);
                if calls == 0 {
                    continue;
                }
                CALLS[kind].fetch_add(calls, Relaxed);
                READIES[kind].fetch_add(std::mem::take(&mut t.readies[kind]), Relaxed);
                for (shared, mine) in SUMS[kind].iter().zip(&mut t.sums[kind]) {
                    if *mine != 0 {
                        shared.fetch_add(std::mem::take(mine), Relaxed);
                    }
                }
            }
            for (shared, mine) in LOCKED.iter().flatten().zip(t.locks.iter_mut().flatten()) {
                if *mine != 0 {
                    shared.fetch_add(std::mem::take(mine), Relaxed);
                }
            }
        });
    }

    /// For the frame-rate line.
    pub(crate) fn line(frames: f64) {
        if !on() {
            return;
        }
        for kind in 0..KINDS {
            let calls = CALLS[kind].swap(0, Relaxed);
            let ready = READIES[kind].swap(0, Relaxed);
            let ns: Vec<u64> = SUMS[kind].iter().map(|s| s.swap(0, Relaxed)).collect();
            if calls == 0 {
                continue;
            }
            let us = |ns: u64| ns as f64 / 1e3 / calls as f64;
            let mut parts: Vec<(usize, u64)> = ns.iter().copied().enumerate().filter(|p| p.1 > 0).collect();
            parts.sort_unstable_by_key(|p| std::cmp::Reverse(p.1));
            let text: Vec<String> = parts.iter().map(|&(part, ns)| format!("{} {:.2}", PART_NAMES[part], us(ns))).collect();
            tracing::info!(
                "list calls translated per frame, their parts (VICTORY_CALL_PARTS=0: not timed): {} {:.0}x at {:.1} us, in us a call: {}; \
                 {:.2} fresh tables a call found ready",
                KIND_NAMES[kind],
                calls as f64 / frames,
                us(ns.iter().sum()),
                text.join(", "),
                ready as f64 / calls as f64
            );
        }
        let locks: Vec<[u64; 3]> = LOCKED.iter().map(|l| [0, 1, 2].map(|i| l[i].swap(0, Relaxed))).collect();
        if locks.iter().all(|l| l[0] == 0) {
            return;
        }
        let text: Vec<String> = locks
            .iter()
            .zip(LOCK_NAMES)
            .map(|(l, name)| {
                format!("{name} {:.0}x ({:.2} of them held by another thread, {:.2} ms waited)", l[0] as f64 / frames, l[1] as f64 / frames, l[2] as f64 / 1e6 / frames)
            })
            .collect();
        tracing::info!("locks the translating threads took in those calls, per frame: {}", text.join(", "));
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// What this thread counted and has not given on: the nanoseconds of
        /// `kind` by part, and its calls.
        fn counted(kind: u8) -> ([u64; PARTS], u64) {
            THREAD.with_borrow_mut(|t| (std::mem::take(&mut t.sums[kind as usize]), std::mem::take(&mut t.calls[kind as usize])))
        }

        fn work() {
            let began = Instant::now();
            while began.elapsed().as_micros() < 200 {
                std::hint::spin_loop();
            }
        }

        /// Every part between two marks gets its time, and together they are
        /// the call's.
        #[test]
        fn the_parts_of_a_call_are_its_time() {
            let began = Instant::now();
            let call = begin(INDIRECT);
            work();
            mark(PIPELINE);
            work();
            kind(INDIRECT_DISPATCH);
            mark(HOST_CALL);
            work();
            ready();
            drop(call);
            let all = began.elapsed().as_nanos() as u64;
            assert_eq!(counted(INDIRECT), ([0; PARTS], 0));
            let (parts, calls) = counted(INDIRECT_DISPATCH);
            assert_eq!(calls, 1);
            for part in [REST, PIPELINE, HOST_CALL] {
                assert!(parts[part as usize] >= 200_000, "{parts:?}");
            }
            let sum: u64 = parts.iter().sum();
            assert_eq!(sum, parts[REST as usize] + parts[PIPELINE as usize] + parts[HOST_CALL as usize]);
            assert!(sum <= all, "{sum} of {all}");
            assert_eq!(THREAD.with_borrow_mut(|t| std::mem::take(&mut t.readies[INDIRECT_DISPATCH as usize])), 1);
        }

        /// Outside a call marks, holds and locks count nothing.
        #[test]
        fn nothing_is_counted_outside_a_call() {
            mark(PIPELINE);
            drop(hold(PASSES));
            ready();
            drop(locked(STEPS, &Mutex::new(0)));
            work();
            mark(HOST_CALL);
            THREAD.with_borrow(|t| {
                assert!(!t.dirty && !t.armed && t.held == 0);
                assert_eq!((t.sums, t.calls, t.locks, t.ready), ([[0; PARTS]; KINDS], [0; KINDS], [[0; 3]; LOCKS], 0));
            });
        }

        /// A part held keeps the marks inside it from ending it, and a call
        /// begun inside a call is part of it.
        #[test]
        fn a_held_part_takes_what_is_marked_inside_it() {
            let call = begin(DRAW);
            {
                let _held = hold(PASSES);
                work();
                mark(TABLES_MADE);
                drop(begin(DISPATCH));
                work();
            }
            mark(AFTER);
            // A lock nobody holds is counted, with no wait.
            drop(locked(SIGNATURES, &Mutex::new(0)));
            drop(call);
            assert_eq!(counted(DISPATCH).1, 0);
            let (parts, calls) = counted(DRAW);
            assert_eq!(calls, 1);
            assert_eq!(parts[TABLES_MADE as usize], 0);
            assert!(parts[PASSES as usize] >= 400_000, "{parts:?}");
            assert_eq!(THREAD.with_borrow_mut(|t| std::mem::take(&mut t.locks)), [[1, 0, 0], [0; 3], [0; 3]]);
        }
    }
}
