use std::io::{stderr, stdin, stdout, IsTerminal};

/// Check if stdin is a TTY.
pub fn is_stdin_tty() -> bool {
    stdin().is_terminal()
}

/// Check if stdin is piped (not a TTY).
pub fn is_stdin_piped() -> bool {
    !stdin().is_terminal()
}

/// Check if stdout is a TTY.
pub fn is_stdout_tty() -> bool {
    stdout().is_terminal()
}

/// Check if stderr is a TTY.
pub fn is_stderr_tty() -> bool {
    stderr().is_terminal()
}

/// Check if TTY requirements are met for interactive mode.
/// The TUI renders to stderr, so we always need stderr to be a TTY.
/// When stdin is piped, we reopen /dev/tty for keyboard input,
/// so we only need stderr for rendering.
pub fn check_tty_requirements() -> bool {
    // stderr must be a TTY since the TUI renders there
    is_stderr_tty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stdin_tty() {
        let _result = is_stdin_tty();
    }

    #[test]
    fn test_is_stdin_piped() {
        let _result = is_stdin_piped();
    }

    #[test]
    fn test_is_stderr_tty() {
        let _result = is_stderr_tty();
    }

    #[test]
    fn test_check_tty_requirements() {
        let _result = check_tty_requirements();
    }

    #[test]
    fn test_check_tty_requirements_consistency() {
        let result1 = check_tty_requirements();
        let result2 = check_tty_requirements();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_stdin_piped_inverse_of_tty() {
        assert_eq!(is_stdin_piped(), !is_stdin_tty());
    }
}
