//! # FF - Fast Fuzzy Finder
//! 
//! A high-performance fuzzy finder library with TUI support for Rust applications.
//! 
//! ## Features
//! 
//! - **Fast fuzzy matching** with substring and character sequence matching
//! - **Case-insensitive search** by default
//! - **Multi-select support** for selecting multiple items
//! - **TUI interface** with keyboard navigation and Gruvbox color scheme
//! - **Caching** for improved performance on repeated queries
//! - **Flexible TUI modes**: Fullscreen with borders or compact non-fullscreen mode
//! - **Configurable height**: Set specific line count or percentage of terminal
//! 
//! ## Quick Start
//! 
//! ```rust
//! use ff::FuzzyFinder;
//! 
//! let items = vec![
//!     "apple".to_string(),
//!     "banana".to_string(),
//!     "cherry".to_string(),
//! ];
//! 
//! let mut finder = FuzzyFinder::new(items, false);
//! finder.query = "app".to_string();
//! finder.update_filter();
//! 
//! assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
//! ```
//! 
//! ## TUI Usage
//! 
//! ```no_run
//! use ff::run_tui;
//! 
//! let items = vec!["item1".to_string(), "item2".to_string()];
//! match run_tui(items, false) {
//!     Ok(selected) => println!("Selected: {:?}", selected),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```
//! 
//! ## TUI with Height Configuration
//! 
//! ```no_run
//! use ff::{run_tui_with_config, TuiConfig};
//! 
//! let items = vec!["item1".to_string(), "item2".to_string()];
//! 
//! // Non-fullscreen mode with 10 lines height
//! let config = TuiConfig::with_height(10);
//! match run_tui_with_config(items.clone(), false, config) {
//!     Ok(selected) => println!("Selected: {:?}", selected),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! 
//! // Non-fullscreen mode with 50% of terminal height
//! let config = TuiConfig::with_height_percentage(50.0);
//! match run_tui_with_config(items, false, config) {
//!     Ok(selected) => println!("Selected: {:?}", selected),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```
//! 
//! ## CLI Usage
//! 
//! The library also provides a CLI binary:
//! 
//! ```bash
//! # Single select
//! echo "apple\nbanana\ncherry" | ff
//! 
//! # Multi-select
//! ff file.txt --multi-select
//! 
//! # Direct items
//! ff apple banana cherry
//! 
//! # Non-fullscreen mode with specific height
//! ff file.txt --height 10
//! 
//! # Non-fullscreen mode with percentage height
//! ff file.txt --height-percentage 50
//! 
//! # Version info
//! ff --version
//! ```

// === Internal Modules ===
pub mod config;
pub mod input;
pub mod bench;
pub mod tui;
pub mod fuzzy;
pub mod cli;

// === Public API Exports ===

/// A high-performance fuzzy finder for searching through lists of items.
/// 
/// Supports both single-select and multi-select modes, with fast fuzzy matching
/// that includes substring matching and character sequence matching.
/// 
/// # Examples
/// 
/// ```no_run
/// use ff::FuzzyFinder;
/// 
/// let items = vec![
///     "apple".to_string(),
///     "banana".to_string(),
///     "cherry".to_string(),
/// ];
/// 
/// let mut finder = FuzzyFinder::new(items, false);
/// finder.query = "app".to_string();
/// finder.update_filter();
/// 
/// assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
/// ```
pub use fuzzy::FuzzyFinder;

/// Run an interactive TUI for fuzzy finding through a list of items.
/// 
/// This function provides a full terminal user interface with:
/// - Real-time fuzzy filtering as you type
/// - Keyboard navigation (arrow keys)
/// - Single-select or multi-select modes
/// - Visual feedback for selections
/// - Fullscreen mode with borders (default)
/// - Gruvbox color scheme for consistent theming
/// 
/// # Arguments
/// 
/// - `items`: The list of items to search through
/// - `multi_select`: If `true`, allows selecting multiple items. If `false`, only single selection is allowed.
/// 
/// # Returns
/// 
/// Returns a `Result<Vec<String>, Box<dyn std::error::Error>>`:
/// - `Ok(selected_items)`: The list of selected items (empty if none selected)
/// - `Err(e)`: An error occurred during TUI operation
/// 
/// # Examples
/// 
/// ```no_run
/// use ff::run_tui;
/// 
/// let items = vec![
///     "apple".to_string(),
///     "banana".to_string(),
///     "cherry".to_string(),
/// ];
/// 
/// match run_tui(items, false) {
///     Ok(selected) => {
///         if !selected.is_empty() {
///             println!("Selected: {}", selected[0]);
///         }
///     }
///     Err(e) => eprintln!("TUI error: {}", e),
/// }
/// ```
/// 
/// # TUI Controls
/// 
/// - **Type to search**: Filter items in real-time
/// - **↑/↓ arrows**: Navigate through results
/// - **Enter**: Select item (single mode) or confirm selection (multi mode)
/// - **Tab/Space**: Toggle selection (multi-select mode only)
/// - **Esc**: Exit without selection
/// - **Ctrl+Q**: Exit without selection
pub use tui::run_tui;

