//! `ui` — a cohesive, accessible terminal output layer for the `rocket` CLI.
//!
//! This module is intentionally **std-only** for its rendering core: every visual
//! primitive (colors, glyphs, tables, progress bars, spinners) is implemented with
//! hand-rolled ANSI handling and pure `-> String` render functions so it always
//! compiles and is trivially unit-testable. The only non-std dependency used is
//! `serde`/`serde_json` for the structured `--json` envelope, both of which are
//! already first-party dependencies of `rocket-sdk`.
//!
//! ## Design goals
//! - **Accessible by default.** Respects `NO_COLOR`, honors `CLICOLOR_FORCE`, and
//!   degrades to ASCII glyphs when color/UTF-8 is unavailable. Color is never
//!   emitted unless the sink is a TTY (the caller passes `is_tty`).
//! - **Testable.** Animation and styling are pure functions returning `String`;
//!   the stderr-writing wrappers are thin shells around them.
//! - **Machine-readable.** Every command can opt into `OutputMode::Json` and emit a
//!   structured envelope instead of human prose, via [`Output::emit`].
//!
//! Nothing here writes to stdout/stderr implicitly except the explicitly-named
//! `*_to`/`print*` helpers; the rendering functions are side-effect free.

use std::fmt;
use std::io::Write;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// ANSI color codes
// ---------------------------------------------------------------------------

/// Raw ANSI SGR escape sequences. Kept private-ish but `pub` so callers can build
/// bespoke output when the high-level helpers do not fit.
pub mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";

    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

    pub const BRIGHT_BLACK: &str = "\x1b[90m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
}

// ---------------------------------------------------------------------------
// Color support detection
// ---------------------------------------------------------------------------

/// Whether ANSI color should be emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Emit ANSI color sequences.
    Always,
    /// Never emit color (plain text).
    Never,
}

impl ColorMode {
    /// Detect color support from the environment plus a caller-supplied `is_tty`.
    ///
    /// Precedence (highest first), matching the de-facto cross-tool convention:
    /// 1. `NO_COLOR` set (to anything, even empty) → [`ColorMode::Never`].
    /// 2. `CLICOLOR_FORCE` set to a non-`"0"` value → [`ColorMode::Always`].
    /// 3. Otherwise color iff `is_tty`.
    ///
    /// `is_tty` is injected (not detected here) so tests are deterministic and the
    /// SDK stays free of platform `isatty` plumbing — `rocket-cmd` passes
    /// `std::io::stderr().is_terminal()`.
    pub fn detect(is_tty: bool) -> Self {
        Self::detect_with(is_tty, |k| std::env::var(k).ok())
    }

    /// Same as [`detect`](Self::detect) but with an injectable env lookup, for tests.
    pub fn detect_with(is_tty: bool, get_env: impl Fn(&str) -> Option<String>) -> Self {
        if get_env("NO_COLOR").is_some() {
            return ColorMode::Never;
        }
        if get_env("CLICOLOR_FORCE").is_some_and(|v| v != "0") {
            return ColorMode::Always;
        }
        if is_tty {
            ColorMode::Always
        } else {
            ColorMode::Never
        }
    }
}

/// Whether to use Unicode glyphs or ASCII fallbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphMode {
    /// Use Unicode glyphs (✓ ⚠ ✗ ℹ →).
    Unicode,
    /// Use ASCII-only fallbacks ([OK] etc.).
    Ascii,
}

impl GlyphMode {
    /// Detect glyph support. Unicode is assumed unless `NO_UNICODE` is set or the
    /// active locale clearly is not UTF-8 (`LC_ALL`/`LC_CTYPE`/`LANG`).
    pub fn detect() -> Self {
        Self::detect_with(|k| std::env::var(k).ok())
    }

    /// Injectable-env variant for tests.
    pub fn detect_with(get_env: impl Fn(&str) -> Option<String>) -> Self {
        if get_env("NO_UNICODE").is_some() {
            return GlyphMode::Ascii;
        }
        let locale = get_env("LC_ALL")
            .or_else(|| get_env("LC_CTYPE"))
            .or_else(|| get_env("LANG"));
        match locale {
            Some(l) => {
                let l = l.to_ascii_uppercase();
                if l.contains("UTF-8") || l.contains("UTF8") {
                    GlyphMode::Unicode
                } else {
                    // A set-but-non-UTF8 locale (e.g. "C", "POSIX") → ASCII.
                    GlyphMode::Ascii
                }
            }
            // Locale unset: assume a modern UTF-8 terminal.
            None => GlyphMode::Unicode,
        }
    }
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// A semantic palette plus the active color/glyph modes. Cheap to copy.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub color: ColorMode,
    pub glyph: GlyphMode,
}

