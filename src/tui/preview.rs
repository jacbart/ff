//! Preview pane: external command rendering with ANSI color support.

use crate::tui::buffer::ScreenBuffer;
use crossterm::style::Color;
use std::collections::HashMap;

/// Sentinel value for the smart auto-preview rule
const AUTO_SENTINEL: &str = "__auto__";

/// Preview rule: command template + optional extension filter
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewRule {
    /// Command template ({} replaced with item path)
    pub cmd: String,
    /// File extensions this rule applies to (empty = default for all)
    pub exts: Vec<String>,
}

impl PreviewRule {
    /// Parse a preview rule string.
    ///
    /// Syntax:
    /// - `"bat"` → default rule (no braces)
    /// - `"bat {rs,toml}"` → rule for .rs and .toml
    /// - `"bat {}"` → explicit default rule
    /// - `"auto"` → smart auto-preview rule
    pub fn parse(s: &str) -> Result<Self, String> {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("auto") {
            return Ok(Self {
                cmd: AUTO_SENTINEL.to_string(),
                exts: vec![],
            });
        }
        if let Some(brace_start) = s.rfind('{') {
            let cmd = s[..brace_start].trim().to_string();
            let brace_content = &s[brace_start + 1..];
            if !brace_content.ends_with('}') {
                return Err("Missing closing brace in preview rule".to_string());
            }
            let inner = &brace_content[..brace_content.len() - 1];
            let exts: Vec<String> = if inner.is_empty() {
                vec![]
            } else {
                inner.split(',').map(|e| e.trim().to_lowercase()).collect()
            };
            Ok(Self { cmd, exts })
        } else {
            // No braces: treat as default rule
            Ok(Self {
                cmd: trimmed.to_string(),
                exts: vec![],
            })
        }
    }

    /// Check if this rule matches a given file extension (case-insensitive)
    pub fn matches_ext(&self, ext: &str) -> bool {
        let ext_lower = ext.to_lowercase();
        self.exts.contains(&ext_lower)
    }
}

/// Result of running a preview command
#[derive(Debug, Clone)]
pub enum PreviewResult {
    /// Parsed output lines
    Success(Vec<StyledLine>),
    /// Command failed
    Error(String),
}

/// A finished preview run, tagged with the item it describes.
///
/// Results arrive asynchronously and can outlive the cursor position that
/// asked for them, so they carry their item and are dropped when stale.
#[derive(Debug, Clone)]
pub struct PreviewUpdate {
    /// Item the preview was rendered for
    pub item: String,
    /// Result of the command
    pub result: PreviewResult,
}

/// How long a preview command may run before it is killed.
///
/// The pane is cosmetic: a command that hangs (a FIFO, a network path, a
/// runaway process) must never pin a worker or block cursor movement.
pub const PREVIEW_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// A line of styled text segments
pub type StyledLine = Vec<(String, Option<Color>, Option<Color>, bool, bool)>;

/// Preview pane state
#[derive(Debug, Clone)]
pub struct PreviewState {
    /// Whether preview pane is visible
    pub visible: bool,
    /// Whether preview pane has keyboard focus
    pub focused: bool,
    /// Parsed output lines
    pub lines: Vec<StyledLine>,
    /// Scroll offset
    pub scroll: usize,
    /// Cache: item text → parsed lines
    pub cache: HashMap<String, Vec<StyledLine>>,
    /// Currently previewed item
    pub current_item: String,
    /// Waiting for command output
    pub loading: bool,
    /// Error message if command failed
    pub error: Option<String>,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewState {
    pub fn new() -> Self {
        Self {
            visible: false,
            focused: false,
            lines: Vec::new(),
            scroll: 0,
            cache: HashMap::new(),
            current_item: String::new(),
            loading: false,
            error: None,
        }
    }

    /// Toggle visibility
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
        if !self.visible {
            self.focused = false;
        }
    }

    /// Start loading a new item
    pub fn start_loading(&mut self, item: &str) {
        self.current_item = item.to_string();
        self.loading = true;
        self.error = None;
        self.scroll = 0;
        if let Some(cached) = self.cache.get(item) {
            self.lines = cached.clone();
            self.loading = false;
        }
    }

