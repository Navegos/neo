use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub use super::colors::rule;
use super::colors::{
    glyphs_enabled, pick, running_inside_claude_code, stdout_is_tty, vt_ready, C, SYM,
};
use super::logger::{begin_log_capture, end_log_capture, quiet};

fn spinner_frames() -> &'static [&'static str] {
    if glyphs_enabled() {
        &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
    } else {
        &["|", "/", "-", "\\"]
    }
}

struct ProgressInner {
    title: String,
    current: String,
    phase: String,
    frac: f64,
    start: Instant,
    frame: usize,
    active: bool,
    stopped: bool,
    tty: bool,
    tree_style: bool,
    tree_root: bool,
    rows: usize,
    last_non_tty: String,
    last_width: usize,
}

impl ProgressInner {
    fn indent(&self) -> String {
        if self.tree_style {
            C.dim(pick("│   ", "|   "))
        } else {
            "  ".to_string()
        }
    }

    fn repaint(&mut self) {
        if !self.tty || self.current.is_empty() || !self.active || quiet() {
            return;
        }
        let frames = spinner_frames();
        let frame_str = frames[self.frame % frames.len()];
        let pct = (self.frac * 100.0 + 0.5) as usize;
        let phase_str = if self.phase.is_empty() {
            String::new()
        } else {
            C.gray(&format!("  {}", self.phase))
        };
        let line = format!(
            "{}{} {} {}{}{}",
            self.indent(),
            C.cyan(frame_str),
            pad_end(&self.current, 16),
            C.gray(&format!("[{}] {:3}%", frac_bar(self.frac), pct)),
            phase_str,
            elapsed_str(self.start.elapsed())
        );
        let cur_width = visible_len(&line);
        let pad = if self.last_width > cur_width {
            self.last_width - cur_width
        } else {
            0
        };
        self.last_width = cur_width;
        print!("\r\x1b[2K{}{}", line, " ".repeat(pad));
        let _ = io::stdout().flush();
    }

    fn clear_line(&mut self) {
        if self.tty {
            self.last_width = 0;
            print!("\r\x1b[2K");
            let _ = io::stdout().flush();
        }
    }
}

#[derive(Clone)]
pub struct Progress {
    inner: Arc<Mutex<ProgressInner>>,
}

pub type RootSectionProgress = Progress;
pub type SectionProgress = Progress;
pub type NewRootSectionProgress = Progress;
pub type NewSectionProgress = Progress;

fn is_progress_tty() -> bool {
    stdout_is_tty() && vt_ready() && !running_inside_claude_code()
}

pub fn new_progress(title: &str) -> Progress {
    Progress {
        inner: Arc::new(Mutex::new(ProgressInner {
            title: title.to_string(),
            current: String::new(),
            phase: String::new(),
            frac: 0.0,
            start: Instant::now(),
            frame: 0,
            active: false,
            stopped: false,
            tty: is_progress_tty(),
            tree_style: false,
            tree_root: false,
            rows: 0,
            last_non_tty: String::new(),
            last_width: 0,
        })),
    }
}

pub fn new_section_progress(section: &str) -> Progress {
    Progress {
        inner: Arc::new(Mutex::new(ProgressInner {
            title: section.to_string(),
            current: String::new(),
            phase: String::new(),
            frac: 0.0,
            start: Instant::now(),
            frame: 0,
            active: false,
            stopped: false,
            tty: is_progress_tty(),
            tree_style: true,
            tree_root: false,
            rows: 0,
            last_non_tty: String::new(),
            last_width: 0,
        })),
    }
}

pub fn new_root_section_progress(section: &str) -> Progress {
    Progress {
        inner: Arc::new(Mutex::new(ProgressInner {
            title: section.to_string(),
            current: String::new(),
            phase: String::new(),
            frac: 0.0,
            start: Instant::now(),
            frame: 0,
            active: false,
            stopped: false,
            tty: is_progress_tty(),
            tree_style: true,
            tree_root: true,
            rows: 0,
            last_non_tty: String::new(),
            last_width: 0,
        })),
    }
}

#[allow(non_snake_case)]
pub fn NewProgress(title: &str) -> Progress {
    new_progress(title)
}

#[allow(non_snake_case)]
pub fn NewSectionProgress(section: &str) -> Progress {
    new_section_progress(section)
}

#[allow(non_snake_case)]
pub fn NewRootSectionProgress(section: &str) -> Progress {
    new_root_section_progress(section)
}

impl Progress {
    pub fn new(title: &str) -> Self {
        new_root_section_progress(title)
    }

