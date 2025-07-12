use std::io::{stdin, stdout, IsTerminal};

/// Check if both stdin and stdout are TTYs (required for interactive selection)
pub fn check_tty_requirements() -> bool {
    stdin().is_terminal() && stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_tty_requirements() {
        // The actual result depends on the test environment
        let _result = check_tty_requirements();
        // Function should not panic
    }

    #[test]
    fn test_check_tty_requirements_consistency() {
        // Multiple calls should return the same result
        let result1 = check_tty_requirements();
        let result2 = check_tty_requirements();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_tty_streams() {
        // Test that we can check individual streams
        let stdin_is_tty = stdin().is_terminal();
        let stdout_is_tty = stdout().is_terminal();

        // Both should be boolean values
        let _stdin_is_tty = stdin_is_tty;
        let _stdout_is_tty = stdout_is_tty;

        // The combined result should match our function
        let expected = stdin_is_tty && stdout_is_tty;
        let actual = check_tty_requirements();
        assert_eq!(expected, actual);
    }
}