    /// Apply a finished preview run.
    ///
    /// Results for an item that is no longer selected are dropped: a slow
    /// command must not overwrite the pane after the cursor has moved on.
    pub fn apply_result(&mut self, update: PreviewUpdate) {
        if update.item != self.current_item {
            return;
        }
        self.loading = false;
        match update.result {
            PreviewResult::Success(lines) => {
                self.lines = lines.clone();
                self.cache.insert(self.current_item.clone(), lines);
                self.error = None;
            }
            PreviewResult::Error(msg) => {
                self.error = Some(msg);
            }
        }
    }

    /// Scroll up
    pub fn scroll_up(&mut self, delta: usize) {
        self.scroll = self.scroll.saturating_sub(delta);
    }

    /// Scroll down
    pub fn scroll_down(&mut self, delta: usize, max_lines: usize) {
        let max_scroll = max_lines.saturating_sub(1);
        self.scroll = (self.scroll + delta).min(max_scroll);
    }

    /// Scroll to top
    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }

    /// Scroll to bottom
    pub fn scroll_bottom(&mut self, max_lines: usize) {
        self.scroll = max_lines.saturating_sub(1);
    }
}

/// Auto-inject --color=always for known tools if not already present
pub fn inject_color_flag(cmd: &str) -> String {
    let trimmed = cmd.trim();
    let first_word = trimmed.split_whitespace().next().unwrap_or("");
    let needs_color = matches!(first_word, "bat" | "batcat" | "eza" | "exa");
    if needs_color && !trimmed.contains("--color=always") && !trimmed.contains("--color always") {
        // Insert --color=always after the command name
        if let Some(pos) = trimmed.find(' ') {
            format!("{} --color=always{}", &trimmed[..pos], &trimmed[pos..])
        } else {
            format!("{} --color=always", trimmed)
        }
    } else {
        cmd.to_string()
    }
}

/// Handle to a running preview command.
///
/// Dropping it cancels the run: the worker notices within a few milliseconds,
/// kills the child process, and does not send a result. This exists because
/// `JoinHandle::abort` cannot stop a `spawn_blocking` task, which is how the
/// preview used to be run — every cursor move could orphan a `sh -c` child.
pub struct PreviewTask {
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: tokio::task::JoinHandle<()>,
}

impl PreviewTask {
    /// Ask the worker to stop. Equivalent to dropping the handle, but leaves the
    /// handle usable for bookkeeping.
    pub fn cancel(&self) {
        self.cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Drop for PreviewTask {
    fn drop(&mut self) {
        self.cancel();
        self.handle.abort();
    }
}

/// Run a preview command and send the result back, tagged with its item.
///
/// The command is killed when the returned handle is dropped, or when
/// [`PREVIEW_TIMEOUT`] elapses.
pub fn spawn_preview_task(
    item: String,
    command: String,
    sender: std::sync::mpsc::Sender<PreviewUpdate>,
) -> PreviewTask {
    let command = inject_color_flag(&command);
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let worker_cancel = std::sync::Arc::clone(&cancel);

    let handle = tokio::task::spawn_blocking(move || {
        if let Some(result) = run_preview_command(&command, PREVIEW_TIMEOUT, &worker_cancel) {
            let _ = sender.send(PreviewUpdate { item, result });
        }
    });

    PreviewTask { cancel, handle }
}

/// Execute `command` through the platform shell.
///
/// Returns `None` when the run was cancelled. The child is polled every few
/// milliseconds so cancellation and the timeout are both honoured while its
/// output is being drained by reader threads (a full pipe would otherwise
/// deadlock the wait).
pub(crate) fn run_preview_command(
    command: &str,
    timeout: std::time::Duration,
    cancel: &std::sync::atomic::AtomicBool,
) -> Option<PreviewResult> {
    let child = if cfg!(target_os = "windows") {
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/C");
        cmd
    } else {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c");
        cmd
    }
    .arg(command)
    .stdin(std::process::Stdio::null())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(err) => return Some(PreviewResult::Error(err.to_string())),
    };

    let stdout_reader = child.stdout.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::Read::read_to_end(&mut pipe, &mut buf);
            buf
        })
    });
    let stderr_reader = child.stderr.take().map(|mut pipe| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::Read::read_to_end(&mut pipe, &mut buf);
            buf
        })
    });

    let deadline = std::time::Instant::now() + timeout;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {}
            Err(err) => {
                let _ = child.kill();
                return Some(PreviewResult::Error(err.to_string()));
            }
        }

        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = child.kill();
            // Reap the child so cancellation cannot leave zombies behind.
            let _ = child.wait();
            // Reader threads are not joined: a grandchild may still hold the pipes.
            return None;
        }

        if std::time::Instant::now() >= deadline {
            timed_out = true;
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    };

    if timed_out {
        return Some(PreviewResult::Error(format!(
            "preview command timed out after {}s",
            timeout.as_secs()
        )));
    }

    let stdout = stdout_reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    let stderr = stderr_reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();

    match status {
        Some(status) if status.success() => Some(PreviewResult::Success(parse_ansi_output(
            &String::from_utf8_lossy(&stdout),
        ))),
        Some(_) => Some(PreviewResult::Error(
            String::from_utf8_lossy(&stderr).to_string(),
        )),
        None => Some(PreviewResult::Error(
            "preview command produced no exit status".to_string(),
        )),
    }
}