impl Default for Theme {
    /// Plain, non-colored, Unicode theme — the safe default for tests and pipes.
    fn default() -> Self {
        Theme {
            color: ColorMode::Never,
            glyph: GlyphMode::Unicode,
        }
    }
}

impl Theme {
    /// Build a theme from a single `is_tty` flag, auto-detecting everything else.
    pub fn auto(is_tty: bool) -> Self {
        Theme {
            color: ColorMode::detect(is_tty),
            glyph: GlyphMode::detect(),
        }
    }

    /// A fully-disabled theme: no color, ASCII glyphs. Useful for `--no-color` or
    /// dumb terminals.
    pub fn plain() -> Self {
        Theme {
            color: ColorMode::Never,
            glyph: GlyphMode::Ascii,
        }
    }

    fn paint(self, code: &str, text: &str) -> String {
        match self.color {
            ColorMode::Always => format!("{code}{text}{}", ansi::RESET),
            ColorMode::Never => text.to_string(),
        }
    }

    /// Apply two SGR codes (e.g. bold + color) at once.
    fn paint2(self, a: &str, b: &str, text: &str) -> String {
        match self.color {
            ColorMode::Always => format!("{a}{b}{text}{}", ansi::RESET),
            ColorMode::Never => text.to_string(),
        }
    }

    // --- semantic styles -------------------------------------------------

    pub fn success(self, t: &str) -> String {
        self.paint(ansi::GREEN, t)
    }
    pub fn warn(self, t: &str) -> String {
        self.paint(ansi::YELLOW, t)
    }
    pub fn error(self, t: &str) -> String {
        self.paint(ansi::RED, t)
    }
    pub fn info(self, t: &str) -> String {
        self.paint(ansi::CYAN, t)
    }
    pub fn dim(self, t: &str) -> String {
        self.paint(ansi::DIM, t)
    }
    pub fn accent(self, t: &str) -> String {
        self.paint(ansi::MAGENTA, t)
    }
    pub fn bold(self, t: &str) -> String {
        self.paint(ansi::BOLD, t)
    }
    pub fn heading(self, t: &str) -> String {
        self.paint2(ansi::BOLD, ansi::CYAN, t)
    }
}

// ---------------------------------------------------------------------------
// Status glyphs & semantic printers
// ---------------------------------------------------------------------------

/// The five semantic kinds of status output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warn,
    Error,
    Info,
    Hint,
}

impl Status {
    /// The glyph for this status under the given glyph mode.
    pub fn glyph(self, mode: GlyphMode) -> &'static str {
        match (self, mode) {
            (Status::Ok, GlyphMode::Unicode) => "✓",
            (Status::Ok, GlyphMode::Ascii) => "[OK]",
            (Status::Warn, GlyphMode::Unicode) => "⚠",
            (Status::Warn, GlyphMode::Ascii) => "[!]",
            (Status::Error, GlyphMode::Unicode) => "✗",
            (Status::Error, GlyphMode::Ascii) => "[x]",
            (Status::Info, GlyphMode::Unicode) => "ℹ",
            (Status::Info, GlyphMode::Ascii) => "[i]",
            (Status::Hint, GlyphMode::Unicode) => "→",
            (Status::Hint, GlyphMode::Ascii) => "->",
        }
    }

    fn colorize(self, theme: Theme, text: &str) -> String {
        match self {
            Status::Ok => theme.success(text),
            Status::Warn => theme.warn(text),
            Status::Error => theme.error(text),
            Status::Info => theme.info(text),
            Status::Hint => theme.dim(text),
        }
    }
}

/// A `Theme` bound printer that renders semantic status lines.
///
/// Render methods are pure (return `String`); `*_to`/`print_*` write to a sink.
#[derive(Debug, Clone, Copy)]
pub struct Printer {
    pub theme: Theme,
    pub verbosity: Verbosity,
}

impl Default for Printer {
    fn default() -> Self {
        Printer {
            theme: Theme::default(),
            verbosity: Verbosity::Normal,
        }
    }
}

impl Printer {
    pub fn new(theme: Theme, verbosity: Verbosity) -> Self {
        Printer { theme, verbosity }
    }

    /// Render a single status line, e.g. `"✓ Build complete"`. The glyph is
    /// colorized to match the status; the message is left plain so callers can
    /// embed their own emphasis.
    pub fn render_status(&self, status: Status, msg: &str) -> String {
        let glyph = status.glyph(self.theme.glyph);
        let glyph = status.colorize(self.theme, glyph);
        format!("{glyph} {msg}")
    }

