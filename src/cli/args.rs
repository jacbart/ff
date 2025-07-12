use std::path::Path;

/// Check if any version flag is present in the arguments
pub fn has_version_flag(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--version" || arg == "-V" || arg == "-v")
}

/// Check if any multi-select flag is present in the arguments
pub fn has_multi_select_flag(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--multi-select" || arg == "-m")
}

/// Check if a string represents a file path (not a special flag)
pub fn is_file_path(arg: &str) -> bool {
    arg != "--multi-select" && arg != "--help" && arg != "-h" && Path::new(arg).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_has_version_flag_empty() {
        let args = vec![];
        assert!(!has_version_flag(&args));
    }

    #[test]
    fn test_has_version_flag_long() {
        let args = vec!["--version".to_string()];
        assert!(has_version_flag(&args));
    }

    #[test]
    fn test_has_version_flag_short_v() {
        let args = vec!["-v".to_string()];
        assert!(has_version_flag(&args));
    }

    #[test]
    fn test_has_version_flag_short_v_upper() {
        let args = vec!["-V".to_string()];
        assert!(has_version_flag(&args));
    }

    #[test]
    fn test_has_version_flag_mixed() {
        let args = vec![
            "other".to_string(),
            "--version".to_string(),
            "file.txt".to_string(),
        ];
        assert!(has_version_flag(&args));
    }

    #[test]
    fn test_has_version_flag_no_version() {
        let args = vec!["--help".to_string(), "--multi-select".to_string()];
        assert!(!has_version_flag(&args));
    }

    #[test]
    fn test_has_multi_select_flag_empty() {
        let args = vec![];
        assert!(!has_multi_select_flag(&args));
    }

    #[test]
    fn test_has_multi_select_flag_long() {
        let args = vec!["--multi-select".to_string()];
        assert!(has_multi_select_flag(&args));
    }

    #[test]
    fn test_has_multi_select_flag_short() {
        let args = vec!["-m".to_string()];
        assert!(has_multi_select_flag(&args));
    }

    #[test]
    fn test_has_multi_select_flag_mixed() {
        let args = vec![
            "other".to_string(),
            "--multi-select".to_string(),
            "file.txt".to_string(),
        ];
        assert!(has_multi_select_flag(&args));
    }

    #[test]
    fn test_has_multi_select_flag_no_multi_select() {
        let args = vec!["--help".to_string(), "--version".to_string()];
        assert!(!has_multi_select_flag(&args));
    }

    #[test]
    fn test_is_file_path_special_flags() {
        assert!(!is_file_path("--multi-select"));
        assert!(!is_file_path("--help"));
        assert!(!is_file_path("-h"));
    }

    #[test]
    fn test_is_file_path_nonexistent() {
        assert!(!is_file_path("nonexistent_file.txt"));
        assert!(!is_file_path("/nonexistent/path"));
    }

    #[test]
    fn test_is_file_path_existing_file() {
        // Create a temporary file for testing
        let temp_file = PathBuf::from("test_temp_file.txt");
        fs::write(&temp_file, "test content").unwrap();

        assert!(is_file_path("test_temp_file.txt"));

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_is_file_path_existing_directory() {
        // Create a temporary directory for testing
        let temp_dir = PathBuf::from("test_temp_dir");
        fs::create_dir(&temp_dir).unwrap();

        assert!(is_file_path("test_temp_dir"));

        // Clean up
        fs::remove_dir(&temp_dir).unwrap();
    }

    #[test]
    fn test_is_file_path_regular_string() {
        // Test that regular strings that aren't special flags are checked for existence
        assert!(!is_file_path("regular_string"));
        assert!(!is_file_path("some_file.txt"));
    }

    #[test]
    fn test_is_file_path_edge_cases() {
        assert!(!is_file_path(""));
        assert!(!is_file_path("   "));
        assert!(!is_file_path("--"));
        assert!(!is_file_path("-"));
    }
}