/// Parse ANSI-encoded text into styled lines
pub fn parse_ansi_output(text: &str) -> Vec<StyledLine> {
    let mut lines = Vec::new();
    let mut current_line: StyledLine = Vec::new();
    let mut current_text = String::new();
    let mut fg: Option<Color> = None;
    let mut bg: Option<Color> = None;
    let mut bold = false;
    let mut underline = false;

    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Flush current segment
            if !current_text.is_empty() {
                current_line.push((std::mem::take(&mut current_text), fg, bg, bold, underline));
            }
            // Parse escape sequence
            if chars.next() == Some('[') {
                let mut params = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == ';' {
                        params.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Always consume the final char of the CSI sequence
                if chars.next() == Some('m') {
                    apply_sgr(&params, &mut fg, &mut bg, &mut bold, &mut underline);
                }
                // else: non-SGR CSI sequence — skip silently
            }
            continue;
        }

        if ch == '\n' {
            if !current_text.is_empty() {
                current_line.push((std::mem::take(&mut current_text), fg, bg, bold, underline));
            }
            lines.push(std::mem::take(&mut current_line));
        } else if ch == '\r' {
            // Ignore carriage return
        } else {
            current_text.push(ch);
        }
    }

    // Flush remaining
    if !current_text.is_empty() {
        current_line.push((current_text, fg, bg, bold, underline));
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

fn apply_sgr(
    params: &str,
    fg: &mut Option<Color>,
    bg: &mut Option<Color>,
    bold: &mut bool,
    underline: &mut bool,
) {
    let nums: Vec<u16> = params.split(';').filter_map(|s| s.parse().ok()).collect();

    if nums.is_empty() {
        // Empty params = reset
        *fg = None;
        *bg = None;
        *bold = false;
        *underline = false;
        return;
    }

    let mut i = 0;
    while i < nums.len() {
        match nums[i] {
            0 => {
                *fg = None;
                *bg = None;
                *bold = false;
                *underline = false;
            }
            1 => *bold = true,
            4 => *underline = true,
            22 => *bold = false,
            24 => *underline = false,
            30 => *fg = Some(Color::Black),
            31 => *fg = Some(Color::Red),
            32 => *fg = Some(Color::Green),
            33 => *fg = Some(Color::Yellow),
            34 => *fg = Some(Color::Blue),
            35 => *fg = Some(Color::Magenta),
            36 => *fg = Some(Color::Cyan),
            37 => *fg = Some(Color::White),
            39 => *fg = None,
            40 => *bg = Some(Color::Black),
            41 => *bg = Some(Color::Red),
            42 => *bg = Some(Color::Green),
            43 => *bg = Some(Color::Yellow),
            44 => *bg = Some(Color::Blue),
            45 => *bg = Some(Color::Magenta),
            46 => *bg = Some(Color::Cyan),
            47 => *bg = Some(Color::White),
            49 => *bg = None,
            90 => *fg = Some(Color::DarkGrey),
            91 => *fg = Some(Color::Red),
            92 => *fg = Some(Color::Green),
            93 => *fg = Some(Color::Yellow),
            94 => *fg = Some(Color::Blue),
            95 => *fg = Some(Color::Magenta),
            96 => *fg = Some(Color::Cyan),
            97 => *fg = Some(Color::White),
            100 => *bg = Some(Color::DarkGrey),
            101 => *bg = Some(Color::Red),
            102 => *bg = Some(Color::Green),
            103 => *bg = Some(Color::Yellow),
            104 => *bg = Some(Color::Blue),
            105 => *bg = Some(Color::Magenta),
            106 => *bg = Some(Color::Cyan),
            107 => *bg = Some(Color::White),
            38 if i + 2 < nums.len() && nums[i + 1] == 5 => {
                // 256-color fg
                if let Some(c) = ansi_256_to_color(nums[i + 2]) {
                    *fg = Some(c);
                }
                i += 2;
            }
            48 if i + 2 < nums.len() && nums[i + 1] == 5 => {
                // 256-color bg
                if let Some(c) = ansi_256_to_color(nums[i + 2]) {
                    *bg = Some(c);
                }
                i += 2;
            }
            38 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                // Truecolor fg
                *fg = Some(Color::Rgb {
                    r: nums[i + 2] as u8,
                    g: nums[i + 3] as u8,
                    b: nums[i + 4] as u8,
                });
                i += 4;
            }
            48 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                // Truecolor bg
                *bg = Some(Color::Rgb {
                    r: nums[i + 2] as u8,
                    g: nums[i + 3] as u8,
                    b: nums[i + 4] as u8,
                });
                i += 4;
            }
            _ => {}
        }
        i += 1;
    }
}

