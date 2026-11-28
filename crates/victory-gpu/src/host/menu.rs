//! The F1 settings menu, in every build. F1 shows and hides it while our
//! window has the focus; Up/Down pick a line, Left/Right (or Enter) change
//! the setting picked, Enter runs an action line, Esc closes. While it is
//! shown the title sees no keyboard ([`crate::SETTINGS_MENU_OPEN`], read by
//! the input module); a pad still works.
//!
//! Each line is one of this fork's `VICTORY_*` switches. A change is saved at
//! once to `victory-settings.txt` in the data folder ([`set_settings_file`],
//! which the program calls; never a file inside the repository), one
//! `VICTORY_NAME=value` line each (an empty value: unset), and
//! `scripts\play.ps1` applies that file over its own settings at every start,
//! under what is given on its command line. The switches are read once at
//! start, so a change applies at the next start, except those marked LIVE
//! (the stats panel, and frames shown per frame rendered while this start has
//! frame generation ready). Without a settings file (a start not through
//! `victory run`) the menu shows but cannot save, and says so.
//!
//! Drawn as the stats panel is: white text on a black box, copied onto the
//! swap chain's buffer, in the middle of the picture.
//!
//! (From VIctory-NVIDIA 22c6be2, with what 29d8dbf, 10862a6, 0cef07e,
//! 64d4931 and c1cbdc9 changed of it, trustno1-coder; cut down to the
//! switches this fork reads, with no restart, preload, save sharing or
//! frame-rate line.)

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_DOWN, VK_ESCAPE, VK_F1, VK_LEFT, VK_RETURN, VK_RIGHT, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use super::font::{colours, glyph};
use super::{Host, HostList, transition};

const GLYPH: (u32, u32) = (5, 7);
const CELL: (u32, u32) = (6, 9);
const COLUMNS: u32 = 60;
/// Lines of the box: a title, a help line, a blank, the lines shown, a
/// blank, what the line picked does ([`Menu::about`]), a note and a last
/// line on where the settings go.
const ROWS: usize = 20;
const LINES: u32 = ROWS as u32 + 9;
/// The last line of the box.
const LAST: &str = "SAVED AT ONCE. PLAY.PS1 APPLIES THEM AT EVERY START";
/// The note while nothing else is said.
const NOTE: &str = "* = CHANGED: APPLIES AT THE NEXT START UNLESS LIVE";
const RIM: u32 = 4;
const UPLOADS: usize = 4;

/// A setting: the switch, its name in the menu, the values it takes (`""`:
/// not set, the switch's own default) and whether a change applies at once.
struct Item {
    var: &'static str,
    label: &'static str,
    values: &'static [(&'static str, &'static str)],
    live: bool,
}

const UNSET_ON: &[(&str, &str)] = &[("", "OFF"), ("1", "ON")];
const ON_UNSET: &[(&str, &str)] = &[("1", "ON"), ("", "OFF")];
const STAGES: &[(&str, &str)] = &[("2", "2"), ("1", "1"), ("0", "0 OFF")];

const DLSS: &str = "VICTORY_DLSS";
const DLSS_MODE: &str = "VICTORY_DLSS_MODE";
const FRAMEGEN: &str = "VICTORY_FRAMEGEN";
const FRAMEGEN_X: &str = "VICTORY_FRAMEGEN_X";
const RENDER_SCALE: &str = "VICTORY_RENDER_SCALE";
const PANEL: &str = "VICTORY_PANEL";

/// A setting that applies at the next start.
const fn item(
    var: &'static str,
    label: &'static str,
    values: &'static [(&'static str, &'static str)],
) -> Item {
    Item {
        var,
        label,
        values,
        live: false,
    }
}

