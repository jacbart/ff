use atty::Stream;

/// Check if both stdin and stdout are TTYs (required for interactive selection)
pub fn check_tty_requirements() -> bool {
    atty::is(Stream::Stdin) && atty::is(Stream::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_tty_requirements() {
        // This test checks that the function doesn't panic
        // The actual result depends on the test environment
        let result = check_tty_requirements();
        assert!(result == true || result == false); // Should be a boolean
    }

    #[test]
    fn test_check_tty_requirements_consistency() {
        // Multiple calls should return the same result
        let result1 = check_tty_requirements();
        let result2 = check_tty_requirements();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_atty_streams() {
        // Test that we can check individual streams
        let stdin_is_tty = atty::is(Stream::Stdin);
        let stdout_is_tty = atty::is(Stream::Stdout);
        
        // Both should be boolean values
        assert!(stdin_is_tty == true || stdin_is_tty == false);
        assert!(stdout_is_tty == true || stdout_is_tty == false);
        
        // The combined result should match our function
        let expected = stdin_is_tty && stdout_is_tty;
        let actual = check_tty_requirements();
        assert_eq!(expected, actual);
    }
}

 