    pub fn render_ok(&self, msg: &str) -> String {
        self.render_status(Status::Ok, msg)
    }
    pub fn render_warn(&self, msg: &str) -> String {
        self.render_status(Status::Warn, msg)
    }
    pub fn render_err(&self, msg: &str) -> String {
        self.render_status(Status::Error, msg)
    }
    pub fn render_info(&self, msg: &str) -> String {
        self.render_status(Status::Info, msg)
    }
    pub fn render_hint(&self, msg: &str) -> String {
        let line = self.render_status(Status::Hint, msg);
        self.theme.dim(&line)
    }

    /// Render a `"[n/total] msg"` step line with a dimmed counter.
    pub fn render_step(&self, n: usize, total: usize, msg: &str) -> String {
        let counter = self.theme.dim(&format!("[{n}/{total}]"));
        format!("{counter} {msg}")
    }

    // --- side-effecting helpers -----------------------------------------

    /// Whether a message at `level` should be shown under the current verbosity.
    fn shows(&self, level: Verbosity) -> bool {
        self.verbosity.allows(level)
    }

    /// Write a status line to the given sink iff verbosity permits.
    /// Errors/warnings always show (except under `Quiet`, where only errors show).
    pub fn status_to<W: Write>(&self, w: &mut W, status: Status, msg: &str) -> std::io::Result<()> {
        let level = match status {
            Status::Error => Verbosity::Quiet,
            Status::Warn => Verbosity::Normal,
            Status::Ok | Status::Info => Verbosity::Normal,
            Status::Hint => Verbosity::Verbose,
        };
        if self.shows(level) {
            writeln!(w, "{}", self.render_status(status, msg))?;
        }
        Ok(())
    }

    pub fn ok(&self, msg: &str) {
        let _ = self.status_to(&mut std::io::stderr(), Status::Ok, msg);
    }
    pub fn warn(&self, msg: &str) {
        let _ = self.status_to(&mut std::io::stderr(), Status::Warn, msg);
    }
    pub fn err(&self, msg: &str) {
        let _ = self.status_to(&mut std::io::stderr(), Status::Error, msg);
    }
    pub fn info(&self, msg: &str) {
        let _ = self.status_to(&mut std::io::stderr(), Status::Info, msg);
    }
    pub fn hint(&self, msg: &str) {
        if self.shows(Verbosity::Verbose) {
            let _ = writeln!(std::io::stderr(), "{}", self.render_hint(msg));
        }
    }
    pub fn step(&self, n: usize, total: usize, msg: &str) {
        if self.shows(Verbosity::Normal) {
            let _ = writeln!(std::io::stderr(), "{}", self.render_step(n, total, msg));
        }
    }
}

// ---------------------------------------------------------------------------
// Verbosity
// ---------------------------------------------------------------------------

/// Output verbosity gate. `Quiet` shows only errors; `Debug` shows everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Verbosity {
    Quiet = 0,
    #[default]
    Normal = 1,
    Verbose = 2,
    Debug = 3,
}

impl Verbosity {
    /// Map a `-q` count and `-v` count (clap `ArgAction::Count` style) to a level.
    /// `quiet` wins if set. Each `-v` raises the level one notch above `Normal`.
    pub fn from_flags(quiet: bool, verbose: u8) -> Self {
        if quiet {
            return Verbosity::Quiet;
        }
        match verbose {
            0 => Verbosity::Normal,
            1 => Verbosity::Verbose,
            _ => Verbosity::Debug,
        }
    }

    /// Whether a message tagged at `level` should be emitted at this verbosity.
    /// A message shows when the active verbosity is at least its level.
    pub fn allows(self, level: Verbosity) -> bool {
        self >= level
    }
}

// ---------------------------------------------------------------------------
// Width / unicode helpers
// ---------------------------------------------------------------------------

/// Length of `s` in terminal columns, ignoring ANSI SGR escape sequences and
/// counting each (non-escape) `char` as width 1. This is a deliberately minimal
/// width model: it is correct for ASCII and the Latin/most-BMP text the CLI emits,
/// and it never miscounts color codes. It does not attempt full east-asian-width
/// or grapheme-cluster handling (that would need an external crate).
pub fn display_width(s: &str) -> usize {
    let mut width = 0usize;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip a CSI sequence: ESC [ ... <final byte 0x40..=0x7e>
            if chars.peek() == Some(&'[') {
                chars.next();
                for d in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&d) {
                        break;
                    }
                }
            }
            continue;
        }
        width += 1;
    }
    width
}

/// Pad `s` on the right with spaces to `target` display columns. Strings already
/// at or beyond `target` are returned unchanged.
pub fn pad_right(s: &str, target: usize) -> String {
    let w = display_width(s);
    if w >= target {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(target - w))
    }
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

