//! # ff - Fuzzy Finder Library
//!
//! A simple, efficient fuzzy finder library and TUI for Rust applications.
//!
//! ## Features
//! - Fuzzy matching (substring and character sequence)
//! - Case-insensitive search
//! - Multi-select support
//! - TUI interface with keyboard navigation
//! - Configurable height for the TUI
//!
//! ## Quick Start
//!
//! ```rust
//! use ff::FuzzyFinder;
//!
//! let items = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
//! let mut finder = FuzzyFinder::new(items, false);
//! finder.query = "app".to_string();
//! finder.update_filter();
//! assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
//! ```
//!
//! ## TUI Usage
//!
//! ```no_run
//! use ff::run_tui;
//! let items = vec!["item1".to_string(), "item2".to_string()];
//! let _ = run_tui(items, false);
//! ```
//!
//! ## TUI with Height Configuration
//!
//! ```no_run
//! use ff::{run_tui_with_config, TuiConfig};
//! let items = vec!["item1".to_string(), "item2".to_string()];
//! let config = TuiConfig::with_height(10);
//! let _ = run_tui_with_config(items, false, config);
//! ```

// === Internal Modules ===
pub mod bench;
pub mod cli;
pub mod config;
pub mod fuzzy;
pub mod input;
pub mod tui;

// === Public API Exports ===

/// Fuzzy finder for searching through lists of items.
///
/// Supports single-select and multi-select modes, with fuzzy matching.
///
/// # Example
/// ```no_run
/// use ff::FuzzyFinder;
/// let items = vec!["apple".to_string(), "banana".to_string()];
/// let mut finder = FuzzyFinder::new(items, false);
/// finder.query = "app".to_string();
/// finder.update_filter();
/// assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
/// ```
pub use fuzzy::FuzzyFinder;

/// Run an interactive TUI for fuzzy finding through a list of items.
///
/// - Real-time fuzzy filtering as you type
/// - Keyboard navigation (arrow keys)
/// - Single-select or multi-select modes
///
/// # Arguments
/// - `items`: The list of items to search through
/// - `multi_select`: If `true`, allows selecting multiple items
///
/// # Returns
/// - `Ok(selected_items)`: The list of selected items (empty if none selected)
/// - `Err(e)`: An error occurred during TUI operation
pub use tui::run_tui;

/// Run an interactive TUI with custom configuration for height and display mode.
///
/// # Arguments
/// - `items`: The list of items to search through
/// - `multi_select`: If `true`, allows selecting multiple items
/// - `config`: TUI configuration specifying height and display mode
///
/// # Returns
/// - `Ok(selected_items)`: The list of selected items (empty if none selected)
/// - `Err(e)`: An error occurred during TUI operation
pub use tui::run_tui_with_config;

/// Configuration for TUI display mode and height.
///
/// # Example
/// ```no_run
/// use ff::TuiConfig;
/// let config = TuiConfig::with_height(10);
/// ```
pub use tui::TuiConfig;

// === Public Functions ===

/// Get build information including version and build timestamp.
/// Returns a string like: ff v0.1.0 (built: 2024-07-11)
pub fn get_build_info() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let build_timestamp = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("");
    let date = if build_timestamp.chars().all(|c| c.is_ascii_digit()) && !build_timestamp.is_empty()
    {
        // Parse as unix timestamp
        if let Ok(ts) = build_timestamp.parse::<i64>() {
            timestamp_to_date(ts)
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

/// Convert Unix timestamp to YYYY-MM-DD format
fn timestamp_to_date(timestamp: i64) -> String {
    let days_since_epoch = timestamp / 86400;
    let mut year = 1970;
    let mut days_in_year = 365;
    let mut remaining_days = days_since_epoch;
    while remaining_days >= days_in_year {
        remaining_days -= days_in_year;
        year += 1;
        days_in_year = if is_leap_year(year) { 366 } else { 365 };
    }
    let mut month = 1;
    let mut day = 1;
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, &days) in days_in_month.iter().enumerate() {
        let days_this_month = if i == 1 && is_leap_year(year) {
            29
        } else {
            days
        };
        if remaining_days >= days_this_month {
            remaining_days -= days_this_month;
            month += 1;
        } else {
            day += remaining_days as u32;
            break;
        }
    }
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub use cli::cli_main;

// === Tests ===
#[cfg(test)]
mod tests;
