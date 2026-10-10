use super::colors::{pick, C, SYM};
pub use super::colors::rule;
use super::logger::quiet;

pub struct RootSectionProgress {
    title: String,
    total: usize,
    current: usize,
}

impl RootSectionProgress {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            total: 0,
            current: 0,
        }
    }

    pub fn start(&mut self, total: usize) {
        self.total = total;
        self.current = 0;
        if !quiet() {
            println!("\n{}", C.bold(&format!("{}:", self.title)));
        }
    }

    pub fn begin(&mut self, label: &str) {
        self.current += 1;
        if !quiet() {
            let count = if self.total > 0 {
                format!("[{}/{}] ", self.current, self.total)
            } else {
                String::new()
            };
            println!("  {}{}{}", C.dim(&count), SYM.bullet(), C.cyan(&format!(" {}", label)));
        }
    }

    pub fn step(&self, phase: &str, _frac: f64) {
        if !quiet() && !phase.is_empty() {
            println!("    {} {}", C.gray(SYM.arrow()), C.dim(phase));
        }
    }

    pub fn complete(&self, note: &str) {
        if !quiet() {
            if note.is_empty() {
                println!("    {}", C.green(SYM.check()));
            } else {
                println!("    {} {}", C.green(SYM.check()), note);
            }
        }
    }

    pub fn done(&self, summary: &str) {
        if !quiet() {
            if !summary.is_empty() {
                println!("    {} {}", C.green(SYM.check()), summary);
            }
        }
    }

    pub fn fail(&self, detail: &str) {
        if !quiet() {
            println!("    {} {}", C.red(SYM.cross()), detail);
        }
    }
}

pub type NewRootSectionProgress = RootSectionProgress;

pub fn new_root_section_progress(title: &str) -> RootSectionProgress {
    RootSectionProgress::new(title)
}

pub fn tree_stem() -> String {
    C.dim(pick("│   ", "|   "))
}

pub fn tree_top(title: &str) {
    println!("{}{}", C.dim(pick("┌─ ", "+- ")), C.bold(&C.magenta(title)));
}

pub fn tree_corner(title: &str) {
    println!("{}{}", C.dim(pick("├─ ", "+- ")), C.bold(&C.magenta(title)));
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

pub fn erase_styled_line(_line: &str) -> String {
    "\r\x1b[2K".to_string()
}
