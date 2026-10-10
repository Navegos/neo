use std::sync::atomic::{AtomicBool, Ordering};
use super::colors::{C, SYM};

static VERBOSE: AtomicBool = AtomicBool::new(false);
static QUIET: AtomicBool = AtomicBool::new(false);

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

pub struct Logger;

impl Logger {
    pub fn info(&self, msg: &str) {
        println!("{} {}", C.cyan(SYM.info()), msg);
    }

    pub fn ok(&self, msg: &str) {
        println!("{} {}", C.green(SYM.check()), msg);
    }

    pub fn warn(&self, msg: &str) {
        println!("{} {}", C.yellow(SYM.warn()), msg);
    }

    pub fn err(&self, msg: &str) {
        eprintln!("{} {}", C.red(SYM.cross()), msg);
    }

    pub fn step(&self, msg: &str) {
        if !quiet() {
            println!("\n{}", C.bold(&C.magenta(&format!("{} {}", SYM.arrow(), msg))));
        }
    }

    pub fn sub(&self, msg: &str) {
        if !quiet() {
            println!("  {} {}", C.gray(SYM.bullet()), msg);
        }
    }

    pub fn debug(&self, msg: &str) {
        if VERBOSE.load(Ordering::Relaxed) {
            println!("  {}", C.gray(&format!("[debug] {}", msg)));
        }
    }

    pub fn raw(&self, msg: &str) {
        println!("{}", msg);
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
        println!("\n{}", C.cyan(&line));
        println!("  {}", C.bold(&C.cyan(title)));
        if !subtitle.is_empty() {
            println!("  {}", C.gray(subtitle));
        }
        println!("{}\n", C.cyan(&line));
    }
}

pub static L: Logger = Logger;

