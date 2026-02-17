//! # ff - Fuzzy Finder Library
//!
//! A fast, lightweight fuzzy finder library with async streaming, LSH, and quicksort.
//!
//! ## Features
//! - Async fuzzy matching with streaming
//! - Locality Sensitive Hashing (LSH) for similarity grouping
//! - Quicksort for fast result sorting
//! - Multi-select support
//! - TUI interface with keyboard navigation
//! - Configurable height for the TUI
//!
//! ## Quick Start
//!
//! ```rust
//! use ff::FuzzyFinder;
//!
//! #[tokio::main]
//! async fn main() {
//!     let items = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
//!     let mut finder = FuzzyFinder::with_items_async(items, false).await;
//!     finder.set_query("app".to_string()).await;
//!     let filtered = finder.get_filtered_items();
//!     assert_eq!(filtered.len(), 1);
//! }
//! ```

// === Internal Modules ===
pub mod cli;
pub mod config;
pub mod fuzzy;
pub mod input;
pub mod tui;

use tokio::sync::mpsc;

// === Public API Exports ===

/// Async fuzzy finder with streaming capabilities, LSH, and quicksort.
///
/// Supports async operations, locality sensitive hashing, and efficient sorting.
///
/// # Example
/// ```no_run
/// use ff::FuzzyFinder;
/// use tokio;
///
/// #[tokio::main]
/// async fn main() {
///     let items = vec!["apple".to_string(), "banana".to_string()];
///     let mut finder = FuzzyFinder::with_items_async(items, false).await;
///     finder.set_query("app".to_string()).await;
///     let filtered = finder.get_filtered_items();
/// }
/// ```
pub use fuzzy::FuzzyFinder;

/// Run an interactive TUI for fuzzy finding through an mpsc receiver of items.
///
/// - Real-time fuzzy filtering as you type
/// - Keyboard navigation (arrow keys)
/// - Single-select or multi-select modes
/// - Asynchronous item processing via mpsc channel
///
/// # Arguments
/// - `items_receiver`: The mpsc receiver for items to search through
/// - `multi_select`: If `true`, allows selecting multiple items
///
/// # Returns
/// - `Ok(selected_items)`: The list of selected items (index, content) (empty if none selected)
/// - `Err(e)`: An error occurred during TUI operation
pub use tui::run_tui;

/// Run an interactive TUI with custom configuration for height and display mode.
///
/// # Arguments
/// - `items_receiver`: The mpsc receiver for items to search through
/// - `multi_select`: If `true`, allows selecting multiple items
/// - `config`: TUI configuration specifying height and display mode
///
/// # Returns
/// - `Ok(selected_items)`: The list of selected items (index, content) (empty if none selected)
/// - `Err(e)`: An error occurred during TUI operation
pub use tui::run_tui_with_config;

/// Create an mpsc channel for sending items to the TUI.
///
/// # Returns
/// - `(sender, receiver)`: A tuple containing the sender and receiver for the channel
pub use tui::create_items_channel;

/// Configuration for TUI display mode and height.
///
/// # Example
/// ```no_run
/// use ff::TuiConfig;
/// let config = TuiConfig::with_height(10);
/// ```
pub use tui::TuiConfig;

/// Per-item indicator that can be displayed alongside items.
///
/// # Example
/// ```no_run
/// use ff::ItemIndicator;
/// let indicator = ItemIndicator::Spinner;
/// let success = ItemIndicator::Success;
/// let custom = ItemIndicator::Text("*".to_string());
/// ```
pub use tui::ItemIndicator;

/// Global status indicator for the TUI prompt line.
///
/// # Example
/// ```no_run
/// use ff::GlobalStatus;
/// let loading = GlobalStatus::Loading(Some("Searching...".to_string()));
/// let ready = GlobalStatus::Ready(Some("Done".to_string()));
/// ```
pub use tui::GlobalStatus;

/// Commands that can be sent to update the TUI state dynamically.
///
/// # Example
/// ```no_run
/// use ff::{TuiCommand, ItemIndicator};
/// let cmd = TuiCommand::AddItem("item".to_string());
/// let cmd_with_indicator = TuiCommand::AddItemWithIndicator("item".to_string(), ItemIndicator::Spinner);
/// let update = TuiCommand::UpdateIndicator("item".to_string(), ItemIndicator::Success);
/// ```
pub use tui::TuiCommand;

/// Create an mpsc channel for sending commands (items with indicators) to the TUI.
pub use tui::create_command_channel;

/// Run an interactive TUI with command channel support for per-item indicators.
///
/// This is the extended version that supports dynamic per-item indicators.
///
/// # Arguments
/// - `command_receiver`: The mpsc receiver for TuiCommand messages
/// - `multi_select`: If `true`, allows selecting multiple items
/// - `config`: TUI configuration specifying height and display mode
///
/// # Returns
/// - `Ok(selected_items)`: The list of selected items (index, content) (empty if none selected)
/// - `Err(e)`: An error occurred during TUI operation
pub use tui::run_tui_with_indicators;

/// A session handle for the fuzzy finder, allowing asynchronous item ingestion.
///
/// This struct provides a high-level interface to the fuzzy finder TUI,
/// allowing you to push items asynchronously while the TUI is running.
///
/// # Example
/// ```no_run
/// use ff::FuzzyFinderSession;
/// use tokio;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     // Start the session
///     let (session, tui_future) = FuzzyFinderSession::new(true);
///
///     // Spawn the TUI runner
///     let runner = tokio::spawn(tui_future);
///
///     // Push items asynchronously
///     session.add("apple").await?;
///     session.add("banana").await?;
///
///     // Wait for result
///     let result = runner.await??;
///     Ok(())
/// }
/// ```
pub struct FuzzyFinderSession {
    sender: mpsc::Sender<String>,
}

