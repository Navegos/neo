use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use super::colors::{C, SYM};

static VERBOSE: AtomicBool = AtomicBool::new(false);
static QUIET: AtomicBool = AtomicBool::new(false);
static CAPTURE_BUFFER: Mutex<Option<String>> = Mutex::new(None);

pub fn set_verbose(v: bool) {
    VERBOSE.store(v, Ordering::Relaxed);
    if v {
        QUIET.store(false, Ordering::Relaxed);
    }
}

pub fn set_quiet(q: bool) {
    QUIET.store(q, Ordering::Relaxed);
}

pub fn quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}

pub fn with_quiet<F: FnOnce() -> R, R>(f: F) -> R {
    let was = QUIET.swap(true, Ordering::Relaxed);
    let res = f();
    QUIET.store(was, Ordering::Relaxed);
    res
}

pub(crate) fn begin_log_capture() -> Option<String> {
    if let Ok(mut guard) = CAPTURE_BUFFER.lock() {
        guard.replace(String::new())
    } else {
        None
    }
}

pub(crate) fn end_log_capture(prev: Option<String>) -> String {
    if let Ok(mut guard) = CAPTURE_BUFFER.lock() {
        let out = guard.take().unwrap_or_default();
        *guard = prev;
        out
    } else {
        String::new()
    }
}

fn emit_stdout(line: &str) {
    if let Ok(mut guard) = CAPTURE_BUFFER.lock() {
        if let Some(ref mut buf) = *guard {
            buf.push_str(line);
            buf.push('\n');
            return;
        }
    }
    println!("{}", line);
}

fn emit_stderr(line: &str) {
    if let Ok(mut guard) = CAPTURE_BUFFER.lock() {
        if let Some(ref mut buf) = *guard {
            buf.push_str(line);
            buf.push('\n');
            return;
        }
    }
    eprintln!("{}", line);
}

pub struct Logger;

impl Logger {
    pub fn info(&self, msg: &str) {
        emit_stdout(&format!("{} {}", C.cyan(SYM.info()), msg));
    }

    pub fn ok(&self, msg: &str) {
        emit_stdout(&format!("{} {}", C.green(SYM.check()), msg));
    }

    pub fn warn(&self, msg: &str) {
        emit_stdout(&format!("{} {}", C.yellow(SYM.warn()), msg));
    }

    pub fn err(&self, msg: &str) {
        emit_stderr(&format!("{} {}", C.red(SYM.cross()), msg));
    }

    pub fn step(&self, msg: &str) {
        if !quiet() {
            emit_stdout(&format!("\n{}", C.bold(&C.magenta(&format!("{} {}", SYM.arrow(), msg)))));
        }
    }

    pub fn sub(&self, msg: &str) {
        if !quiet() {
            emit_stdout(&format!("  {} {}", C.gray(SYM.bullet()), msg));
        }
    }

    pub fn debug(&self, msg: &str) {
        if VERBOSE.load(Ordering::Relaxed) {
            emit_stdout(&format!("  {}", C.gray(&format!("[debug] {}", msg))));
        }
    }

    pub fn raw(&self, msg: &str) {
        emit_stdout(msg);
    }

    pub fn banner(&self, title: &str, subtitle: &str) {
        if quiet() {
            return;
        }
        let mut width = title.len();
        if subtitle.len() > width {
            width = subtitle.len();
        }
        let line = "─".repeat(width + 4);
        emit_stdout(&format!("\n{}", C.cyan(&line)));
        emit_stdout(&format!("  {}", C.bold(&C.cyan(title))));
        if !subtitle.is_empty() {
            emit_stdout(&format!("  {}", C.gray(subtitle)));
        }
        emit_stdout(&format!("{}\n", C.cyan(&line)));
    }
}

pub static L: Logger = Logger;
