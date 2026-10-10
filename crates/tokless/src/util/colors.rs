use std::sync::atomic::{AtomicBool, Ordering};

static VT_READY: AtomicBool = AtomicBool::new(true);

pub fn stdout_is_tty() -> bool {
    crossterm::tty::IsTty::is_tty(&std::io::stdout())
}

pub fn stdin_is_tty() -> bool {
    crossterm::tty::IsTty::is_tty(&std::io::stdin())
}

pub fn stdout_ansi() -> bool {
    stdout_is_tty() && VT_READY.load(Ordering::Relaxed)
}

pub fn running_inside_claude_code() -> bool {
    std::env::var("CLAUDECODE").unwrap_or_default() == "1"
        || std::env::var("CLAUDE_CODE_CHILD_SESSION").unwrap_or_default() == "1"
}

pub fn interactive_tty() -> bool {
    stdin_is_tty() && stdout_is_tty() && !running_inside_claude_code()
}

pub fn colors_enabled() -> bool {
    stdout_is_tty()
        && VT_READY.load(Ordering::Relaxed)
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").unwrap_or_default() != "dumb"
}

fn wrap(open: u8, close: u8, s: &str) -> String {
    if colors_enabled() {
        format!("\x1b[{}m{}\x1b[{}m", open, s, close)
    } else {
        s.to_string()
    }
}

fn wrap_fg256(n: u8, s: &str) -> String {
    if colors_enabled() {
        format!("\x1b[38;5;{}m{}\x1b[39m", n, s)
    } else {
        s.to_string()
    }
}

pub struct Colors;

impl Colors {
    pub fn bold(&self, s: &str) -> String { wrap(1, 22, s) }
    pub fn dim(&self, s: &str) -> String { wrap(2, 22, s) }
    pub fn italic(&self, s: &str) -> String { wrap(3, 23, s) }
    pub fn underline(&self, s: &str) -> String { wrap(4, 24, s) }
    pub fn red(&self, s: &str) -> String { wrap(31, 39, s) }
    pub fn green(&self, s: &str) -> String { wrap(32, 39, s) }
    pub fn yellow(&self, s: &str) -> String { wrap(33, 39, s) }
    pub fn blue(&self, s: &str) -> String { wrap(34, 39, s) }
    pub fn magenta(&self, s: &str) -> String { wrap(35, 39, s) }
    pub fn cyan(&self, s: &str) -> String { wrap(36, 39, s) }
    pub fn gray(&self, s: &str) -> String { wrap(90, 39, s) }
    pub fn orange(&self, s: &str) -> String { wrap_fg256(208, s) }
    pub fn soft(&self, s: &str) -> String { wrap_fg256(111, s) }
}

pub static C: Colors = Colors;

fn legacy_win_console() -> bool {
    if !cfg!(windows) {
        return false;
    }
    std::env::var("WT_SESSION").is_err()
        && std::env::var("TERM_PROGRAM").is_err()
        && std::env::var("ConEmuANSI").unwrap_or_default() != "ON"
        && std::env::var("TERM").is_err()
}

pub fn glyphs_enabled() -> bool {
    colors_enabled() && !legacy_win_console()
}

pub fn pick<'a>(uni: &'a str, ascii: &'a str) -> &'a str {
    if glyphs_enabled() {
        uni
    } else {
        ascii
    }
}

pub struct Symbols;

impl Symbols {
    pub fn bullet(&self) -> &str { pick("•", "*") }
    pub fn arrow(&self) -> &str { pick("›", ">") }
    pub fn check(&self) -> &str { pick("✔", "v") }
    pub fn cross(&self) -> &str { pick("✖", "x") }
    pub fn warn(&self) -> &str { pick("⚠", "!") }
    pub fn info(&self) -> &str { pick("ℹ", "i") }
    pub fn selected(&self) -> &str { pick("◉", "(x)") }
    pub fn unselected(&self) -> &str { pick("◯", "( )") }
    pub fn disabled(&self) -> &str { pick("·", " - ") }
    pub fn pointer(&self) -> &str { pick("❯", ">") }
}

pub static SYM: Symbols = Symbols;

pub struct Sym;
impl Sym {
    pub const CHECK: &'static str = "✔";
    pub const WARN: &'static str = "⚠";
    pub const INFO: &'static str = "ℹ";
    pub const BULLET: &'static str = "•";
    pub const CROSS: &'static str = "✖";
}

pub fn rule(width: usize) -> String {
    pick("─", "-").repeat(width)
}

