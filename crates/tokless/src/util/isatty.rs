pub fn is_terminal(fd: u32) -> bool {
    match fd {
        0 => crossterm::tty::IsTty::is_tty(&std::io::stdin()),
        1 => crossterm::tty::IsTty::is_tty(&std::io::stdout()),
        2 => crossterm::tty::IsTty::is_tty(&std::io::stderr()),
        _ => false,
    }
}

pub fn is_interactive() -> bool {
    is_terminal(0) && is_terminal(1)
}

pub fn stdout_is_tty() -> bool {
    is_terminal(1)
}