impl FuzzyFinderSession {
    /// Start a new fuzzy finder session with default configuration.
    ///
    /// Returns a tuple containing:
    /// 1. The session handle (for adding items)
    /// 2. A future that runs the TUI (must be awaited or spawned)
    pub fn new(
        multi_select: bool,
    ) -> (
        Self,
        impl std::future::Future<Output = Result<Vec<(usize, String)>, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        Self::with_config(multi_select, TuiConfig::default())
    }

    /// Start a new session with custom configuration.
    pub fn with_config(
        multi_select: bool,
        config: TuiConfig,
    ) -> (
        Self,
        impl std::future::Future<Output = Result<Vec<(usize, String)>, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        let (sender, receiver) = tui::create_items_channel();
        (
            Self { sender },
            tui::run_tui_with_config(receiver, multi_select, config),
        )
    }

    /// Add a single item to the finder.
    pub async fn add(&self, item: impl Into<String>) -> Result<(), mpsc::error::SendError<String>> {
        self.sender.send(item.into()).await
    }

    /// Add multiple items to the finder.
    pub async fn add_batch<I>(&self, items: I) -> Result<(), mpsc::error::SendError<String>>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        for item in items {
            self.sender.send(item.into()).await?;
        }
        Ok(())
    }
}

/// A session handle for the fuzzy finder with per-item indicator support.
///
/// This struct provides a high-level interface to the fuzzy finder TUI,
/// allowing you to push items with dynamic indicators.
///
/// # Example
/// ```no_run
/// use ff::{FuzzyFinderWithIndicators, ItemIndicator};
/// use tokio;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     // Start the session
///     let (session, tui_future) = FuzzyFinderWithIndicators::new(true);
///
///     // Spawn the TUI runner
///     let runner = tokio::spawn(tui_future);
///
///     // Add items with indicators
///     session.add_with_indicator("task1", ItemIndicator::Spinner).await?;
///     session.add("task2").await?;
///     
///     // Update indicator later
///     session.set_indicator("task1", ItemIndicator::Success).await?;
///
///     // Wait for result
///     let result = runner.await??;
///     Ok(())
/// }
/// ```
pub struct FuzzyFinderWithIndicators {
    sender: mpsc::Sender<TuiCommand>,
}

impl FuzzyFinderWithIndicators {
    /// Start a new fuzzy finder session with indicator support.
    ///
    /// Returns a tuple containing:
    /// 1. The session handle (for adding items and updating indicators)
    /// 2. A future that runs the TUI (must be awaited or spawned)
    pub fn new(
        multi_select: bool,
    ) -> (
        Self,
        impl std::future::Future<Output = Result<Vec<(usize, String)>, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        Self::with_config(multi_select, TuiConfig::default())
    }

    /// Start a new session with custom configuration.
    pub fn with_config(
        multi_select: bool,
        config: TuiConfig,
    ) -> (
        Self,
        impl std::future::Future<Output = Result<Vec<(usize, String)>, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        let (sender, receiver) = tui::create_command_channel();
        (
            Self { sender },
            tui::run_tui_with_indicators(receiver, multi_select, config),
        )
    }

    /// Add a single item to the finder (no indicator).
    pub async fn add(
        &self,
        item: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<TuiCommand>> {
        self.sender.send(TuiCommand::AddItem(item.into())).await
    }

    /// Add a single item with an indicator.
    pub async fn add_with_indicator(
        &self,
        item: impl Into<String>,
        indicator: ItemIndicator,
    ) -> Result<(), mpsc::error::SendError<TuiCommand>> {
        self.sender
            .send(TuiCommand::AddItemWithIndicator(item.into(), indicator))
            .await
    }

    /// Update the indicator for an existing item.
    pub async fn set_indicator(
        &self,
        item: impl Into<String>,
        indicator: ItemIndicator,
    ) -> Result<(), mpsc::error::SendError<TuiCommand>> {
        self.sender
            .send(TuiCommand::UpdateIndicator(item.into(), indicator))
            .await
    }

    /// Clear the indicator for an item.
    pub async fn clear_indicator(
        &self,
        item: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<TuiCommand>> {
        self.sender
            .send(TuiCommand::UpdateIndicator(
                item.into(),
                ItemIndicator::None,
            ))
            .await
    }

    /// Set the global status indicator.
    pub async fn set_global_status(
        &self,
        status: GlobalStatus,
    ) -> Result<(), mpsc::error::SendError<TuiCommand>> {
        self.sender.send(TuiCommand::SetGlobalStatus(status)).await
    }

    /// Add multiple items to the finder.
    pub async fn add_batch<I>(&self, items: I) -> Result<(), mpsc::error::SendError<TuiCommand>>
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        for item in items {
            self.sender.send(TuiCommand::AddItem(item.into())).await?;
        }
        Ok(())
    }
}

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
        format!("ff v{version}")
    } else {
        format!("ff v{version} (built: {date})")
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
    format!("{year:04}-{month:02}-{day:02}")
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub use cli::cli_main;

// === Tests ===
#[cfg(test)]
mod tests;