/// The first value of each is what `scripts\play.ps1` starts with when
/// nothing is saved and nothing given. Every switch here is read where it is
/// named (and documented there); the values are those it takes.
#[rustfmt::skip]
const ITEMS: &[Item] = &[
    item(DLSS, "DLSS", UNSET_ON),
    item(DLSS_MODE, "DLSS MODE", &[
        ("quality", "QUALITY"), ("balanced", "BALANCED"), ("performance", "PERFORMANCE"),
        ("dlaa", "DLAA (NATIVE)"),
    ]),
    item("VICTORY_DLSS_PRESET", "DLSS MODEL", &[
        ("", "DEFAULT"), ("k", "K"), ("j", "J"), ("m", "M"), ("l", "L"),
    ]),
    item(FRAMEGEN, "FRAME GENERATION", UNSET_ON),
    // Live while this start has frame generation ready ([`live`]).
    item(FRAMEGEN_X, "FRAMES SHOWN PER FRAME", &[
        ("2", "2X"), ("3", "3X"), ("4", "4X"), ("5", "5X"), ("6", "6X (MOST)"),
        ("0", "0 (READY, OFF)"),
    ]),
    item("VICTORY_OUTPUT", "OUTPUT SIZE", &[
        ("2560x1440", "2560X1440"), ("1920x1080", "1920X1080"), ("1664x936", "1664X936"),
        ("1280x720", "1280X720"), ("3840x2160", "3840X2160"), ("", "THE GAME OWN (4K)"),
    ]),
    item(RENDER_SCALE, "RENDER SCALE", &[
        ("1.0", "1.0 (FULL)"), ("0.9", "0.9"), ("0.8", "0.8"), ("0.75", "0.75"),
        ("0.667", "0.667"), ("0.5", "0.5"), ("", "THE GAME OWN (2/3)"),
    ]),
    Item { var: PANEL, label: "STATS PANEL (F3)", values: UNSET_ON, live: true },
    // Window and input (VIctory-NVIDIA 06afe63, 4a0c8ea, 12b1d16): off unless `=1`.
    item("VICTORY_FULLSCREEN", "FULLSCREEN (NO BORDER)", UNSET_ON),
    item("VICTORY_MOUSE_CAPTURE", "MOUSE CAPTURE", UNSET_ON),
    item("VICTORY_IME_ENGLISH", "ENGLISH KEYBOARD", UNSET_ON),
    // Light.
    item("VICTORY_AMBIENT", "LIGHT ESTIMATE", &[
        ("0.8", "0.8"), ("1", "1"), ("0.5", "0.5"), ("", "OFF"),
    ]),
    item("VICTORY_AMBIENT_AIR", "FOG CAP", &[
        ("", "NONE"), ("1.5", "1.5"), ("1", "1"), ("2", "2"),
    ]),
    item("VICTORY_BVH_BUILDER", "RAY TRACING (SLOW)", UNSET_ON),
    // Speed.
    item("VICTORY_EARLY_DEPTH", "EARLY DEPTH TEST", &[("", "ON"), ("0", "OFF")]),
    item("VICTORY_WC_NO_BACK", "SKIP BIG COPY-BACKS", UNSET_ON),
    item("VICTORY_TEXTURE_NO_WAIT", "TEXTURE NO WAIT", ON_UNSET),
    item("VICTORY_RELAY", "RELAY", STAGES),
    item("VICTORY_CARD_STEPS", "CARD STEPS", STAGES),
    item("VICTORY_INDIRECT_BATCH", "INDIRECT BATCH", STAGES),
    item("VICTORY_CARD_BUFFERS_MIN_KB", "CARD COMPARES FROM (KB)", &[
        ("", "DEFAULT"), ("64", "64"), ("1024", "1024"),
    ]),
];

/// The lines after the settings.
const ACTIONS: [&str; 2] = ["RESET ALL TO PLAY DEFAULTS (NEXT START)", "CLOSE"];
const RESET: usize = 0;

/// The settings file's first lines.
const HEADER: &str = "# Saved by the F1 settings menu: one VICTORY_NAME=value a line (an empty \
                      value: unset).\r\n# scripts\\play.ps1 applies them over its own settings \
                      at every start, under what its command line gives (-NoSaved: not).\r\n";

/// Where the menu saves ([`set_settings_file`]).
static SETTINGS_FILE: OnceLock<PathBuf> = OnceLock::new();