/// A minimal column-aligned table renderer.
///
/// ```
/// # use rocket_sdk::ui::{Table, Theme};
/// let out = Table::new(["Name", "Status"])
///     .row(["ShooterGame", "ok"])
///     .row(["Brm", "missing"])
///     .render(&Theme::default());
/// assert!(out.contains("ShooterGame"));
/// ```
#[derive(Debug, Clone)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new<I, S>(headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Table {
            headers: headers.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
        }
    }

    /// Append a row. Rows shorter than the header are padded with empty cells;
    /// longer rows are truncated to the header width.
    pub fn row<I, S>(mut self, cells: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut cells: Vec<String> = cells.into_iter().map(Into::into).collect();
        cells.resize(self.headers.len(), String::new());
        self.rows.push(cells);
        self
    }

    /// Number of data rows (excluding the header).
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Compute the column widths from headers + all rows.
    fn col_widths(&self) -> Vec<usize> {
        let mut widths: Vec<usize> = self.headers.iter().map(|h| display_width(h)).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(display_width(cell));
                }
            }
        }
        widths
    }

    /// Render the table to a `String` with a `theme`. Headers are bolded (if color
    /// is enabled) and underlined with a separator row of dashes.
    pub fn render(&self, theme: &Theme) -> String {
        let widths = self.col_widths();
        let mut out = String::new();

        // header
        let header_line = self.render_row(&self.headers, &widths, |c| theme.bold(c));
        out.push_str(&header_line);
        out.push('\n');

        // separator
        let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
        out.push_str(&self.render_row(&sep, &widths, |c| theme.dim(c)));
        out.push('\n');

        // rows
        for (ri, row) in self.rows.iter().enumerate() {
            out.push_str(&self.render_row(row, &widths, |c| c.to_string()));
            if ri + 1 < self.rows.len() {
                out.push('\n');
            }
        }
        out
    }

    /// Render one row: pad each cell to its column width (padding uses the raw
    /// cell so alignment stays correct), then apply `style` to the padded cell.
    fn render_row(
        &self,
        cells: &[String],
        widths: &[usize],
        style: impl Fn(&str) -> String,
    ) -> String {
        let mut parts = Vec::with_capacity(cells.len());
        for (i, cell) in cells.iter().enumerate() {
            let w = widths.get(i).copied().unwrap_or_else(|| display_width(cell));
            let padded = pad_right(cell, w);
            parts.push(style(&padded));
        }
        parts.join("  ")
    }
}

// ---------------------------------------------------------------------------
// Progress: bar, spinner, timer
// ---------------------------------------------------------------------------

/// A text progress bar. Rendering is a pure function so it can be snapshot-tested;
/// the animated stderr variant is a thin wrapper ([`ProgressBar::draw_to`]).
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub total: u64,
    pub current: u64,
    /// Width of the bar body in characters (excluding brackets).
    pub width: usize,
    pub glyph: GlyphMode,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        ProgressBar {
            total,
            current: 0,
            width: 30,
            glyph: GlyphMode::Unicode,
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width.max(1);
        self
    }

    pub fn with_glyph(mut self, glyph: GlyphMode) -> Self {
        self.glyph = glyph;
        self
    }

    pub fn set(&mut self, current: u64) {
        self.current = current.min(self.total);
    }

    pub fn inc(&mut self, by: u64) {
        self.set(self.current.saturating_add(by));
    }

    /// Fraction complete in `0.0..=1.0`. A zero-total bar is treated as complete.
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            (self.current as f64 / self.total as f64).clamp(0.0, 1.0)
        }
    }

    /// Render the bar, e.g. `"[██████----------] 40% (4/10)"`.
    pub fn render(&self) -> String {
        let (full, empty) = match self.glyph {
            GlyphMode::Unicode => ('█', '░'),
            GlyphMode::Ascii => ('#', '-'),
        };
        let filled = (self.fraction() * self.width as f64).round() as usize;
        let filled = filled.min(self.width);
        let mut bar = String::with_capacity(self.width);
        for _ in 0..filled {
            bar.push(full);
        }
        for _ in filled..self.width {
            bar.push(empty);
        }
        let pct = (self.fraction() * 100.0).round() as u64;
        format!("[{bar}] {pct}% ({}/{})", self.current, self.total)
    }

    /// Draw the bar to a sink on a single, carriage-return-prefixed line so it
    /// overwrites itself in place. Caller is responsible for the trailing newline
    /// when finished.
    pub fn draw_to<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        write!(w, "\r{}", self.render())?;
        w.flush()
    }
}

/// A spinner whose frame is a pure function of a tick counter.
#[derive(Debug, Clone)]
pub struct Spinner {
    frames: Vec<&'static str>,
    pub tick: usize,
}