fn ansi_256_to_color(code: u16) -> Option<Color> {
    match code {
        0 => Some(Color::Black),
        1 => Some(Color::DarkRed),
        2 => Some(Color::DarkGreen),
        3 => Some(Color::DarkYellow),
        4 => Some(Color::DarkBlue),
        5 => Some(Color::DarkMagenta),
        6 => Some(Color::DarkCyan),
        7 => Some(Color::Grey),
        8 => Some(Color::DarkGrey),
        9 => Some(Color::Red),
        10 => Some(Color::Green),
        11 => Some(Color::Yellow),
        12 => Some(Color::Blue),
        13 => Some(Color::Magenta),
        14 => Some(Color::Cyan),
        15 => Some(Color::White),
        16..=231 => {
            // 6x6x6 color cube
            let n = code - 16;
            let r = (n / 36) as u8;
            let g = ((n % 36) / 6) as u8;
            let b = (n % 6) as u8;
            Some(Color::Rgb {
                r: if r == 0 { 0 } else { r * 40 + 55 },
                g: if g == 0 { 0 } else { g * 40 + 55 },
                b: if b == 0 { 0 } else { b * 40 + 55 },
            })
        }
        232..=255 => {
            // Grayscale
            let level = ((code - 232) as u8) * 10 + 8;
            Some(Color::Rgb {
                r: level,
                g: level,
                b: level,
            })
        }
        _ => None,
    }
}

/// Render styled lines into a screen buffer region
#[allow(clippy::too_many_arguments)]
pub fn render_preview_to_buffer(
    buffer: &mut ScreenBuffer,
    lines: &[StyledLine],
    scroll: usize,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    loading: bool,
    error: Option<&str>,
) {
    if loading {
        let msg = "Loading...";
        buffer.put_str(x, y, msg, Some(Color::DarkGrey), None, false, false);
        return;
    }

    if let Some(err) = error {
        let msg = if err.len() > width as usize {
            &err[..width as usize]
        } else {
            err
        };
        let color = if msg == "(not a file)" {
            Some(Color::DarkGrey)
        } else {
            Some(Color::Red)
        };
        buffer.put_str(x, y, msg, color, None, false, false);
        return;
    }

    if lines.is_empty() {
        buffer.put_str(x, y, "(empty)", Some(Color::DarkGrey), None, false, false);
        return;
    }

    let visible_lines = lines.iter().skip(scroll).take(height as usize);
    for (row_offset, line) in visible_lines.enumerate() {
        let row = y + row_offset as u16;
        if row >= y + height {
            break;
        }
        let mut col = x;
        for (text, fg, bg, bold, underline) in line {
            if col >= x + width {
                break;
            }
            let written = buffer.put_str(col, row, text, *fg, *bg, *bold, *underline);
            col += written;
        }
    }
}