/// Where the F1 menu saves its settings: `<data_dir>\victory-settings.txt`,
/// which `scripts\play.ps1` reads at every start. Set by the program once
/// it knows the data folder; without it the menu cannot save.
pub fn set_settings_file(path: PathBuf) {
    let _ = SETTINGS_FILE.set(path);
}

fn settings_file() -> Option<&'static Path> {
    SETTINGS_FILE.get().map(PathBuf::as_path)
}

fn get(var: &str) -> String {
    std::env::var(var).unwrap_or_default()
}

/// Whether a change of `item` shows at once. Frames shown per frame
/// rendered do while this start has frame generation ready.
fn live(item: &Item) -> bool {
    item.live || (item.var == FRAMEGEN_X && crate::streamline::framegen_live())
}

/// The index of `value` among the item's values.
fn index_of(item: &Item, value: &str) -> Option<usize> {
    item.values.iter().position(|(v, _)| *v == value)
}

/// What the menu shows for `value`: its label, an unknown value as itself,
/// and an empty value it has no label for as "NOT SET".
fn shown_value(item: &Item, value: &str) -> String {
    match index_of(item, value) {
        Some(i) => item.values[i].1.to_owned(),
        None if value.is_empty() => "NOT SET".to_owned(),
        None => value.to_ascii_uppercase(),
    }
}

/// Keys pressed, gathered on a thread of their own every 10 ms: a frame
/// lasts 50 ms and more, and a short press would fall between two frames.
fn keys() -> &'static Mutex<Vec<u16>> {
    static KEYS: OnceLock<Mutex<Vec<u16>>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let _ = std::thread::Builder::new()
            .name("settings menu keys".into())
            .spawn(|| {
                const WATCHED: [u16; 7] = [
                    VK_F1, VK_UP, VK_DOWN, VK_LEFT, VK_RIGHT, VK_RETURN, VK_ESCAPE,
                ];
                let mut was = [false; WATCHED.len()];
                loop {
                    std::thread::sleep(Duration::from_millis(10));
                    let mut process = 0;
                    // SAFETY: plain queries; room for the answer.
                    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut process) };
                    let ours = process == std::process::id();
                    for (i, &vk) in WATCHED.iter().enumerate() {
                        // SAFETY: a plain query.
                        let down = ours && unsafe { GetAsyncKeyState(i32::from(vk)) } < 0;
                        if down && !was[i] {
                            keys().lock().unwrap_or_else(|e| e.into_inner()).push(vk);
                        }
                        was[i] = down;
                    }
                }
            });
        Mutex::new(Vec::new())
    })
}

/// The box's size in pixels.
const fn size(scale: u32) -> (u32, u32) {
    (
        (COLUMNS * CELL.0 + 2 * RIM) * scale,
        (LINES * CELL.1 + 2 * RIM) * scale,
    )
}

fn lit(lines: &[String], x: u32, y: u32) -> bool {
    let (Some(x), Some(y)) = (x.checked_sub(RIM), y.checked_sub(RIM)) else {
        return false;
    };
    if x % CELL.0 >= GLYPH.0 || y % CELL.1 >= GLYPH.1 || x / CELL.0 >= COLUMNS {
        return false;
    }
    let Some(&c) = lines
        .get((y / CELL.1) as usize)
        .and_then(|l| l.as_bytes().get((x / CELL.0) as usize))
    else {
        return false;
    };
    (glyph(c.to_ascii_uppercase())[(y % CELL.1) as usize] >> (GLYPH.0 - 1 - x % CELL.0)) & 1 != 0
}

/// The box as rows of `pitch` bytes.
fn painted(lines: &[String], scale: u32, colours: [&[u8]; 2], pitch: usize) -> Vec<u8> {
    let (width, height) = size(scale);
    let bytes = colours[0].len();
    let mut out = vec![0; pitch * height as usize];
    for y in 0..height / scale {
        let top = pitch * (y * scale) as usize;
        for x in 0..width / scale {
            let colour = colours[usize::from(lit(lines, x, y))];
            for i in 0..scale {
                let at = top + (x * scale + i) as usize * bytes;
                out[at..at + bytes].copy_from_slice(colour);
            }
        }
        for i in 1..scale as usize {
            out.copy_within(top..top + pitch, top + i * pitch);
        }
    }
    out
}