    pub fn start(&self, _total: usize) {
        let should_spin = {
            let mut inner = self.inner.lock().unwrap();
            inner.rows = 0;
            inner.stopped = false;
            if !inner.title.is_empty() && !quiet() {
                if inner.tree_style {
                    if inner.tty {
                        println!(
                            "{} {}",
                            C.magenta(&C.bold(pick("●", "*"))),
                            C.magenta(&C.bold(&inner.title))
                        );
                    } else if inner.tree_root {
                        tree_top(&inner.title);
                    } else {
                        tree_corner(&inner.title);
                    }
                } else {
                    println!("\n  {}", C.bold(&C.cyan(&inner.title)));
                }
            }
            inner.tty
        };

        if should_spin {
            let weak = Arc::downgrade(&self.inner);
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(80));
                let Some(arc) = weak.upgrade() else {
                    return;
                };
                let mut inner = arc.lock().unwrap();
                if inner.stopped {
                    return;
                }
                if inner.active {
                    let n_frames = spinner_frames().len();
                    inner.frame = (inner.frame + 1) % n_frames;
                    inner.repaint();
                }
            });
        }
    }

    pub fn begin(&self, label: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.current = label.to_string();
        inner.phase.clear();
        inner.frac = 0.0;
        inner.last_non_tty.clear();
        inner.start = Instant::now();
        inner.active = true;
        if inner.tty {
            inner.repaint();
        }
    }

    pub fn step(&self, phase: &str, frac: f64) {
        let mut inner = self.inner.lock().unwrap();
        let clamped = frac.clamp(0.0, 1.0);
        inner.phase = phase.to_string();
        inner.frac = clamped;
        if inner.tty {
            inner.repaint();
            return;
        }
        if phase.is_empty() || phase == inner.last_non_tty {
            return;
        }
        inner.last_non_tty = phase.to_string();
        let label = if inner.current.is_empty() {
            inner.title.clone()
        } else {
            inner.current.clone()
        };
        if !quiet() {
            println!("  {} {} — {}", C.dim(pick("·", ".")), label, phase);
        }
    }

    pub fn complete(&self, note: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.active = false;
        inner.clear_line();
        if !quiet() {
            let note_str = if note.is_empty() {
                String::new()
            } else {
                C.gray(&format!(" {}", note))
            };
            println!(
                "{}{} {} {}{}",
                inner.indent(),
                C.green(SYM.check()),
                pad_end(&inner.current, 16),
                C.gray(&format!("[{}] 100%", frac_bar(1.0))),
                note_str
            );
        }
        if inner.tree_style {
            inner.rows += 1;
        }
    }

    pub fn fail(&self, reason: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.active = false;
        inner.clear_line();
        if !quiet() {
            println!(
                "{}{} {} {}",
                inner.indent(),
                C.red(SYM.cross()),
                pad_end(&inner.current, 16),
                C.red(reason)
            );
        }
        if inner.tree_style {
            inner.rows += 1;
        }
    }

    pub fn skip(&self, note: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.active = false;
        inner.clear_line();
        if !quiet() {
            println!(
                "{}{} {} {}",
                inner.indent(),
                C.gray(SYM.bullet()),
                pad_end(&inner.current, 16),
                C.gray(&format!("[{}] 100%  {}", frac_bar(1.0), note))
            );
        }
        if inner.tree_style {
            inner.rows += 1;
        }
    }

    pub fn done(&self, summary: &str) {
        let (tree_style, tty, tree_root, rows, title) = {
            let mut inner = self.inner.lock().unwrap();
            inner.stopped = true;
            inner.active = false;
            inner.clear_line();
            (
                inner.tree_style,
                inner.tty,
                inner.tree_root,
                inner.rows,
                inner.title.clone(),
            )
        };
        if quiet() {
            return;
        }
        if tree_style {
            if tty && !title.is_empty() {
                let title_line = format!(
                    "{}{}",
                    C.dim(tree_corner_glyph(tree_root)),
                    C.bold(&title)
                );
                let up = rows + 1;
                print!("\x1b[{}A\r\x1b[2K{}\x1b[{}B\r", up, title_line, up);
                let _ = io::stdout().flush();
            }
            tree_close();
            return;
        }
        if !summary.is_empty() {
            println!("  {}", C.gray(summary));
        }
    }
}

fn tree_corner_glyph(root: bool) -> &'static str {
    if root {
        pick("┌─ ", "+- ")
    } else {
        pick("├─ ", "+- ")
    }
}

pub fn tree_stem() -> String {
    C.dim(pick("│   ", "|   "))
}

pub fn tree_top(title: &str) {
    println!(
        "{}{}",
        C.dim(tree_corner_glyph(true)),
        C.bold(&C.magenta(title))
    );
}

pub fn tree_corner(title: &str) {
    println!(
        "{}{}",
        C.dim(tree_corner_glyph(false)),
        C.bold(&C.magenta(title))
    );
}

pub fn tree_corner_styled(title: &str) {
    println!("{}{}", C.dim(tree_corner_glyph(false)), title);
}

pub fn tree_leaf(line: &str) {
    println!("{}{}", tree_stem(), line);
}

pub fn tree_close() {
    println!("{}", C.dim(pick("│", "|")));
}

pub fn tree_footer(rule_width: usize) {
    println!("{}{}", C.dim(pick("└", "+")), C.gray(&rule(rule_width)));
}

pub fn visible_len(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut n = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            while i < bytes.len() {
                let c = bytes[i];
                if c.is_ascii_alphabetic() {
                    break;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        n += 1;
        i += 1;
    }
    n
}

pub fn erase_styled_line(_s: &str) -> String {
    if !stdout_is_tty() || running_inside_claude_code() {
        return String::new();
    }
    "\r\x1b[2K".to_string()
}

pub fn pad_end(s: &str, n: usize) -> String {
    if s.len() >= n {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(n - s.len()))
    }
}

fn frac_bar(frac: f64) -> String {
    let width = 16usize;
    let mut filled = (frac * (width as f64) + 0.5) as usize;
    if filled > width {
        filled = width;
    }
    format!(
        "{}{}",
        C.green(&"█".repeat(filled)),
        C.gray(&"░".repeat(width - filled))
    )
}

fn elapsed_str(d: Duration) -> String {
    let s = d.as_secs();
    if s == 0 {
        String::new()
    } else {
        C.gray(&format!(" {}s", s))
    }
}

pub fn capture_logs<F, T>(f: F) -> (String, T)
where
    F: FnOnce() -> T,
{
    let prev = begin_log_capture();
    let res = f();
    let captured = end_log_capture(prev);
    (captured, res)
}

pub fn with_silenced_logs<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let (_, res) = capture_logs(f);
    res
}