/// Run an interactive TUI with custom configuration for height and display mode.
/// 
/// This function provides the same functionality as `run_tui` but allows you to configure:
/// - **Fullscreen mode**: Traditional interface with borders (default)
/// - **Non-fullscreen mode**: Compact interface without borders, search bar as input line
/// - **Height configuration**: Set specific line count or percentage of terminal height
/// - **Gruvbox color scheme**: Consistent theming across all modes
/// 
/// # Arguments
/// 
/// - `items`: The list of items to search through
/// - `multi_select`: If `true`, allows selecting multiple items. If `false`, only single selection is allowed.
/// - `config`: TUI configuration specifying height and display mode
/// 
/// # Returns
/// 
/// Returns a `Result<Vec<String>, Box<dyn std::error::Error>>`:
/// - `Ok(selected_items)`: The list of selected items (empty if none selected)
/// - `Err(e)`: An error occurred during TUI operation
/// 
/// # Examples
/// 
/// ```no_run
/// use ff::{run_tui_with_config, TuiConfig};
/// 
/// let items = vec!["apple".to_string(), "banana".to_string()];
/// 
/// // Non-fullscreen mode with 10 lines height
/// let config = TuiConfig::with_height(10);
/// match run_tui_with_config(items.clone(), false, config) {
///     Ok(selected) => println!("Selected: {:?}", selected),
///     Err(e) => eprintln!("TUI error: {}", e),
/// }
/// 
/// // Non-fullscreen mode with 50% of terminal height
/// let config = TuiConfig::with_height_percentage(50.0);
/// match run_tui_with_config(items, false, config) {
///     Ok(selected) => println!("Selected: {:?}", selected),
///     Err(e) => eprintln!("TUI error: {}", e),
/// }
/// ```
pub use tui::run_tui_with_config;

/// Configuration for TUI display mode and height.
/// 
/// This struct allows you to configure how the TUI is displayed:
/// - **Fullscreen mode**: Traditional interface with borders and full terminal usage
/// - **Non-fullscreen mode**: Compact interface without borders, with configurable height
/// 
/// # Examples
/// 
/// ```no_run
/// use ff::TuiConfig;
/// 
/// // Fullscreen mode (default)
/// let config = TuiConfig::fullscreen();
/// 
/// // Non-fullscreen mode with specific height
/// let config = TuiConfig::with_height(10);
/// 
/// // Non-fullscreen mode with percentage height
/// let config = TuiConfig::with_height_percentage(50.0);
/// ```
pub use tui::TuiConfig;

// === Public Functions ===

/// Get build information including version and build timestamp.
/// Returns a short string like: ff v0.1.0 (built: 2024-07-11)
pub fn get_build_info() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let build_timestamp = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("");
    let date = if build_timestamp.chars().all(|c| c.is_ascii_digit()) && !build_timestamp.is_empty() {
        // Parse as unix timestamp
        if let Ok(ts) = build_timestamp.parse::<i64>() {
            if let Some(dt) = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0) {
                dt.format("%Y-%m-%d").to_string()
            } else {
                build_timestamp.to_string()
            }
        } else {
            build_timestamp.to_string()
        }
    } else if build_timestamp.contains('T') {
        build_timestamp.split('T').next().unwrap_or("").to_string()
    } else {
        build_timestamp.to_string()
    };
    if date.is_empty() {
        format!("ff v{}", version)
    } else {
        format!("ff v{} (built: {})", version, date)
    }
}

pub use cli::cli_main;

// === Tests ===
#[cfg(test)]
mod tests; 