impl Spinner {
    /// Braille spinner for UTF-8 terminals.
    pub fn unicode() -> Self {
        Spinner {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            tick: 0,
        }
    }

    /// ASCII spinner fallback.
    pub fn ascii() -> Self {
        Spinner {
            frames: vec!["|", "/", "-", "\\"],
            tick: 0,
        }
    }

    /// Build the appropriate spinner for a glyph mode.
    pub fn for_mode(mode: GlyphMode) -> Self {
        match mode {
            GlyphMode::Unicode => Self::unicode(),
            GlyphMode::Ascii => Self::ascii(),
        }
    }

    /// The current frame glyph.
    pub fn frame(&self) -> &'static str {
        self.frames[self.tick % self.frames.len()]
    }

    /// Advance the spinner one frame.
    pub fn advance(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// Render `"<frame> <msg>"`.
    pub fn render(&self, msg: &str) -> String {
        format!("{} {msg}", self.frame())
    }

    /// Draw the spinner to a sink in place (carriage-return prefixed) and advance.
    pub fn draw_to<W: Write>(&mut self, w: &mut W, msg: &str) -> std::io::Result<()> {
        write!(w, "\r{}", self.render(msg))?;
        w.flush()?;
        self.advance();
        Ok(())
    }
}

/// A wall-clock timer for "took 1.2s" style suffixes.
#[derive(Debug, Clone, Copy)]
pub struct Timer {
    start: Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Self::start()
    }
}

impl Timer {
    pub fn start() -> Self {
        Timer {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Human-friendly elapsed string for the current elapsed time.
    pub fn human(&self) -> String {
        format_duration(self.elapsed())
    }
}

/// Format a `Duration` as a compact human string: `"850ms"`, `"1.2s"`, `"1m03s"`.
pub fn format_duration(d: Duration) -> String {
    let total_ms = d.as_millis();
    if total_ms < 1000 {
        return format!("{total_ms}ms");
    }
    let secs = d.as_secs();
    if secs < 60 {
        let frac = d.subsec_millis() / 100;
        return format!("{secs}.{frac}s");
    }
    let mins = secs / 60;
    let rem = secs % 60;
    format!("{mins}m{rem:02}s")
}

// ---------------------------------------------------------------------------
// Structured output envelope (Human vs JSON)
// ---------------------------------------------------------------------------

/// Output rendering mode for a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    /// Human-readable, themed text.
    #[default]
    Human,
    /// Machine-readable JSON envelope.
    Json,
}

impl OutputMode {
    /// `OutputMode::Json` if `json` is true, else `Human`.
    pub fn from_flag(json: bool) -> Self {
        if json {
            OutputMode::Json
        } else {
            OutputMode::Human
        }
    }
}

/// The output driver: pairs an [`OutputMode`] with a [`Printer`] for human mode.
#[derive(Debug, Clone, Copy)]
pub struct Output {
    pub mode: OutputMode,
    pub printer: Printer,
}

impl Default for Output {
    fn default() -> Self {
        Output {
            mode: OutputMode::Human,
            printer: Printer::default(),
        }
    }
}

impl Output {
    pub fn new(mode: OutputMode, printer: Printer) -> Self {
        Output { mode, printer }
    }

    /// Render the result of a command into a single `String`.
    ///
    /// In `Human` mode, calls `human_render` and returns its text. In `Json` mode,
    /// serializes `value` into a stable envelope:
    /// `{"ok": true, "command": "<command>", "data": <value>}`.
    ///
    /// `value` is `&impl Serialize` — `serde`/`serde_json` are already first-party
    /// deps of this crate, so the JSON path needs no new dependency.
    pub fn render<T, F>(&self, command: &str, value: &T, human_render: F) -> String
    where
        T: serde::Serialize,
        F: FnOnce() -> String,
    {
        match self.mode {
            OutputMode::Human => human_render(),
            OutputMode::Json => {
                let envelope = serde_json::json!({
                    "ok": true,
                    "command": command,
                    "data": value,
                });
                serde_json::to_string_pretty(&envelope)
                    .unwrap_or_else(|e| format!("{{\"ok\":false,\"error\":\"{e}\"}}"))
            }
        }
    }

    /// Render an *error* result. Human mode produces a themed error line; JSON mode
    /// produces `{"ok": false, "command": "<command>", "error": "<message>"}`.
    pub fn render_error(&self, command: &str, message: &str) -> String {
        match self.mode {
            OutputMode::Human => self.printer.render_err(message),
            OutputMode::Json => {
                let envelope = serde_json::json!({
                    "ok": false,
                    "command": command,
                    "error": message,
                });
                serde_json::to_string_pretty(&envelope)
                    .unwrap_or_else(|_| String::from("{\"ok\":false}"))
            }
        }
    }