/// The buffers of a box that size and format (as the stats panel's).
struct Buffers {
    format: DXGI_FORMAT,
    scale: u32,
    pitch: u32,
    uploads: Vec<(ID3D12Resource, *mut u8)>,
    card: ID3D12Resource,
    /// The upload written last.
    turn: usize,
    /// It is not in the card's buffer yet.
    fresh: bool,
}

// SAFETY: free-threaded resources, and memory only written under the presenter's lock.
unsafe impl Send for Buffers {}

impl Buffers {
    fn new(host: &Host, format: DXGI_FORMAT, scale: u32) -> Option<Buffers> {
        let (width, height) = size(scale);
        let pitch = (width * colours(format)?[0].len() as u32).next_multiple_of(256);
        let bytes = u64::from(pitch) * u64::from(height);
        let uploads = (0..UPLOADS)
            .map(|_| host.upload_buffer(bytes))
            .collect::<Option<Vec<_>>>()?;
        let heap = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            ..Default::default()
        };
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: bytes,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            ..Default::default()
        };
        let mut card: Option<ID3D12Resource> = None;
        // SAFETY: plain creation of a buffer.
        unsafe {
            host.device
                .CreateCommittedResource(
                    &heap,
                    D3D12_HEAP_FLAG_NONE,
                    &desc,
                    D3D12_RESOURCE_STATE_COMMON,
                    None,
                    &mut card,
                )
                .ok()?;
        }
        Some(Buffers {
            format,
            scale,
            pitch,
            uploads,
            card: card?,
            turn: 0,
            fresh: false,
        })
    }

    fn write(&mut self, lines: &[String]) {
        let Some(colours) = colours(self.format) else {
            return;
        };
        let picture = painted(lines, self.scale, colours, self.pitch as usize);
        self.turn = (self.turn + 1) % UPLOADS;
        // SAFETY: a mapped upload buffer of the picture's size, which the card read frames ago.
        unsafe {
            std::ptr::copy_nonoverlapping(
                picture.as_ptr(),
                self.uploads[self.turn].1,
                picture.len(),
            )
        };
        self.fresh = true;
    }
}

/// The menu of a presenter.
pub(super) struct Menu {
    /// The values chosen, by item (as at start until changed).
    values: Vec<String>,
    /// The values at start: what differs applies at the next start.
    started: Vec<String>,
    /// The values as last saved (at first, as at start): what differs is
    /// written at the next save.
    saved: Vec<String>,
    /// The line picked: an item, or an action after them.
    at: usize,
    note: String,
    changed: bool,
    buffers: Option<Buffers>,
    failed: bool,
}

impl Menu {
    pub(super) fn new() -> Menu {
        // Start the key thread now: F1 must be seen while the menu is closed.
        let _ = keys();
        // Diagnostics: `VICTORY_MENU_OPEN=1` shows the menu from the start (to
        // look at it in a run nobody presses keys in). Off unless set to 1.
        #[cfg(feature = "diagnostics")]
        if std::env::var_os("VICTORY_MENU_OPEN").is_some_and(|v| v == "1") {
            crate::SETTINGS_MENU_OPEN.store(true, Ordering::Relaxed);
        }
        Menu::with(ITEMS.iter().map(|i| get(i.var)).collect())
    }

    fn with(values: Vec<String>) -> Menu {
        Menu {
            started: values.clone(),
            saved: values.clone(),
            values,
            at: 0,
            note: String::new(),
            changed: true,
            buffers: None,
            failed: false,
        }
    }

    fn open() -> bool {
        crate::SETTINGS_MENU_OPEN.load(Ordering::Relaxed)
    }