/// Strip ANSI escape sequences from a string
pub fn strip_ansi_sequences(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Skip escape sequence
            if chars.next() == Some('[') {
                // CSI sequence: skip until letter or @~_`
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() || matches!(next, '@' | '~' | '_' | '`') {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Build a smart preview command based on filesystem metadata.
///
/// Returns an empty string for non-existent paths (caller should render
/// "(not a file)"), otherwise returns a coreutils-safe command.
fn smart_preview_command(clean_item: &str) -> String {
    match std::fs::metadata(clean_item) {
        Ok(meta) if meta.is_dir() => format!("ls -la '{}'", shell_escape_single_quote(clean_item)),
        Ok(meta) if meta.is_file() => format!(
            "cat '{}' | head -n 1000",
            shell_escape_single_quote(clean_item)
        ),
        _ => String::new(),
    }
}

/// Escape single quotes for shell single-quoted strings.
/// `'a'b'` → `a'\''b`
fn shell_escape_single_quote(s: &str) -> String {
    s.replace('\'', "'\"'\"'")
}

/// Build preview command from rules and item.
///
/// Rules are scanned in order:
///   1. First rule whose exts contain the item's extension
///   2. First rule with empty exts (default)
///
/// If no rule matches, returns empty string.
pub fn build_preview_command(item: &str, rules: &[PreviewRule]) -> String {
    let clean_item = strip_ansi_sequences(item);
    let ext = std::path::Path::new(&clean_item)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    // Find matching rule
    let rule = ext
        .as_ref()
        .and_then(|e| rules.iter().find(|r| r.matches_ext(e)))
        .or_else(|| rules.iter().find(|r| r.exts.is_empty()));

    let Some(rule) = rule else {
        return String::new();
    };

    if rule.cmd == AUTO_SENTINEL {
        return smart_preview_command(&clean_item);
    }

    let tmpl = &rule.cmd;
    let escaped = format!("'{}'", shell_escape_single_quote(&clean_item));
    if tmpl.contains("{}") {
        tmpl.replace("{}", &escaped)
    } else {
        format!("{} {}", tmpl, escaped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ansi_basic_colors() {
        let text = "\x1b[31mred\x1b[0m normal";
        let lines = parse_ansi_output(text);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0][0].0, "red");
        assert_eq!(lines[0][0].1, Some(Color::Red));
        assert_eq!(lines[0][1].0, " normal");
        assert_eq!(lines[0][1].1, None);
    }

    #[test]
    fn test_parse_ansi_bold_underline() {
        let text = "\x1b[1mbold\x1b[4munder\x1b[0m";
        let lines = parse_ansi_output(text);
        assert!(lines[0][0].3); // bold
        assert!(!lines[0][0].4); // not underline
        assert!(lines[0][1].3); // bold
        assert!(lines[0][1].4); // underline
    }

    #[test]
    fn test_parse_ansi_multiline() {
        let text = "line1\nline2\n";
        let lines = parse_ansi_output(text);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0][0].0, "line1");
        assert_eq!(lines[1][0].0, "line2");
    }

    #[test]
    fn test_preview_rule_parse_default() {
        let r = PreviewRule::parse("bat").unwrap();
        assert_eq!(r.cmd, "bat");
        assert!(r.exts.is_empty());
    }

    #[test]
    fn test_preview_rule_parse_with_exts() {
        let r = PreviewRule::parse("bat {rs,toml}").unwrap();
        assert_eq!(r.cmd, "bat");
        assert_eq!(r.exts, vec!["rs", "toml"]);
    }

    #[test]
    fn test_preview_rule_parse_explicit_default() {
        let r = PreviewRule::parse("cat {}").unwrap();
        assert_eq!(r.cmd, "cat");
        assert!(r.exts.is_empty());
    }

    #[test]
    fn test_preview_rule_matches_ext() {
        let r = PreviewRule::parse("bat {RS,toml}").unwrap();
        assert!(r.matches_ext("rs"));
        assert!(r.matches_ext("RS"));
        assert!(r.matches_ext("toml"));
        assert!(!r.matches_ext("md"));
    }

    #[test]
    fn test_build_preview_command_default() {
        let rules = vec![PreviewRule::parse("cat").unwrap()];
        assert_eq!(build_preview_command("foo.rs", &rules), "cat 'foo.rs'");
    }

    #[test]
    fn test_build_preview_command_with_exts() {
        let rules = vec![
            PreviewRule::parse("bat {rs,toml}").unwrap(),
            PreviewRule::parse("glow {md}").unwrap(),
            PreviewRule::parse("cat").unwrap(),
        ];
        assert_eq!(build_preview_command("foo.rs", &rules), "bat 'foo.rs'");
        assert_eq!(build_preview_command("foo.md", &rules), "glow 'foo.md'");
        assert_eq!(build_preview_command("foo.txt", &rules), "cat 'foo.txt'");
    }

    #[test]
    fn test_build_preview_command_escapes_special_chars() {
        let rules = vec![PreviewRule::parse("cat").unwrap()];
        assert_eq!(
            build_preview_command("provider | name", &rules),
            "cat 'provider | name'"
        );
        assert_eq!(
            build_preview_command("it's ok", &rules),
            "cat 'it'\"'\"'s ok'"
        );
    }

    #[test]
    fn test_build_preview_command_no_match() {
        let rules = vec![PreviewRule::parse("bat {rs}").unwrap()];
        assert_eq!(build_preview_command("foo.md", &rules), "");
    }

    #[test]
    fn test_strip_ansi_basic() {
        let s = "\x1b[31mred\x1b[0m normal";
        assert_eq!(strip_ansi_sequences(s), "red normal");
    }

    #[test]
    fn test_strip_ansi_no_ansi() {
        let s = "hello world";
        assert_eq!(strip_ansi_sequences(s), "hello world");
    }

    #[test]
    fn test_strip_ansi_eza_icon() {
        let s = "\x1b[1;34m\u{f016}\x1b[0m file.txt";
        assert_eq!(strip_ansi_sequences(s), "\u{f016} file.txt");
    }

    #[test]
    fn test_build_preview_command_strips_ansi() {
        let rules = vec![PreviewRule::parse("cat").unwrap()];
        assert_eq!(
            build_preview_command("\x1b[31mfoo.txt\x1b[0m", &rules),
            "cat 'foo.txt'"
        );
    }

    #[test]
    fn test_preview_rule_parse_auto() {
        let r = PreviewRule::parse("auto").unwrap();
        assert_eq!(r.cmd, AUTO_SENTINEL);
        assert!(r.exts.is_empty());

        let r2 = PreviewRule::parse("AUTO").unwrap();
        assert_eq!(r2.cmd, AUTO_SENTINEL);
    }

    #[test]
    fn test_smart_preview_file() {
        let tmp = std::env::temp_dir().join("ff_test_smart_file.txt");
        std::fs::write(&tmp, "hello").unwrap();
        let cmd = smart_preview_command(tmp.to_str().unwrap());
        assert!(cmd.starts_with("cat "));
        assert!(cmd.contains("| head -n 1000"));
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_smart_preview_directory() {
        let tmp = std::env::temp_dir().join("ff_test_smart_dir");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir(&tmp).unwrap();
        let cmd = smart_preview_command(tmp.to_str().unwrap());
        assert!(cmd.starts_with("ls -la "));
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_smart_preview_nonexistent() {
        let cmd = smart_preview_command("/ff_test_definitely_does_not_exist_12345");
        assert_eq!(cmd, "");
    }

    #[test]
    fn test_auto_composes_with_explicit_rules() {
        // Explicit rule first, auto fallback
        let rules = vec![
            PreviewRule::parse("bat {rs}").unwrap(),
            PreviewRule::parse("auto").unwrap(),
        ];
        // .rs hits explicit rule
        assert_eq!(build_preview_command("foo.rs", &rules), "bat 'foo.rs'");
        // .md falls through to auto — but we can't test exact command because
        // it depends on whether the file exists. We can at least verify it
        // generates a command (or empty for non-existent).
        let cmd = build_preview_command("foo.md", &rules);
        // Since foo.md doesn't exist, auto returns ""
        assert_eq!(cmd, "");
    }

    #[test]
    fn test_apply_result_ignores_stale_item() {
        let mut state = PreviewState::new();
        state.start_loading("a.rs");
        assert!(state.loading);

        state.apply_result(PreviewUpdate {
            item: "b.rs".to_string(),
            result: PreviewResult::Success(vec![vec![(
                "stale".to_string(),
                None,
                None,
                false,
                false,
            )]]),
        });

        assert!(state.loading, "a stale result must not clear loading");
        assert!(state.lines.is_empty());
        assert!(!state.cache.contains_key("b.rs"));
        assert_eq!(state.current_item, "a.rs");
    }

    #[test]
    fn test_apply_result_updates_lines_and_cache() {
        let mut state = PreviewState::new();
        state.start_loading("a.rs");

        state.apply_result(PreviewUpdate {
            item: "a.rs".to_string(),
            result: PreviewResult::Success(vec![vec![(
                "fresh".to_string(),
                None,
                None,
                false,
                false,
            )]]),
        });

        assert!(!state.loading);
        assert_eq!(state.lines[0][0].0, "fresh");
        assert!(state.cache.contains_key("a.rs"));
        assert!(state.error.is_none());
    }

    fn not_cancelled() -> std::sync::atomic::AtomicBool {
        std::sync::atomic::AtomicBool::new(false)
    }

    #[test]
    fn test_preview_command_captures_stdout() {
        let result = run_preview_command(
            "printf 'one\\ntwo\\n'",
            std::time::Duration::from_secs(5),
            &not_cancelled(),
        );

        match result {
            Some(PreviewResult::Success(lines)) => {
                assert_eq!(lines.len(), 2);
                assert_eq!(lines[0][0].0, "one");
                assert_eq!(lines[1][0].0, "two");
            }
            _ => panic!("expected successful output"),
        }
    }

    #[test]
    fn test_preview_command_reports_failure() {
        let result = run_preview_command(
            "exit 3",
            std::time::Duration::from_secs(5),
            &not_cancelled(),
        );
        assert!(matches!(result, Some(PreviewResult::Error(_))));
    }

    #[test]
    fn test_preview_command_times_out() {
        let result = run_preview_command(
            "sleep 30",
            std::time::Duration::from_millis(50),
            &not_cancelled(),
        );

        match result {
            Some(PreviewResult::Error(msg)) => assert!(msg.contains("timed out"), "got: {msg}"),
            _ => panic!("expected a timeout"),
        }
    }

    #[test]
    fn test_preview_command_is_cancelled_promptly() {
        let cancel = std::sync::atomic::AtomicBool::new(true);
        let start = std::time::Instant::now();

        let result = run_preview_command("sleep 30", std::time::Duration::from_secs(30), &cancel);

        assert!(result.is_none(), "a cancelled run produces no result");
        assert!(
            start.elapsed() < std::time::Duration::from_secs(2),
            "cancellation must not wait for the command"
        );
    }

    /// Dropping the handle must kill the child, not merely stop waiting for it.
    #[tokio::test]
    async fn test_dropping_preview_task_kills_the_command() {
        let dir = tempfile::TempDir::new().unwrap();
        let marker = dir.path().join("marker");
        let (tx, rx) = std::sync::mpsc::channel();

        let task = spawn_preview_task(
            "item".to_string(),
            format!("sleep 0.6 && touch {}", marker.display()),
            tx,
        );
        drop(task);

        std::thread::sleep(std::time::Duration::from_millis(1200));

        assert!(
            !marker.exists(),
            "the preview command kept running after its handle was dropped"
        );
        assert!(
            rx.try_recv().is_err(),
            "a cancelled preview must not send a result"
        );
    }
}
