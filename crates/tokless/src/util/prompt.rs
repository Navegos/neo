use std::io::{self, BufRead, Write};
use super::colors::{interactive_tty, stdin_is_tty, C};

#[derive(Debug, Clone, Default)]
pub struct MultiSelectOption {
    pub value: String,
    pub label: String,
    pub hint: String,
    pub selected: bool,
    pub disabled: bool,
    pub disabled_reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub hint: String,
    pub selected: bool,
}

pub fn is_interactive() -> bool {
    stdin_is_tty()
}

pub fn confirm(question: &str, default_val: bool) -> bool {
    if !interactive_tty() {
        return default_val;
    }
    let hint = if default_val { " [Y/n] " } else { " [y/N] " };
    print!("{}{}", C.bold(question), hint);
    io::stdout().flush().unwrap();

    let mut line = String::new();
    let stdin = io::stdin();
    if stdin.lock().read_line(&mut line).is_ok() {
        let trimmed = line.trim().to_lowercase();
        if trimmed.is_empty() {
            return default_val;
        }
        return trimmed == "y" || trimmed == "yes";
    }
    default_val
}

pub fn select_one(question: &str, options: &[SelectOption]) -> String {
    if options.is_empty() {
        return String::new();
    }
    let selected = options.iter().position(|o| o.selected).unwrap_or(0);
    if !interactive_tty() {
        return options[selected].value.clone();
    }

    println!("\n{}", C.bold(question));
    for (i, opt) in options.iter().enumerate() {
        let num = format!("{}. ", i + 1);
        let mut line = format!("  {}{}", C.cyan(&num), opt.label);
        if !opt.hint.is_empty() {
            line.push_str(&format!(" ({})", C.dim(&opt.hint)));
        }
        println!("{}", line);
    }
    print!("Select [1-{}, default {}]: ", options.len(), selected + 1);
    io::stdout().flush().unwrap();

    let mut line = String::new();
    let stdin = io::stdin();
    if stdin.lock().read_line(&mut line).is_ok() {
        let trimmed = line.trim();
        if let Ok(choice) = trimmed.parse::<usize>() {
            if choice >= 1 && choice <= options.len() {
                return options[choice - 1].value.clone();
            }
        }
    }
    options[selected].value.clone()
}

pub fn multi_select(question: &str, options: &[MultiSelectOption]) -> Vec<String> {
    if options.is_empty() {
        return Vec::new();
    }
    if !interactive_tty() {
        return options.iter().filter(|o| o.selected && !o.disabled).map(|o| o.value.clone()).collect();
    }

    println!("\n{}", C.bold(question));
    for (i, opt) in options.iter().enumerate() {
        let mark = if opt.disabled {
            C.gray("[-]")
        } else if opt.selected {
            C.green("[*]")
        } else {
            C.gray("[ ]")
        };
        let num = format!("{}. ", i + 1);
        let mut line = format!("  {} {}{}", mark, C.cyan(&num), opt.label);
        if !opt.hint.is_empty() {
            line.push_str(&format!(" ({})", C.dim(&opt.hint)));
        }
        if opt.disabled && !opt.disabled_reason.is_empty() {
            line.push_str(&format!(" [{}]", C.dim(&opt.disabled_reason)));
        }
        println!("{}", line);
    }
    print!("Enter numbers separated by comma (or enter for default): ");
    io::stdout().flush().unwrap();

    let mut line = String::new();
    let stdin = io::stdin();
    if stdin.lock().read_line(&mut line).is_ok() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let mut result = Vec::new();
            for part in trimmed.split(',') {
                if let Ok(choice) = part.trim().parse::<usize>() {
                    if choice >= 1 && choice <= options.len() {
                        let opt = &options[choice - 1];
                        if !opt.disabled {
                            result.push(opt.value.clone());
                        }
                    }
                }
            }
            if !result.is_empty() {
                return result;
            }
        }
    }
    options.iter().filter(|o| o.selected && !o.disabled).map(|o| o.value.clone()).collect()
}