    /// What is said of the line picked, under the lines.
    fn about(&self) -> [&'static str; 3] {
        if self.at == ITEMS.len() + RESET {
            return [
                "REMOVES THE SAVED SETTINGS: THE NEXT START HAS",
                "PLAY.PS1 OWN SETTINGS. THIS START KEEPS WHAT IT HAS.",
                "",
            ];
        }
        match ITEMS.get(self.at).map(|i| i.var) {
            Some(DLSS) => [
                "NVIDIA DLSS DOES THE GAME UPSCALING, WITH NVIDIA FILES",
                "IN THE DATA FOLDER (SCRIPTS/GET-NVIDIA-SDKS.PS1).",
                "WITHOUT THEM, OR ON ANOTHER CARD, NOTHING CHANGES.",
            ],
            Some(FRAMEGEN) => [
                "DLSS FRAME GENERATION. NEEDS DLSS ON AND AN OUTPUT",
                "SMALLER THAN 3840X2160. HOW MANY FRAMES: NEXT LINE.",
                "",
            ],
            Some(FRAMEGEN_X) => [
                "FRAMES SHOWN FOR EACH ONE RENDERED. LIVE WHILE THIS",
                "START HAS FRAME GENERATION, ELSE AT THE NEXT START.",
                "0: MADE READY BUT OFF. MORE THAN 2X NEEDS AN RTX 50.",
            ],
            Some(RENDER_SCALE) => [
                "THE SCALE THE SCENE RENDERS AT, OF THE OUTPUT SIZE.",
                "NOT USED WHILE DLSS IS ON: ITS MODE SETS THE SCALE.",
                "",
            ],
            _ => [""; 3],
        }
    }

    fn lines(&self) -> Vec<String> {
        let total = ITEMS.len() + ACTIONS.len();
        let first = self
            .at
            .saturating_sub(ROWS / 2)
            .min(total.saturating_sub(ROWS));
        let mut out = vec![
            "SETTINGS                       F1 OR ESC CLOSES".to_owned(),
            "UP/DOWN PICK  LEFT/RIGHT CHANGE  ENTER DOES IT".to_owned(),
            String::new(),
        ];
        for row in first..(first + ROWS).min(total) {
            let cursor = if row == self.at { '>' } else { ' ' };
            let line = if let Some(item) = ITEMS.get(row) {
                let value = &self.values[row];
                let shown = shown_value(item, value);
                let mark = match (*value != self.started[row], live(item)) {
                    (true, true) => " *LIVE",
                    (true, false) => " *",
                    (false, true) => " LIVE",
                    (false, false) => "",
                };
                format!("{cursor} {:<27} < {shown} >{mark}", item.label)
            } else {
                format!("{cursor} {}", ACTIONS[row - ITEMS.len()])
            };
            out.push(line.chars().take(COLUMNS as usize).collect());
        }
        while out.len() < 3 + ROWS {
            out.push(String::new());
        }
        out.push(String::new());
        out.extend(self.about().iter().map(|l| (*l).to_owned()));
        out.push(if self.note.is_empty() {
            NOTE.to_owned()
        } else {
            self.note.clone()
        });
        out.push(LAST.to_owned());
        out
    }

    /// Changes item `i` by `step` (+1 / -1) through its values.
    fn change(&mut self, i: usize, step: isize) {
        let item = &ITEMS[i];
        let n = item.values.len() as isize;
        let now = index_of(item, &self.values[i]).map_or(-1, |x| x as isize);
        let next = if now < 0 {
            if step > 0 { 0 } else { n - 1 }
        } else {
            (now + step).rem_euclid(n)
        };
        self.values[i] = item.values[next as usize].0.to_owned();
        if live(item) {
            self.apply_live(i);
        }
        self.note.clear();
    }

    /// A setting that changes while the title runs: changed at once.
    fn apply_live(&self, i: usize) {
        let value = &self.values[i];
        match ITEMS[i].var {
            // Off (0) or the frames shown for each one rendered, from the next present on.
            FRAMEGEN_X => crate::streamline::set_framegen(value.parse().unwrap_or(0)),
            PANEL => {
                super::panel::shown().store(!matches!(value.as_str(), "" | "0"), Ordering::Relaxed)
            }
            _ => {}
        }
    }