    /// Convenience: render to a sink, with trailing newline.
    pub fn emit_to<W, T, F>(
        &self,
        w: &mut W,
        command: &str,
        value: &T,
        human_render: F,
    ) -> std::io::Result<()>
    where
        W: Write,
        T: serde::Serialize,
        F: FnOnce() -> String,
    {
        writeln!(w, "{}", self.render(command, value, human_render))
    }

    /// Convenience: print a success result to stdout.
    pub fn emit<T, F>(&self, command: &str, value: &T, human_render: F)
    where
        T: serde::Serialize,
        F: FnOnce() -> String,
    {
        let _ = self.emit_to(&mut std::io::stdout(), command, value, human_render);
    }
}

// ---------------------------------------------------------------------------
// A small display helper so callers can `theme.success(x).to_string()` style.
// ---------------------------------------------------------------------------

/// Wraps a styled string so it can participate in `format!`/`Display`.
pub struct Styled(pub String);

impl fmt::Display for Styled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn env_map<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| v.to_string())
        }
    }

    // --- color detection -------------------------------------------------

    #[test]
    fn no_color_forces_never_even_with_tty() {
        let env = env_map(&[("NO_COLOR", "1")]);
        assert_eq!(ColorMode::detect_with(true, &env), ColorMode::Never);
    }

    #[test]
    fn no_color_empty_value_still_disables() {
        // The convention is "present, regardless of value".
        let env = env_map(&[("NO_COLOR", "")]);
        assert_eq!(ColorMode::detect_with(true, &env), ColorMode::Never);
    }

    #[test]
    fn clicolor_force_enables_without_tty() {
        let env = env_map(&[("CLICOLOR_FORCE", "1")]);
        assert_eq!(ColorMode::detect_with(false, &env), ColorMode::Always);
    }

    #[test]
    fn clicolor_force_zero_is_ignored() {
        let env = env_map(&[("CLICOLOR_FORCE", "0")]);
        assert_eq!(ColorMode::detect_with(false, &env), ColorMode::Never);
    }

    #[test]
    fn no_color_beats_clicolor_force() {
        let env = env_map(&[("NO_COLOR", "1"), ("CLICOLOR_FORCE", "1")]);
        assert_eq!(ColorMode::detect_with(true, &env), ColorMode::Never);
    }

    #[test]
    fn tty_alone_enables_color() {
        let env = env_map(&[]);
        assert_eq!(ColorMode::detect_with(true, &env), ColorMode::Always);
        assert_eq!(ColorMode::detect_with(false, &env), ColorMode::Never);
    }

    // --- glyph detection -------------------------------------------------

    #[test]
    fn utf8_locale_gives_unicode() {
        let env = env_map(&[("LANG", "en_US.UTF-8")]);
        assert_eq!(GlyphMode::detect_with(&env), GlyphMode::Unicode);
    }

    #[test]
    fn c_locale_gives_ascii() {
        let env = env_map(&[("LANG", "C")]);
        assert_eq!(GlyphMode::detect_with(&env), GlyphMode::Ascii);
    }

    #[test]
    fn no_unicode_env_forces_ascii() {
        let env = env_map(&[("NO_UNICODE", "1"), ("LANG", "en_US.UTF-8")]);
        assert_eq!(GlyphMode::detect_with(&env), GlyphMode::Ascii);
    }

    #[test]
    fn unset_locale_defaults_unicode() {
        let env = env_map(&[]);
        assert_eq!(GlyphMode::detect_with(&env), GlyphMode::Unicode);
    }

    // --- glyph fallback --------------------------------------------------

    #[test]
    fn glyphs_have_ascii_fallbacks() {
        assert_eq!(Status::Ok.glyph(GlyphMode::Unicode), "✓");
        assert_eq!(Status::Ok.glyph(GlyphMode::Ascii), "[OK]");
        assert_eq!(Status::Error.glyph(GlyphMode::Unicode), "✗");
        assert_eq!(Status::Error.glyph(GlyphMode::Ascii), "[x]");
        assert_eq!(Status::Hint.glyph(GlyphMode::Unicode), "→");
        assert_eq!(Status::Hint.glyph(GlyphMode::Ascii), "->");
    }

    // --- theme / painting ------------------------------------------------

    #[test]
    fn never_mode_emits_no_escapes() {
        let t = Theme {
            color: ColorMode::Never,
            glyph: GlyphMode::Unicode,
        };
        let s = t.success("hello");
        assert_eq!(s, "hello");
        assert!(!s.contains('\x1b'));
    }

    #[test]
    fn always_mode_wraps_in_escapes_and_resets() {
        let t = Theme {
            color: ColorMode::Always,
            glyph: GlyphMode::Unicode,
        };
        let s = t.error("boom");
        assert!(s.starts_with(ansi::RED));
        assert!(s.ends_with(ansi::RESET));
        assert!(s.contains("boom"));
    }

    // --- printers --------------------------------------------------------

    #[test]
    fn render_status_plain() {
        let p = Printer::default(); // no color, unicode
        assert_eq!(p.render_ok("done"), "✓ done");
        assert_eq!(p.render_err("nope"), "✗ nope");
    }

    #[test]
    fn render_status_ascii() {
        let p = Printer::new(Theme::plain(), Verbosity::Normal);
        assert_eq!(p.render_ok("done"), "[OK] done");
    }

    #[test]
    fn render_step_counts() {
        let p = Printer::default();
        assert_eq!(p.render_step(2, 5, "compiling"), "[2/5] compiling");
    }

    #[test]
    fn status_to_respects_quiet() {
        let p = Printer::new(Theme::default(), Verbosity::Quiet);
        let mut buf: Vec<u8> = Vec::new();
        p.status_to(&mut buf, Status::Ok, "hidden").unwrap();
        p.status_to(&mut buf, Status::Error, "shown").unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(!out.contains("hidden"));
        assert!(out.contains("shown"));
    }

    // --- verbosity -------------------------------------------------------

    #[test]
    fn verbosity_ordering() {
        assert!(Verbosity::Debug > Verbosity::Verbose);
        assert!(Verbosity::Normal.allows(Verbosity::Quiet));
        assert!(!Verbosity::Quiet.allows(Verbosity::Normal));
        assert!(Verbosity::Verbose.allows(Verbosity::Normal));
    }

    #[test]
    fn verbosity_from_flags() {
        assert_eq!(Verbosity::from_flags(true, 5), Verbosity::Quiet);
        assert_eq!(Verbosity::from_flags(false, 0), Verbosity::Normal);
        assert_eq!(Verbosity::from_flags(false, 1), Verbosity::Verbose);
        assert_eq!(Verbosity::from_flags(false, 9), Verbosity::Debug);
    }

    // --- width helpers ---------------------------------------------------

    #[test]
    fn display_width_ignores_ansi() {
        let colored = format!("{}{}{}", ansi::RED, "abc", ansi::RESET);
        assert_eq!(display_width(&colored), 3);
        assert_eq!(display_width("abc"), 3);
    }

    #[test]
    fn pad_right_pads_and_preserves() {
        assert_eq!(pad_right("ab", 5), "ab   ");
        assert_eq!(pad_right("abcde", 3), "abcde");
        // padding measures display width, ignoring color
        let colored = format!("{}{}{}", ansi::GREEN, "ab", ansi::RESET);
        let padded = pad_right(&colored, 5);
        assert_eq!(display_width(&padded), 5);
    }

    // --- tables ----------------------------------------------------------

    #[test]
    fn table_aligns_columns() {
        let t = Table::new(["Name", "Status"])
            .row(["ShooterGame", "ok"])
            .row(["Brm", "missing"]);
        let out = t.render(&Theme::default());
        let lines: Vec<&str> = out.lines().collect();
        // header + separator + 2 rows
        assert_eq!(lines.len(), 4);
        // "Name" column padded to width of "ShooterGame" (11)
        assert!(lines[0].starts_with("Name       "));
        // separator row uses dashes matching column width
        assert!(lines[1].starts_with("-----------"));
        // each rendered line has identical display width
        let w0 = display_width(lines[0]);
        for l in &lines {
            assert_eq!(display_width(l), w0, "line not aligned: {l:?}");
        }
    }

    #[test]
    fn table_handles_short_and_long_rows() {
        let t = Table::new(["A", "B", "C"])
            .row(["1"]) // short → padded with empties
            .row(["x", "y", "z", "extra"]); // long → truncated
        assert_eq!(t.len(), 2);
        let out = t.render(&Theme::default());
        assert!(!out.contains("extra"));
        assert!(out.contains('1'));
    }

    #[test]
    fn empty_table_renders_header_and_separator() {
        let t = Table::new(["X", "Y"]);
        assert!(t.is_empty());
        let out = t.render(&Theme::default());
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    // --- progress bar ----------------------------------------------------

    #[test]
    fn progress_bar_zero_and_full() {
        let mut pb = ProgressBar::new(10).with_width(10).with_glyph(GlyphMode::Ascii);
        assert_eq!(pb.render(), "[----------] 0% (0/10)");
        pb.set(10);
        assert_eq!(pb.render(), "[##########] 100% (10/10)");
    }

    #[test]
    fn progress_bar_midpoint() {
        let mut pb = ProgressBar::new(10).with_width(10).with_glyph(GlyphMode::Ascii);
        pb.set(5);
        assert_eq!(pb.render(), "[#####-----] 50% (5/10)");
    }

    #[test]
    fn progress_bar_clamps_overflow() {
        let mut pb = ProgressBar::new(4).with_width(8).with_glyph(GlyphMode::Ascii);
        pb.inc(100);
        assert_eq!(pb.fraction(), 1.0);
        assert!(pb.render().contains("(4/4)"));
    }

    #[test]
    fn progress_bar_zero_total_is_complete() {
        let pb = ProgressBar::new(0).with_glyph(GlyphMode::Ascii);
        assert_eq!(pb.fraction(), 1.0);
        assert!(pb.render().contains("100%"));
    }

    #[test]
    fn progress_bar_unicode_glyphs() {
        let mut pb = ProgressBar::new(2).with_width(2).with_glyph(GlyphMode::Unicode);
        pb.set(1);
        assert_eq!(pb.render(), "[█░] 50% (1/2)");
    }

    #[test]
    fn progress_bar_draw_uses_carriage_return() {
        let pb = ProgressBar::new(2).with_width(2).with_glyph(GlyphMode::Ascii);
        let mut buf: Vec<u8> = Vec::new();
        pb.draw_to(&mut buf).unwrap();
        assert!(String::from_utf8(buf).unwrap().starts_with('\r'));
    }

    // --- spinner ---------------------------------------------------------

    #[test]
    fn spinner_cycles_frames() {
        let mut s = Spinner::ascii();
        assert_eq!(s.frame(), "|");
        s.advance();
        assert_eq!(s.frame(), "/");
        s.advance();
        s.advance();
        s.advance();
        // wrapped back around
        assert_eq!(s.frame(), "|");
    }

    #[test]
    fn spinner_render_includes_message() {
        let s = Spinner::ascii();
        assert_eq!(s.render("loading"), "| loading");
    }

    #[test]
    fn spinner_for_mode() {
        assert_eq!(Spinner::for_mode(GlyphMode::Ascii).frame(), "|");
        assert_eq!(Spinner::for_mode(GlyphMode::Unicode).frame(), "⠋");
    }

    // --- timer / duration ------------------------------------------------

    #[test]
    fn duration_formatting() {
        assert_eq!(format_duration(Duration::from_millis(850)), "850ms");
        assert_eq!(format_duration(Duration::from_millis(1200)), "1.2s");
        assert_eq!(format_duration(Duration::from_secs(63)), "1m03s");
    }

    #[test]
    fn timer_runs() {
        let t = Timer::start();
        assert!(t.elapsed() < Duration::from_secs(5));
        assert!(!t.human().is_empty());
    }

    // --- output envelope -------------------------------------------------

    #[derive(serde::Serialize)]
    struct DemoData {
        count: u32,
        name: String,
    }

    #[test]
    fn output_human_uses_render_fn() {
        let out = Output::default();
        let data = DemoData {
            count: 3,
            name: "x".into(),
        };
        let rendered = out.render("info", &data, || "human text".to_string());
        assert_eq!(rendered, "human text");
    }

    #[test]
    fn output_json_envelope_shape() {
        let out = Output::new(OutputMode::Json, Printer::default());
        let data = DemoData {
            count: 3,
            name: "shooter".into(),
        };
        let rendered = out.render("info", &data, || unreachable!("human not called"));
        let v: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(v["ok"], serde_json::json!(true));
        assert_eq!(v["command"], serde_json::json!("info"));
        assert_eq!(v["data"]["count"], serde_json::json!(3));
        assert_eq!(v["data"]["name"], serde_json::json!("shooter"));
    }

    #[test]
    fn output_json_error_shape() {
        let out = Output::new(OutputMode::Json, Printer::default());
        let rendered = out.render_error("build", "no target");
        let v: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(v["ok"], serde_json::json!(false));
        assert_eq!(v["command"], serde_json::json!("build"));
        assert_eq!(v["error"], serde_json::json!("no target"));
    }

    #[test]
    fn output_mode_from_flag() {
        assert_eq!(OutputMode::from_flag(true), OutputMode::Json);
        assert_eq!(OutputMode::from_flag(false), OutputMode::Human);
    }

    #[test]
    fn emit_to_writes_newline() {
        let out = Output::default();
        let data = DemoData {
            count: 1,
            name: "a".into(),
        };
        let mut buf: Vec<u8> = Vec::new();
        out.emit_to(&mut buf, "info", &data, || "line".to_string())
            .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "line\n");
    }
}
