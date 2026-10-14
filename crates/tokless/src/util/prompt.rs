// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/prompt.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::collections::HashSet;
use std::io::{self, BufRead, Read, Write};
use std::sync::atomic::AtomicU32;

use super::colors::{interactive_tty, pick, set_vt_ready, stdin_is_tty, vt_ready, C, SYM};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultiSelectOption {
    pub value: String,
    pub label: String,
    pub hint: String,
    pub selected: bool,
    pub disabled: bool,
    pub disabled_reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub hint: String,
    pub selected: bool,
}

pub fn is_interactive() -> bool {
    stdin_is_tty()
}

#[allow(dead_code)]
static SAVED_OUTPUT_CP: AtomicU32 = AtomicU32::new(0);

#[cfg(windows)]
mod win_console {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;
    use std::sync::atomic::Ordering;
    use super::SAVED_OUTPUT_CP;

    const WIN_ENABLE_PROCESSED_INPUT: u32 = 0x0001;
    const WIN_ENABLE_LINE_INPUT: u32 = 0x0002;
    const WIN_ENABLE_ECHO_INPUT: u32 = 0x0004;
    const WIN_ENABLE_WINDOW_INPUT: u32 = 0x0008;
    const WIN_ENABLE_MOUSE_INPUT: u32 = 0x0010;
    const WIN_ENABLE_VT_INPUT: u32 = 0x0200;
    const WIN_ENABLE_VT_PROCESSING: u32 = 0x0004;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetConsoleMode(hConsoleHandle: *mut c_void, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: *mut c_void, dwMode: u32) -> i32;
        fn GetConsoleOutputCP() -> u32;
        fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    }

    fn get_console_mode(handle: *mut c_void) -> Option<u32> {
        let mut mode: u32 = 0;
        let ok = unsafe { GetConsoleMode(handle, &mut mode) };
        if ok != 0 { Some(mode) } else { None }
    }

    fn set_console_mode(handle: *mut c_void, mode: u32) -> bool {
        unsafe { SetConsoleMode(handle, mode) != 0 }
    }

    pub fn restore_console_cp() {
        let saved = SAVED_OUTPUT_CP.load(Ordering::Relaxed);
        if saved != 0 && saved != 65001 {
            unsafe {
                let _ = SetConsoleOutputCP(saved);
            }
        }
    }

    pub fn enable_vt() -> bool {
        unsafe {
            let cp = GetConsoleOutputCP();
            if cp != 0 {
                SAVED_OUTPUT_CP.store(cp, Ordering::Relaxed);
            }
            let _ = SetConsoleOutputCP(65001);
        }
        let mut ok = true;
        let stdout_h = std::io::stdout().as_raw_handle() as *mut c_void;
        let stderr_h = std::io::stderr().as_raw_handle() as *mut c_void;
        for h in [stdout_h, stderr_h] {
            if let Some(mode) = get_console_mode(h) {
                if mode & WIN_ENABLE_VT_PROCESSING == 0 {
                    if !set_console_mode(h, mode | WIN_ENABLE_VT_PROCESSING) {
                        ok = false;
                    }
                }
            }
        }
        ok
    }

    pub fn raw_mode() -> Option<Box<dyn FnOnce()>> {
        let stdin_h = std::io::stdin().as_raw_handle() as *mut c_void;
        let saved = get_console_mode(stdin_h)?;
        let mut raw = saved
            & !(WIN_ENABLE_ECHO_INPUT
                | WIN_ENABLE_LINE_INPUT
                | WIN_ENABLE_PROCESSED_INPUT
                | WIN_ENABLE_WINDOW_INPUT
                | WIN_ENABLE_MOUSE_INPUT);
        raw |= WIN_ENABLE_VT_INPUT;
        if !set_console_mode(stdin_h, raw) {
            return None;
        }
        let handle_addr = stdin_h as usize;
        Some(Box::new(move || {
            let _ = set_console_mode(handle_addr as *mut c_void, saved);
        }))
    }

    pub fn suspend_tty(restore: Box<dyn FnOnce()>) -> (Box<dyn FnOnce()>, bool) {
        (restore, false)
    }
}

#[cfg(not(windows))]
mod unix_console {
    use std::process::{Command, Stdio};

    pub fn restore_console_cp() {}

    pub fn enable_vt() -> bool {
        true
    }