    /// Writes the values changed since the last save over those in the
    /// settings file.
    fn save(&mut self) -> bool {
        let Some(path) = settings_file() else {
            self.note = "NOT SAVED: NO SETTINGS FILE IN THIS START".to_owned();
            return false;
        };
        // The settings are not the title's data, but the data folder is never
        // in the repository; a file that would be is refused.
        if let Err(e) = victory_core::ensure_outside_repo(path) {
            tracing::warn!(error = %e, "settings menu: not saved");
            self.note = "NOT SAVED: THE FILE WOULD BE IN THE REPOSITORY".to_owned();
            return false;
        }
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let mut kept: Vec<(String, String)> = std::fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .filter_map(|l| {
                l.split_once('=')
                    .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
            })
            .filter(|(k, _)| k.starts_with("VICTORY_"))
            .collect();
        for (i, item) in ITEMS.iter().enumerate() {
            if self.values[i] == self.saved[i] {
                continue;
            }
            kept.retain(|(k, _)| k != item.var);
            kept.push((item.var.to_owned(), self.values[i].clone()));
        }
        let mut text = String::from(HEADER);
        for (k, v) in &kept {
            text.push_str(&format!("{k}={v}\r\n"));
        }
        match std::fs::write(path, text) {
            Ok(()) => {
                tracing::info!(path = %path.display(), "settings menu: saved");
                self.saved = self.values.clone();
                self.note = "SAVED. APPLIES AT THE NEXT START".to_owned();
                true
            }
            Err(e) => {
                tracing::warn!(error = %e, path = %path.display(), "settings menu: not saved");
                self.note = "COULD NOT SAVE THE SETTINGS FILE".to_owned();
                false
            }
        }
    }

    /// Removes the settings file: `scripts\play.ps1`'s own settings at the
    /// next start.
    fn reset(&mut self) {
        let Some(path) = settings_file() else {
            self.note = "NO SETTINGS FILE IN THIS START".to_owned();
            return;
        };
        match std::fs::remove_file(path) {
            Ok(()) => {
                tracing::info!(path = %path.display(), "settings menu: saved settings removed")
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(error = %e, path = %path.display(), "settings menu: not reset");
                self.note = "COULD NOT REMOVE THE SETTINGS FILE".to_owned();
                return;
            }
        }
        // What this start has is no longer saved anywhere, and what was
        // changed in the menu since is as at start again: left as changed, the
        // lines kept their stars with the file gone, and the next save of
        // another line wrote them all back.
        self.saved = self.started.clone();
        let changed: Vec<usize> = (0..ITEMS.len())
            .filter(|&i| self.values[i] != self.started[i])
            .collect();
        self.values = self.started.clone();
        for i in changed {
            if live(&ITEMS[i]) {
                self.apply_live(i);
            }
        }
        self.note = "RESET. PLAY DEFAULTS AT THE NEXT START".to_owned();
    }

    /// The stats panel's line follows the panel: F3 shows and hides it
    /// without the menu. What F3 did is not a change of the menu's (it is
    /// not saved with the next line that is).
    fn follow_panel(&mut self) {
        let Some(i) = ITEMS.iter().position(|item| item.var == PANEL) else {
            return;
        };
        let shown = super::panel::shown().load(Ordering::Relaxed);
        if shown != !matches!(self.values[i].as_str(), "" | "0") {
            let value = if shown { "1" } else { "" };
            self.values[i] = value.to_owned();
            self.saved[i] = value.to_owned();
            self.changed = true;
        }
    }

    /// Changes a setting and saves it at once: players changed settings and
    /// closed the menu without a save line, and lost them (VIctory-NVIDIA
    /// 64d4931).
    fn change_and_save(&mut self, i: usize, step: isize) {
        self.change(i, step);
        if self.save() && live(&ITEMS[i]) {
            self.note = "SAVED. ON NOW".to_owned();
        }
    }

    /// A frame was presented: the keys pressed since the last one.
    pub(super) fn frame(&mut self) {
        let pressed = std::mem::take(&mut *keys().lock().unwrap_or_else(|e| e.into_inner()));
        if pressed.is_empty() {
            return;
        }
        let total = ITEMS.len() + ACTIONS.len();
        for vk in pressed {
            let open = Self::open();
            if vk == VK_F1 || (open && vk == VK_ESCAPE) {
                crate::SETTINGS_MENU_OPEN.store(!open && vk == VK_F1, Ordering::Relaxed);
                self.note.clear();
                self.changed = true;
                continue;
            }
            if !open {
                continue;
            }
            let at = self.at;
            match vk {
                VK_UP => self.at = (at + total - 1) % total,
                VK_DOWN => self.at = (at + 1) % total,
                VK_LEFT if at < ITEMS.len() => self.change_and_save(at, -1),
                VK_RIGHT | VK_RETURN if at < ITEMS.len() => self.change_and_save(at, 1),
                VK_RETURN if at - ITEMS.len() == RESET => self.reset(),
                VK_RETURN => crate::SETTINGS_MENU_OPEN.store(false, Ordering::Relaxed),
                _ => {}
            }
            self.changed = true;
        }
    }

    /// Records the copy of the box onto `back` (of `width` x `height`, in the
    /// copy-destination state), in its middle, while the menu is shown.
    pub(super) fn draw(
        &mut self,
        host: &Host,
        list: &HostList,
        back: &ID3D12Resource,
        (width, height): (u32, u32),
        format: DXGI_FORMAT,
    ) {
        if self.failed || !Self::open() {
            return;
        }
        self.follow_panel();
        // Not the title's: the frame recorder (diagnostics builds) leaves it out.
        #[cfg(feature = "diagnostics")]
        let _ours = crate::replay::record::unrecorded();
        let scale = (height / 540).max(1);
        if self
            .buffers
            .as_ref()
            .is_none_or(|b| (b.format, b.scale) != (format, scale))
        {
            // (What the card may still read stays: a picture's size changes
            // once or twice in a run.)
            std::mem::forget(self.buffers.take());
            self.buffers = Buffers::new(host, format, scale);
            self.failed = self.buffers.is_none();
            self.changed = true;
        }
        let lines = self.changed.then(|| self.lines());
        let Some(buffers) = &mut self.buffers else {
            return;
        };
        if let Some(lines) = lines {
            self.changed = false;
            buffers.write(&lines);
        }
        let (w, h) = size(scale);
        let (w, h) = (w.min(width), h.min(height));
        let (x, y) = ((width - w) / 2, (height - h) / 2);
        let list = &list.list;
        // SAFETY: an open list and live resources; the box lies within both.
        unsafe {
            if std::mem::take(&mut buffers.fresh) {
                // (A buffer is back in the common state after each submission
                // of a list, as the stats panel's.)
                list.CopyResource(&buffers.card, &buffers.uploads[buffers.turn].0);
                list.ResourceBarrier(&[transition(
                    &buffers.card,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                )]);
            }
            let (bw, bh) = size(scale);
            let from = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&buffers.card),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                        Offset: 0,
                        Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                            Format: format,
                            Width: bw,
                            Height: bh,
                            Depth: 1,
                            RowPitch: buffers.pitch,
                        },
                    },
                },
            };
            let to = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(back),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: 0,
                },
            };
            let part = D3D12_BOX {
                left: 0,
                top: 0,
                front: 0,
                right: w,
                bottom: h,
                back: 1,
            };
            list.CopyTextureRegion(&to, x, y, 0, &from, Some(&part));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> Menu {
        let mut menu = Menu::with(ITEMS.iter().map(|_| String::new()).collect());
        menu.changed = false;
        menu
    }

    /// Every line of the box, and every note, fits it and has its letters.
    fn fits(line: &str) {
        assert!(line.len() <= COLUMNS as usize, "{line}");
        for c in line.bytes() {
            assert_ne!(
                glyph(c.to_ascii_uppercase()),
                glyph(b'#'),
                "{line}: {}",
                c as char
            );
        }
    }

    #[test]
    fn every_line_fits_and_every_character_has_a_picture() {
        let mut menu = menu();
        for at in 0..ITEMS.len() + ACTIONS.len() {
            menu.at = at;
            let lines = menu.lines();
            assert_eq!(lines.len(), LINES as usize);
            assert_eq!(lines[lines.len() - 1], LAST);
            lines.iter().for_each(|l| fits(l));
        }
        for item in ITEMS {
            fits(item.label);
            assert!(item.label.len() <= 27, "{}", item.label);
            for (_, shown) in item.values {
                fits(shown);
            }
        }
        for note in [
            NOTE,
            "SAVED. ON NOW",
            "SAVED. APPLIES AT THE NEXT START",
            "NOT SAVED: NO SETTINGS FILE IN THIS START",
            "NOT SAVED: THE FILE WOULD BE IN THE REPOSITORY",
            "COULD NOT SAVE THE SETTINGS FILE",
            "COULD NOT REMOVE THE SETTINGS FILE",
            "RESET. PLAY DEFAULTS AT THE NEXT START",
        ] {
            fits(note);
        }
    }

    #[test]
    fn every_item_is_a_switch_once() {
        for (i, item) in ITEMS.iter().enumerate() {
            assert!(item.var.starts_with("VICTORY_"), "{}", item.var);
            assert!(!item.values.is_empty(), "{}", item.var);
            assert!(
                ITEMS[i + 1..].iter().all(|o| o.var != item.var),
                "{}",
                item.var
            );
        }
    }

    #[test]
    fn values_cycle_both_ways() {
        let mut menu = Menu::with(ITEMS.iter().map(|i| i.values[0].0.to_owned()).collect());
        let n = ITEMS[0].values.len();
        for _ in 0..n {
            menu.change(0, 1);
        }
        assert_eq!(menu.values[0], ITEMS[0].values[0].0);
        menu.change(0, -1);
        assert_eq!(menu.values[0], ITEMS[0].values[n - 1].0);
    }

    #[test]
    fn a_change_is_saved_at_once() {
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("victory-settings.txt");
        set_settings_file(path.clone());
        // (Another test may have set it first; this one is the only one that does.)
        assert_eq!(settings_file(), Some(path.as_path()));
        let mut menu = menu();
        let dlss = ITEMS
            .iter()
            .position(|i| i.var == DLSS)
            .expect("in the menu");
        menu.change_and_save(dlss, 1);
        let text = std::fs::read_to_string(&path).expect("saved without a save line");
        assert!(text.contains("VICTORY_DLSS=1\r\n"), "{text}");
        assert_eq!(menu.note, "SAVED. APPLIES AT THE NEXT START");
        // Back to off: written as an empty value (unset), once.
        menu.change_and_save(dlss, 1);
        let text = std::fs::read_to_string(&path).expect("saved");
        assert_eq!(text.matches("VICTORY_DLSS=").count(), 1, "{text}");
        assert!(text.contains("VICTORY_DLSS=\r\n"), "{text}");
        // Still marked as changed from this start? No: it is as at start again.
        assert!(menu.lines().iter().all(|l| !l.contains(" *")));
        // A reset takes back what was changed since the start: nothing stays
        // marked, and the next save does not write it again.
        menu.change_and_save(dlss, 1);
        menu.reset();
        assert!(!path.exists());
        assert_eq!(menu.values, menu.started);
        let early = ITEMS
            .iter()
            .position(|i| i.var == "VICTORY_EARLY_DEPTH")
            .expect("in the menu");
        menu.change_and_save(early, 1);
        let text = std::fs::read_to_string(&path).expect("saved");
        assert!(!text.contains("VICTORY_DLSS="), "{text}");
        menu.reset();
        assert!(!path.exists());
    }
}