    pub fn raw_mode() -> Option<Box<dyn FnOnce()>> {
        let saved_out = Command::new("stty")
            .args(["-F", "/dev/tty", "-g"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .or_else(|| {
                Command::new("stty")
                    .arg("-g")
                    .stdin(Stdio::inherit())
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
            })?;
        let saved = String::from_utf8_lossy(&saved_out.stdout).trim().to_string();
        if saved.is_empty() {
            return None;
        }

        let set_ok = Command::new("stty")
            .args(["-F", "/dev/tty", "raw", "-echo"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            || Command::new("stty")
                .args(["raw", "-echo"])
                .stdin(Stdio::inherit())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        if !set_ok {
            return None;
        }

        Some(Box::new(move || {
            let ok = Command::new("stty")
                .args(["-F", "/dev/tty", &saved])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !ok {
                let _ = Command::new("stty")
                    .arg(&saved)
                    .stdin(Stdio::inherit())
                    .status();
            }
        }))
    }

    pub fn suspend_tty(restore: Box<dyn FnOnce()>) -> (Box<dyn FnOnce()>, bool) {
        restore();
        let _ = std::io::Write::write_all(&mut std::io::stdout(), b"\r\n");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        restore_console_cp();
        unsafe {
            libc::kill(libc::getpid(), libc::SIGTSTP);
        }
        match raw_mode() {
            Some(next) => (next, true),
            None => (Box::new(|| {}), false),
        }
    }
}

pub fn restore_console_cp() {
    #[cfg(windows)]
    win_console::restore_console_cp();
    #[cfg(not(windows))]
    unix_console::restore_console_cp();
}

pub fn enable_vt() -> bool {
    #[cfg(windows)]
    let ok = win_console::enable_vt();
    #[cfg(not(windows))]
    let ok = unix_console::enable_vt();
    set_vt_ready(ok);
    ok
}

pub fn raw_mode() -> Option<Box<dyn FnOnce()>> {
    #[cfg(windows)]
    return win_console::raw_mode();
    #[cfg(not(windows))]
    return unix_console::raw_mode();
}

fn suspend_tty(restore: Box<dyn FnOnce()>) -> (Box<dyn FnOnce()>, bool) {
    #[cfg(windows)]
    return win_console::suspend_tty(restore);
    #[cfg(not(windows))]
    return unix_console::suspend_tty(restore);
}

fn prompt_exit(restore: Option<Box<dyn FnOnce()>>) -> ! {
    if let Some(r) = restore {
        r();
    }
    print!("\r\n");
    let _ = io::stdout().flush();
    restore_console_cp();
    std::process::exit(130);
}

fn read_byte<R: Read>(r: &mut R) -> Option<u8> {
    let mut b = [0u8; 1];
    if r.read_exact(&mut b).is_ok() {
        Some(b[0])
    } else {
        None
    }
}

pub fn select_one(question: &str, options: &[SelectOption]) -> String {
    if options.is_empty() {
        return String::new();
    }
    let selected = options.iter().position(|o| o.selected).unwrap_or(0);
    if !interactive_tty() {
        return options[selected].value.clone();
    }
    let mut restore = match raw_mode() {
        Some(r) if vt_ready() => Some(r),
        Some(r) => {
            r();
            return select_one_line(question, options, selected);
        }
        None => return select_one_line(question, options, selected),
    };

    let mut cursor = selected;
    let mut first = true;
    let dot = pick("●", "*");
    let divider = pick("─", "-");
    let divider_w = 60usize;
    let ptr = pick("❯", ">");
    let key_arrow = format!(
        "{}{}{}{}",
        C.orange("↑/↓"),
        C.dim(" move · "),
        C.orange("<enter>"),
        C.dim(" confirm")
    );

    let render = |cursor: usize, first: &mut bool| {
        if !*first {
            print!("\x1b[{}A", options.len() + 3);
        }
        *first = false;
        print!("\x1b[0J");
        println!(
            "{} {}",
            C.magenta(&C.bold(dot)),
            C.magenta(&C.bold(question))
        );
        for (i, o) in options.iter().enumerate() {
            let mark = if i == cursor {
                C.cyan(&C.bold(ptr))
            } else {
                " ".to_string()
            };
            let (box_glyph, label) = if i == cursor {
                (C.green(SYM.selected()), C.cyan(&C.bold(&o.label)))
            } else {
                (C.gray(SYM.unselected()), o.label.clone())
            };
            let extra = if o.hint.is_empty() {
                String::new()
            } else {
                format!("  {}", C.dim(&o.hint))
            };
            println!(
                "{}{} {}  {}{}",
                C.dim("│   "),
                mark,
                box_glyph,
                label,
                extra
            );
        }
        println!("{}", C.dim(&format!("  {}", divider.repeat(divider_w))));
        println!("  {}", key_arrow);
        let _ = io::stdout().flush();
    };

    render(cursor, &mut first);
    let mut stdin = io::stdin();
    loop {
        let Some(ch) = read_byte(&mut stdin) else {
            if let Some(r) = restore.take() {
                r();
            }
            return options[cursor].value.clone();
        };
        match ch {
            3 => prompt_exit(restore.take()),
            26 => {
                if let Some(r) = restore.take() {
                    let (next, ok) = suspend_tty(r);
                    restore = Some(next);
                    if ok {
                        render(cursor, &mut first);
                    }
                }
            }
            27 => {
                let b1 = read_byte(&mut stdin).unwrap_or(0);
                let b2 = read_byte(&mut stdin).unwrap_or(0);
                if b1 == b'[' && b2 == b'A' {
                    cursor = (cursor + options.len() - 1) % options.len();
                    render(cursor, &mut first);
                } else if b1 == b'[' && b2 == b'B' {
                    cursor = (cursor + 1) % options.len();
                    render(cursor, &mut first);
                }
            }
            b' ' | b'\r' | b'\n' => {
                if let Some(r) = restore.take() {
                    r();
                }
                settle_select_one(question, options, cursor);
                return options[cursor].value.clone();
            }
            _ => {}
        }
    }
}

fn settle_select_one(question: &str, options: &[SelectOption], chosen: usize) {
    let height = options.len() + 3;
    print!("\x1b[{}A\x1b[0J", height);
    let corner = pick("├─ ", "+- ");
    let stem = C.dim(pick("│   ", "|   "));
    let bul = pick("│", "|");
    println!("{}{}", C.dim(corner), C.bold(question));
    for (i, o) in options.iter().enumerate() {
        let (mark, label) = if i == chosen {
            (C.green(SYM.selected()), C.green(&C.bold(&o.label)))
        } else {
            (C.gray(SYM.unselected()), o.label.clone())
        };
        let extra = if o.hint.is_empty() {
            String::new()
        } else {
            format!("  {}", C.dim(&o.hint))
        };
        println!("{}{}  {}{}", stem, mark, label, extra);
    }
    println!("{}", C.dim(bul));
    let _ = io::stdout().flush();
}

fn settle_multi_select(question: &str, items: &[MultiSelectOption]) {
    let height = items.len() + 3;
    print!("\x1b[{}A\x1b[0J", height);
    let corner = pick("├─ ", "+- ");
    let stem = C.dim(pick("│   ", "|   "));
    let bul = pick("│", "|");

    let label_w = items
        .iter()
        .map(|it| it.label.chars().count())
        .max()
        .unwrap_or(0);

    println!("{}{}", C.dim(corner), C.bold(question));
    for it in items {
        let pad = " ".repeat(label_w.saturating_sub(it.label.chars().count()));
        let extra = if it.hint.is_empty() {
            String::new()
        } else {
            format!("  {}", C.dim(&it.hint))
        };
        let (box_glyph, label, tag) = if it.disabled {
            (
                C.dim(SYM.disabled()),
                C.dim(&it.label),
                C.yellow("[MISSING]"),
            )
        } else if it.selected {
            (
                C.green(SYM.selected()),
                C.green(&C.bold(&it.label)),
                format!("{}{}  ", C.green(&C.bold("✓ ")), C.green("[READY]")),
            )
        } else {
            (
                C.gray(SYM.unselected()),
                it.label.clone(),
                format!("{}  ", C.green("[READY]")),
            )
        };
        println!("{}{}  {}{}  {}{}", stem, box_glyph, label, pad, tag, extra);
    }
    println!("{}", C.dim(bul));
    let _ = io::stdout().flush();
}

fn select_one_line(question: &str, options: &[SelectOption], selected: usize) -> String {
    println!("{} {}", C.bold(&C.cyan("?")), C.bold(question));
    for (i, o) in options.iter().enumerate() {
        let mark = if i == selected { "*" } else { " " };
        let extra = if o.hint.is_empty() {
            String::new()
        } else {
            format!("  {}", C.dim(&o.hint))
        };
        println!("  {}) {}{}{}", i + 1, mark, o.label, extra);
    }
    print!("  {}", C.gray(&format!("Enter = {}: ", options[selected].label)));
    let _ = io::stdout().flush();

    let mut line = String::new();
    let _ = io::stdin().lock().read_line(&mut line);
    if let Ok(n) = line.trim().parse::<usize>() {
        if n >= 1 && n <= options.len() {
            return options[n - 1].value.clone();
        }
    }
    options[selected].value.clone()
}

pub fn multi_select(question: &str, options: &[MultiSelectOption]) -> Vec<String> {
    if !interactive_tty() {
        return options
            .iter()
            .filter(|o| !o.disabled && o.selected)
            .map(|o| o.value.clone())
            .collect();
    }

    let mut items = options.to_vec();
    let mut cursor = 0;
    while cursor < items.len() && items[cursor].disabled {
        cursor += 1;
    }
    if cursor == items.len() {
        return multi_select_line(question, &items);
    }

    let mut restore = match raw_mode() {
        Some(r) if vt_ready() => Some(r),
        Some(r) => {
            r();
            return multi_select_line(question, &items);
        }
        None => return multi_select_line(question, &items),
    };

    let mut first_render = true;
    const HEADER_LINES: usize = 3;

    let label_w = items
        .iter()
        .map(|it| it.label.chars().count())
        .max()
        .unwrap_or(0);

    let dot = pick("●", "*");
    let divider = pick("─", "-");
    let divider_w = 60usize;
    let ptr = pick("❯", ">");
    let check_mark = "✓";
    let tag_pad = "  ";
    let key_hint = format!(
        "  {}{}{}{}{}{}{}{}",
        C.orange("↑/↓"),
        C.dim(" move · "),
        C.orange("<space>"),
        C.dim(" select · "),
        C.orange("<a>"),
        C.dim(" all · "),
        C.orange("<enter>"),
        C.dim(" confirm")
    );

    let render = |items: &[MultiSelectOption], cursor: usize, first_render: &mut bool| {
        if !*first_render {
            print!("\x1b[{}A", items.len() + HEADER_LINES);
        }
        *first_render = false;
        print!("\x1b[0J");
        let mut b = String::new();
        b.push_str(&format!(
            "{} {}\r\n",
            C.magenta(&C.bold(dot)),
            C.magenta(&C.bold(question))
        ));

        for (i, it) in items.iter().enumerate() {
            let pad = " ".repeat(label_w.saturating_sub(it.label.chars().count()));
            let active = i == cursor && !it.disabled;
            let ptr_col = if active {
                format!("{} ", C.cyan(&C.bold(ptr)))
            } else {
                "  ".to_string()
            };

            let extra = if it.hint.is_empty() {
                String::new()
            } else {
                format!("  {}", C.dim(&it.hint))
            };

            let (box_glyph, mut label, tag) = if it.disabled {
                (
                    C.dim(SYM.disabled()),
                    C.dim(&it.label),
                    C.yellow("[MISSING]"),
                )
            } else if it.selected {
                (
                    C.green(SYM.selected()),
                    C.green(&C.bold(&it.label)),
                    format!(
                        "{}{}  ",
                        C.green(&C.bold(&format!("{} ", check_mark))),
                        C.green("[READY]")
                    ),
                )
            } else {
                (
                    C.gray(SYM.unselected()),
                    it.label.clone(),
                    format!("{}{}  ", tag_pad, C.green("[READY]")),
                )
            };

            if active {
                label = C.cyan(&C.bold(&it.label));
            }

            b.push_str(&format!(
                "{}{}{}  {}{}  {}{}\r\n",
                C.dim("│   "),
                ptr_col,
                box_glyph,
                label,
                pad,
                tag,
                extra
            ));
        }
        b.push_str(&format!(
            "{}\r\n",
            C.dim(&format!("  {}", divider.repeat(divider_w)))
        ));
        b.push_str(&format!("{}\r\n", key_hint));
        print!("{}", b);
        let _ = io::stdout().flush();
    };

    render(&items, cursor, &mut first_render);
    let mut stdin = io::stdin();

    loop {
        let Some(ch) = read_byte(&mut stdin) else {
            break;
        };
        match ch {
            3 => prompt_exit(restore.take()),
            26 => {
                if let Some(r) = restore.take() {
                    let (next, ok) = suspend_tty(r);
                    restore = Some(next);
                    if ok {
                        render(&items, cursor, &mut first_render);
                    }
                }
            }
            27 => {
                let b1 = read_byte(&mut stdin).unwrap_or(0);
                if b1 == b'[' {
                    let b2 = read_byte(&mut stdin).unwrap_or(0);
                    if b2 == b'A' {
                        move_cursor(&mut cursor, &items, -1);
                        render(&items, cursor, &mut first_render);
                    } else if b2 == b'B' {
                        move_cursor(&mut cursor, &items, 1);
                        render(&items, cursor, &mut first_render);
                    }
                }
            }
            b' ' => {
                if !items[cursor].disabled {
                    items[cursor].selected = !items[cursor].selected;
                }
                render(&items, cursor, &mut first_render);
            }
            b'a' => {
                let all_on = items
                    .iter()
                    .filter(|it| !it.disabled)
                    .all(|it| it.selected);
                for it in &mut items {
                    if !it.disabled {
                        it.selected = !all_on;
                    }
                }
                render(&items, cursor, &mut first_render);
            }
            b'\r' | b'\n' => {
                if let Some(r) = restore.take() {
                    r();
                }
                settle_multi_select(question, &items);
                return items
                    .into_iter()
                    .filter(|it| it.selected && !it.disabled)
                    .map(|it| it.value)
                    .collect();
            }
            _ => {}
        }
    }
    if let Some(r) = restore.take() {
        r();
    }
    Vec::new()
}

fn move_cursor(cursor: &mut usize, items: &[MultiSelectOption], delta: isize) {
    let n = items.len() as isize;
    if n == 0 {
        return;
    }
    loop {
        *cursor = ((*cursor as isize + delta).rem_euclid(n)) as usize;
        if !items[*cursor].disabled {
            return;
        }
    }
}

fn multi_select_line(question: &str, items: &[MultiSelectOption]) -> Vec<String> {
    println!("{} {}", C.bold(&C.cyan("?")), C.bold(question));
    let label_w = items
        .iter()
        .map(|it| it.label.chars().count())
        .max()
        .unwrap_or(0);

    let mut defaults = Vec::new();
    let mut num = 0usize;
    let mut num_by_idx = vec![0usize; items.len()];

    for (i, it) in items.iter().enumerate() {
        let pad = " ".repeat(label_w.saturating_sub(it.label.chars().count()));
        if it.disabled {
            num_by_idx[i] = 0;
            let mut line = format!("       {}{}  {}", C.dim(&it.label), pad, C.yellow("[MISSING]"));
            if !it.hint.is_empty() {
                line.push_str(&format!("  {}", C.dim(&it.hint)));
            }
            println!("{}", line);
            continue;
        }
        num += 1;
        num_by_idx[i] = num;
        let mark = if it.selected {
            defaults.push(it.value.clone());
            "*"
        } else {
            " "
        };
        let mut line = format!(
            "  {:2}) {}{}{}  {}  ",
            num,
            mark,
            it.label,
            pad,
            C.green("[READY]")
        );
        if !it.hint.is_empty() {
            line.push_str(&format!("  {}", C.dim(&it.hint)));
        }
        println!("{}", line);
    }
    if num == 0 {
        return Vec::new();
    }
    let hint = if defaults.is_empty() { "none" } else { "keep *" };
    print!(
        "  {}",
        C.gray(&format!(
            "Numbers comma-separated, 'a' = all, Enter = {}: ",
            hint
        ))
    );
    let _ = io::stdout().flush();

    let mut line = String::new();
    let _ = io::stdin().lock().read_line(&mut line);
    parse_line_selection(&line, items, &num_by_idx, &defaults)
}

pub fn parse_line_selection(
    line: &str,
    items: &[MultiSelectOption],
    num_by_idx: &[usize],
    defaults: &[String],
) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return defaults.to_vec();
    }
    if trimmed.eq_ignore_ascii_case("a") {
        return items
            .iter()
            .filter(|it| !it.disabled)
            .map(|it| it.value.clone())
            .collect();
    }
    let mut want = HashSet::new();
    for tok in trimmed.split(',') {
        if let Ok(n) = tok.trim().parse::<usize>() {
            want.insert(n);
        }
    }
    let mut out = Vec::new();
    for (i, it) in items.iter().enumerate() {
        if let Some(&num) = num_by_idx.get(i) {
            if num > 0 && want.contains(&num) {
                out.push(it.value.clone());
            }
        }
    }
    out
}

pub fn confirm(question: &str, default_yes: bool) -> bool {
    if !interactive_tty() {
        return default_yes;
    }
    let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{} {} {} ", C.cyan("?"), C.bold(question), C.gray(hint));
    let _ = io::stdout().flush();

    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line).is_ok() {
        let a = line.trim().to_lowercase();
        if a.is_empty() {
            return default_yes;
        }
        return a == "y" || a == "yes";
    }
    default_yes
}
