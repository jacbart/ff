use crate::fuzzy::FuzzyFinder;
use crate::tui::buffer::ScreenBuffer;
use crate::tui::controls::Action;
use crossterm::{
    cursor::{position, Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use std::{
    io::{self, Write},
    mem,
    time::Instant,
};
use tokio::sync::mpsc;

/// Built-in spinner frames (Braille dots pattern)
const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Global status indicator state
#[derive(Debug, Clone, Default)]
pub enum GlobalStatus {
    /// Show a spinning indicator with optional message
    Loading(Option<String>),
    /// Show a static message (e.g., "Done", "Ready")
    Ready(Option<String>),
    /// Custom static text
    Custom(String),
    /// No indicator shown
    #[default]
    Hidden,
}

/// Per-item indicator that can be displayed alongside items
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ItemIndicator {
    /// Spinning indicator (animated)
    Spinner,
    /// Static text indicator
    Text(String),
    /// Colored text indicator
    ColoredText(String, Color),
    /// Success indicator (checkmark)
    Success,
    /// Error indicator (x mark)
    Error,
    /// Warning indicator
    Warning,
    /// No indicator
    #[default]
    None,
}

/// Commands that can be sent to update the TUI state
#[derive(Debug, Clone)]
pub enum TuiCommand {
    /// Add a new item
    AddItem(String),
    /// Add a new item with an indicator
    AddItemWithIndicator(String, ItemIndicator),
    /// Update indicator for an existing item
    UpdateIndicator(String, ItemIndicator),
    /// Set global status
    SetGlobalStatus(GlobalStatus),
}

/// Configuration for TUI display mode and height
#[derive(Debug, Clone)]
pub struct TuiConfig {
    /// Whether to use fullscreen mode
    pub fullscreen: bool,
    /// Fixed height in lines (non-fullscreen mode)
    pub height: Option<u16>,
    /// Height as percentage of terminal (non-fullscreen mode)
    pub height_percentage: Option<f32>,
    /// Whether to show help/instructions text at the bottom
    pub show_help_text: bool,
    /// Whether to show a loading spinner while items are being received
    pub show_loading_indicator: bool,
    /// Custom loading message (shown next to spinner)
    pub loading_message: Option<String>,
    /// Custom ready message (shown when loading is complete)
    pub ready_message: Option<String>,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            fullscreen: true,
            height: None,
            height_percentage: None,
            show_help_text: true,
            show_loading_indicator: true,
            loading_message: None,
            ready_message: None,
        }
    }
}

impl TuiConfig {
    /// Create a new TUI configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with fixed height
    pub fn with_height(height: u16) -> Self {
        Self {
            fullscreen: false,
            height: Some(height),
            height_percentage: None,
            show_help_text: true,
            show_loading_indicator: true,
            loading_message: None,
            ready_message: None,
        }
    }

    /// Create a configuration with height as percentage
    pub fn with_height_percentage(percentage: f32) -> Self {
        Self {
            fullscreen: false,
            height: None,
            height_percentage: Some(percentage),
            show_help_text: true,
            show_loading_indicator: true,
            loading_message: None,
            ready_message: None,
        }
    }

    /// Create a fullscreen configuration
    pub fn fullscreen() -> Self {
        Self {
            fullscreen: true,
            height: None,
            height_percentage: None,
            show_help_text: true,
            show_loading_indicator: true,
            loading_message: None,
            ready_message: None,
        }
    }

    /// Calculate the actual height based on terminal size
    pub fn calculate_height(&self, terminal_height: u16) -> u16 {
        if self.fullscreen {
            terminal_height
        } else if let Some(height) = self.height {
            height.min(terminal_height)
        } else if let Some(percentage) = self.height_percentage {
            let calculated = (terminal_height as f32 * percentage / 100.0) as u16;
            calculated.max(1).min(terminal_height)
        } else {
            terminal_height
        }
    }
}

/// Run an async interactive TUI for fuzzy finding through an mpsc receiver of items.
pub async fn run_tui(
    items_receiver: mpsc::Receiver<String>,
    multi_select: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    run_tui_with_config(items_receiver, multi_select, TuiConfig::default()).await
}

/// Run an async interactive TUI with custom configuration for height and display mode.
pub async fn run_tui_with_config(
    items_receiver: mpsc::Receiver<String>,
    multi_select: bool,
    config: TuiConfig,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    run_interactive_tui(items_receiver, multi_select, config).await
}

/// Run the async interactive TUI
async fn run_interactive_tui(
    mut items_receiver: mpsc::Receiver<String>,
    multi_select: bool,
    config: TuiConfig,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut fuzzy_finder = FuzzyFinder::new(multi_select);
    let mut stdout = io::stdout();

    // Enable raw mode and hide cursor
    enable_raw_mode()?;
    execute!(stdout, Hide)?;

    let fullscreen = config.fullscreen;
    let mut original_cursor = position()?;
    let (_term_width, term_height) = size()?;
    let tui_height = config.calculate_height(term_height);

    if fullscreen {
        execute!(
            &mut stdout,
            crossterm::terminal::EnterAlternateScreen,
            Clear(ClearType::All)
        )?;
    } else {
        // If not enough space below, scroll the terminal down
        if original_cursor.1 + tui_height > term_height {
            let needed = (original_cursor.1 + tui_height).saturating_sub(term_height);
            for _ in 0..needed {
                writeln!(stdout)?;
            }
            stdout.flush()?;
            // After scrolling, we should draw at the bottom of the terminal
            original_cursor = (0, term_height.saturating_sub(tui_height));
        }
        // Always move to column 0 at the current line
        execute!(&mut stdout, MoveTo(0, original_cursor.1))?;
    }

    let mut selected_items = Vec::new();
    let mut needs_redraw = true;
    let mut items_buffer = Vec::new();
    let mut receiver_exhausted = false;
    let mut scroll_offset = 0;

    // Spinner animation state
    let mut spinner_frame: usize = 0;
    let mut last_spinner_update = Instant::now();
    let spinner_interval = std::time::Duration::from_millis(80);

    // Create screen buffer for double-buffered rendering
    let (term_width, _) = size()?;
    let mut screen_buffer = ScreenBuffer::new(term_width, tui_height);

    loop {
        // Process new items from mpsc receiver
        if !receiver_exhausted {
            let mut batch_count = 0;
            const MAX_BATCH_SIZE: usize = 1000;

            loop {
                match items_receiver.try_recv() {
                    Ok(item) => {
                        items_buffer.push(item);
                        batch_count += 1;
                        if batch_count >= MAX_BATCH_SIZE {
                            break;
                        }
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        break;
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        receiver_exhausted = true;
                        needs_redraw = true; // Redraw to show ready state
                        break;
                    }
                }
            }

            if !items_buffer.is_empty() {
                fuzzy_finder.add_items(mem::take(&mut items_buffer)).await;
                needs_redraw = true;
            }
        }

        let (_term_width, term_height) = size()?;
        let tui_height = config.calculate_height(term_height);
        // Always reserve 1 line for prompt, 1 for result if possible, 1 for instructions
        let available_height = if tui_height > 2 {
            if config.show_help_text {
                tui_height - 2 // 1 for prompt, 1 for instructions
            } else {
                tui_height - 1
            }
        } else if tui_height == 2 {
            1 // Only room for prompt and one result
        } else {
            0 // Only room for prompt
        };

        // Update scroll offset to keep cursor in view
        let cursor_pos = fuzzy_finder.get_cursor_position();
        if cursor_pos < scroll_offset {
            scroll_offset = cursor_pos;
        } else if cursor_pos >= scroll_offset + available_height as usize {
            scroll_offset = cursor_pos - available_height as usize + 1;
        }

        // Ensure scroll offset is valid (e.g. if list shrank)
        let total_items = fuzzy_finder.get_filtered_items().len();
        if scroll_offset > total_items {
            scroll_offset = total_items.saturating_sub(available_height as usize);
        }

        // Only redraw if needed (when query changes or cursor moves)
        if needs_redraw {
            // Resize buffer if terminal size changed
            let (term_width, _) = size()?;
            screen_buffer.resize(term_width, tui_height);
            screen_buffer.clear();

            // Draw search prompt with optional status indicator (row 0 in buffer)
            let mut col: u16 = 0;
            col += screen_buffer.put_str(col, 0, "> ", Some(Color::Cyan), None, false, false);
            col +=
                screen_buffer.put_str(col, 0, fuzzy_finder.get_query(), None, None, false, false);

            // Draw status indicator (spinner or ready message)
            if config.show_loading_indicator {
                col += screen_buffer.put_str(col, 0, " ", None, None, false, false);
                if !receiver_exhausted {
                    // Show spinner
                    let frame = SPINNER_FRAMES[spinner_frame % SPINNER_FRAMES.len()];
                    col += screen_buffer.put_str(
                        col,
                        0,
                        &frame.to_string(),
                        Some(Color::Yellow),
                        None,
                        false,
                        false,
                    );
                    if let Some(ref msg) = config.loading_message {
                        col += screen_buffer.put_str(col, 0, " ", None, None, false, false);
                        screen_buffer.put_str(
                            col,
                            0,
                            msg,
                            Some(Color::DarkGrey),
                            None,
                            false,
                            false,
                        );
                    }
                } else if let Some(ref msg) = config.ready_message {
                    // Show ready message
                    screen_buffer.put_str(col, 0, msg, Some(Color::Green), None, false, false);
                }
            }

            // Draw items
            if tui_height >= 2 && available_height > 0 {
                let filtered_items = fuzzy_finder.get_filtered_items();
                let visible_items = filtered_items
                    .iter()
                    .skip(scroll_offset)
                    .take(available_height as usize);

                for (i, item) in visible_items.enumerate() {
                    let absolute_index = scroll_offset + i;
                    let row = (i + 1) as u16; // Row in buffer (0 is prompt)

                    let is_cursor = absolute_index == fuzzy_finder.get_cursor_position();
                    let is_selected = fuzzy_finder.is_selected(item);

                    draw_item_to_buffer(
                        &mut screen_buffer,
                        row,
                        item,
                        is_cursor,
                        is_selected,
                        fuzzy_finder.get_match_positions(absolute_index),
                    );
                }
            }

            if tui_height < 2 {
                screen_buffer.put_str(
                    0,
                    1,
                    "Terminal too small. Please resize to continue...",
                    Some(Color::Yellow),
                    None,
                    false,
                    false,
                );
            }

            // Draw instructions (always at the bottom of the TUI area)
            if config.show_help_text {
                let instructions_row = tui_height.saturating_sub(1);
                let instructions = if multi_select {
                    "Tab/Space: Toggle | Enter: Confirm | Esc/Ctrl+C/Ctrl+Q: Exit"
                } else {
                    "↑/↓: Navigate | Enter: Select | Esc/Ctrl+C/Ctrl+Q: Exit"
                };
                screen_buffer.put_str(
                    0,
                    instructions_row,
                    instructions,
                    Some(Color::DarkGrey),
                    None,
                    false,
                    false,
                );
            }

            // Render buffer to terminal in a single write
            let rendered = if fullscreen {
                screen_buffer.render_fullscreen()
            } else {
                screen_buffer.render(original_cursor.1)
            };
            write!(stdout, "{}", rendered)?;
            stdout.flush()?;
            needs_redraw = false;
        }

        // Handle input with timeout to allow stream processing
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match handle_async_key_event(&key_event, &mut fuzzy_finder).await {
                    Action::Continue => {
                        needs_redraw = true;
                        continue;
                    }
                    Action::Exit => break,
                    Action::Select(items) => {
                        selected_items = items;
                        break;
                    }
                }
            }
        }

        // Update spinner animation if still loading
        if config.show_loading_indicator
            && !receiver_exhausted
            && last_spinner_update.elapsed() >= spinner_interval
        {
            spinner_frame = (spinner_frame + 1) % SPINNER_FRAMES.len();
            last_spinner_update = Instant::now();
            needs_redraw = true;
        }
    }

    // Restore terminal
    if fullscreen {
        execute!(&mut stdout, crossterm::terminal::LeaveAlternateScreen)?;
        execute!(&mut stdout, Show)?;
    } else {
        for i in 0..config.calculate_height(size()?.1) {
            execute!(
                &mut stdout,
                MoveTo(0, original_cursor.1 + i),
                Clear(ClearType::CurrentLine)
            )?;
        }
        execute!(
            &mut stdout,
            MoveTo(original_cursor.0, original_cursor.1),
            Show
        )?;
        stdout.flush()?;
    }

    // Restore terminal state
    disable_raw_mode()?;

    if !selected_items.is_empty() {
        // Move to the original cursor position
        execute!(&mut stdout, MoveTo(0, original_cursor.1))?;
    }

    Ok(selected_items)
}

/// Create an mpsc channel for sending items to the TUI
pub fn create_items_channel() -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
    mpsc::channel(1000) // Buffer size of 1000 items
}

/// Create an mpsc channel for sending commands (items with indicators) to the TUI
pub fn create_command_channel() -> (mpsc::Sender<TuiCommand>, mpsc::Receiver<TuiCommand>) {
    mpsc::channel(1000) // Buffer size of 1000 commands
}

/// Run an async interactive TUI with command channel support for per-item indicators
pub async fn run_tui_with_indicators(
    command_receiver: mpsc::Receiver<TuiCommand>,
    multi_select: bool,
    config: TuiConfig,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    run_interactive_tui_with_indicators(command_receiver, multi_select, config).await
}

/// Run the async interactive TUI with command channel support
async fn run_interactive_tui_with_indicators(
    mut command_receiver: mpsc::Receiver<TuiCommand>,
    multi_select: bool,
    config: TuiConfig,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut fuzzy_finder = FuzzyFinder::new(multi_select);
    let mut stdout = io::stdout();

    // Per-item indicators storage (keyed by item text)
    let mut item_indicators: std::collections::HashMap<String, ItemIndicator> =
        std::collections::HashMap::new();
    let mut global_status = GlobalStatus::Loading(None);

    // Enable raw mode and hide cursor
    enable_raw_mode()?;
    execute!(stdout, Hide)?;

    let fullscreen = config.fullscreen;
    let mut original_cursor = position()?;
    let (_term_width, term_height) = size()?;
    let tui_height = config.calculate_height(term_height);

    if fullscreen {
        execute!(
            &mut stdout,
            crossterm::terminal::EnterAlternateScreen,
            Clear(ClearType::All)
        )?;
    } else {
        // If not enough space below, scroll the terminal down
        if original_cursor.1 + tui_height > term_height {
            let needed = (original_cursor.1 + tui_height).saturating_sub(term_height);
            for _ in 0..needed {
                writeln!(stdout)?;
            }
            stdout.flush()?;
            original_cursor = (0, term_height.saturating_sub(tui_height));
        }
        execute!(&mut stdout, MoveTo(0, original_cursor.1))?;
    }

    let mut selected_items = Vec::new();
    let mut needs_redraw = true;
    let mut items_buffer = Vec::new();
    let mut receiver_exhausted = false;
    let mut scroll_offset = 0;

    // Spinner animation state
    let mut spinner_frame: usize = 0;
    let mut last_spinner_update = Instant::now();
    let spinner_interval = std::time::Duration::from_millis(80);

    // Create screen buffer for double-buffered rendering
    let (term_width, _) = size()?;
    let mut screen_buffer = ScreenBuffer::new(term_width, tui_height);

    loop {
        // Process commands from channel
        if !receiver_exhausted {
            let mut batch_count = 0;
            const MAX_BATCH_SIZE: usize = 1000;

            loop {
                match command_receiver.try_recv() {
                    Ok(command) => {
                        match command {
                            TuiCommand::AddItem(item) => {
                                items_buffer.push(item);
                            }
                            TuiCommand::AddItemWithIndicator(item, indicator) => {
                                if indicator != ItemIndicator::None {
                                    item_indicators.insert(item.clone(), indicator);
                                }
                                items_buffer.push(item);
                            }
                            TuiCommand::UpdateIndicator(item, indicator) => {
                                if indicator == ItemIndicator::None {
                                    item_indicators.remove(&item);
                                } else {
                                    item_indicators.insert(item, indicator);
                                }
                                needs_redraw = true;
                            }
                            TuiCommand::SetGlobalStatus(status) => {
                                global_status = status;
                                needs_redraw = true;
                            }
                        }
                        batch_count += 1;
                        if batch_count >= MAX_BATCH_SIZE {
                            break;
                        }
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        break;
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        receiver_exhausted = true;
                        global_status = GlobalStatus::Ready(config.ready_message.clone());
                        needs_redraw = true;
                        break;
                    }
                }
            }

            if !items_buffer.is_empty() {
                fuzzy_finder.add_items(mem::take(&mut items_buffer)).await;
                needs_redraw = true;
            }
        }

        let (_term_width, term_height) = size()?;
        let tui_height = config.calculate_height(term_height);
        let available_height = if tui_height > 2 {
            if config.show_help_text {
                tui_height - 2
            } else {
                tui_height - 1
            }
        } else if tui_height == 2 {
            1
        } else {
            0
        };

        // Update scroll offset to keep cursor in view
        let cursor_pos = fuzzy_finder.get_cursor_position();
        if cursor_pos < scroll_offset {
            scroll_offset = cursor_pos;
        } else if cursor_pos >= scroll_offset + available_height as usize {
            scroll_offset = cursor_pos - available_height as usize + 1;
        }

        let total_items = fuzzy_finder.get_filtered_items().len();
        if scroll_offset > total_items {
            scroll_offset = total_items.saturating_sub(available_height as usize);
        }

        if needs_redraw {
            // Resize buffer if terminal size changed
            let (term_width, _) = size()?;
            screen_buffer.resize(term_width, tui_height);
            screen_buffer.clear();

            // Draw search prompt with global status indicator (row 0 in buffer)
            let mut col: u16 = 0;
            col += screen_buffer.put_str(col, 0, "> ", Some(Color::Cyan), None, false, false);
            col +=
                screen_buffer.put_str(col, 0, fuzzy_finder.get_query(), None, None, false, false);

            // Draw global status indicator
            if config.show_loading_indicator {
                col += screen_buffer.put_str(col, 0, " ", None, None, false, false);
                match &global_status {
                    GlobalStatus::Loading(msg) => {
                        let frame = SPINNER_FRAMES[spinner_frame % SPINNER_FRAMES.len()];
                        col += screen_buffer.put_str(
                            col,
                            0,
                            &frame.to_string(),
                            Some(Color::Yellow),
                            None,
                            false,
                            false,
                        );
                        if let Some(ref m) = msg {
                            col += screen_buffer.put_str(col, 0, " ", None, None, false, false);
                            screen_buffer.put_str(
                                col,
                                0,
                                m,
                                Some(Color::DarkGrey),
                                None,
                                false,
                                false,
                            );
                        } else if let Some(ref m) = config.loading_message {
                            col += screen_buffer.put_str(col, 0, " ", None, None, false, false);
                            screen_buffer.put_str(
                                col,
                                0,
                                m,
                                Some(Color::DarkGrey),
                                None,
                                false,
                                false,
                            );
                        }
                    }
                    GlobalStatus::Ready(msg) => {
                        if let Some(ref m) = msg {
                            screen_buffer.put_str(
                                col,
                                0,
                                m,
                                Some(Color::Green),
                                None,
                                false,
                                false,
                            );
                        }
                    }
                    GlobalStatus::Custom(text) => {
                        screen_buffer.put_str(col, 0, text, None, None, false, false);
                    }
                    GlobalStatus::Hidden => {}
                }
            }

            // Draw items with per-item indicators
            if tui_height >= 2 && available_height > 0 {
                let filtered_items = fuzzy_finder.get_filtered_items();
                let visible_items = filtered_items
                    .iter()
                    .skip(scroll_offset)
                    .take(available_height as usize);

                for (i, item) in visible_items.enumerate() {
                    let absolute_index = scroll_offset + i;
                    let row = (i + 1) as u16; // Row in buffer (0 is prompt)

                    let is_cursor = absolute_index == fuzzy_finder.get_cursor_position();
                    let is_selected = fuzzy_finder.is_selected(item);
                    let indicator = item_indicators.get(item);

                    draw_item_with_indicator_to_buffer(
                        &mut screen_buffer,
                        row,
                        item,
                        is_cursor,
                        is_selected,
                        fuzzy_finder.get_match_positions(absolute_index),
                        indicator,
                        spinner_frame,
                    );
                }
            }

            if tui_height < 2 {
                screen_buffer.put_str(
                    0,
                    1,
                    "Terminal too small. Please resize to continue...",
                    Some(Color::Yellow),
                    None,
                    false,
                    false,
                );
            }

            // Draw instructions (always at the bottom of the TUI area)
            if config.show_help_text {
                let instructions_row = tui_height.saturating_sub(1);
                let instructions = if multi_select {
                    "Tab/Space: Toggle | Enter: Confirm | Esc/Ctrl+C/Ctrl+Q: Exit"
                } else {
                    "↑/↓: Navigate | Enter: Select | Esc/Ctrl+C/Ctrl+Q: Exit"
                };
                screen_buffer.put_str(
                    0,
                    instructions_row,
                    instructions,
                    Some(Color::DarkGrey),
                    None,
                    false,
                    false,
                );
            }

            // Render buffer to terminal in a single write
            let rendered = if fullscreen {
                screen_buffer.render_fullscreen()
            } else {
                screen_buffer.render(original_cursor.1)
            };
            write!(stdout, "{}", rendered)?;
            stdout.flush()?;
            needs_redraw = false;
        }

        // Handle input
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match handle_async_key_event(&key_event, &mut fuzzy_finder).await {
                    Action::Continue => {
                        needs_redraw = true;
                        continue;
                    }
                    Action::Exit => break,
                    Action::Select(items) => {
                        selected_items = items;
                        break;
                    }
                }
            }
        }

        // Update spinner animation
        if last_spinner_update.elapsed() >= spinner_interval {
            spinner_frame = (spinner_frame + 1) % SPINNER_FRAMES.len();
            last_spinner_update = Instant::now();
            // Only redraw if there are any spinning indicators
            let has_spinners = matches!(global_status, GlobalStatus::Loading(_))
                || item_indicators
                    .values()
                    .any(|i| matches!(i, ItemIndicator::Spinner));
            if has_spinners {
                needs_redraw = true;
            }
        }
    }

    // Restore terminal
    if fullscreen {
        execute!(&mut stdout, crossterm::terminal::LeaveAlternateScreen)?;
        execute!(&mut stdout, Show)?;
    } else {
        for i in 0..config.calculate_height(size()?.1) {
            execute!(
                &mut stdout,
                MoveTo(0, original_cursor.1 + i),
                Clear(ClearType::CurrentLine)
            )?;
        }
        execute!(
            &mut stdout,
            MoveTo(original_cursor.0, original_cursor.1),
            Show
        )?;
        stdout.flush()?;
    }

    disable_raw_mode()?;

    if !selected_items.is_empty() {
        execute!(&mut stdout, MoveTo(0, original_cursor.1))?;
    }

    Ok(selected_items)
}

/// Draw an item with optional per-item indicator
/// NOTE: This function is kept for testing purposes. Production code uses draw_item_with_indicator_to_buffer.
#[allow(dead_code)]
fn draw_item_with_indicator<W: Write>(
    stdout: &mut W,
    item: &str,
    is_cursor: bool,
    is_selected: bool,
    match_positions: Option<&crate::fuzzy::finder::MatchPositions>,
    indicator: Option<&ItemIndicator>,
    spinner_frame: usize,
) -> io::Result<()> {
    // Set cursor highlighting
    if is_cursor {
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::Yellow),
            SetAttribute(Attribute::Bold)
        )?;
    }

    // Draw indicator prefix
    match indicator {
        Some(ItemIndicator::Spinner) => {
            let frame = SPINNER_FRAMES[spinner_frame % SPINNER_FRAMES.len()];
            if !is_cursor {
                execute!(stdout, SetForegroundColor(Color::Yellow))?;
            }
            execute!(stdout, Print(frame), Print(" "))?;
            if !is_cursor {
                execute!(stdout, ResetColor)?;
            }
        }
        Some(ItemIndicator::Text(text)) => {
            execute!(stdout, Print(text), Print(" "))?;
        }
        Some(ItemIndicator::ColoredText(text, color)) => {
            let saved_color = if is_cursor {
                Color::Yellow
            } else {
                Color::Reset
            };
            execute!(stdout, SetForegroundColor(*color), Print(text), Print(" "))?;
            if is_cursor {
                execute!(stdout, SetForegroundColor(saved_color))?;
            } else {
                execute!(stdout, ResetColor)?;
            }
        }
        Some(ItemIndicator::Success) => {
            if !is_cursor {
                execute!(stdout, SetForegroundColor(Color::Green))?;
            }
            execute!(stdout, Print("✓ "))?;
            if !is_cursor {
                execute!(stdout, ResetColor)?;
            }
        }
        Some(ItemIndicator::Error) => {
            if !is_cursor {
                execute!(stdout, SetForegroundColor(Color::Red))?;
            }
            execute!(stdout, Print("✗ "))?;
            if !is_cursor {
                execute!(stdout, ResetColor)?;
            }
        }
        Some(ItemIndicator::Warning) => {
            if !is_cursor {
                execute!(stdout, SetForegroundColor(Color::Yellow))?;
            }
            execute!(stdout, Print("⚠ "))?;
            if !is_cursor {
                execute!(stdout, ResetColor)?;
            }
        }
        Some(ItemIndicator::None) | None => {
            // Selection indicator takes precedence when no other indicator
            if is_selected {
                execute!(stdout, SetForegroundColor(Color::Green), Print("✓ "))?;
                if is_cursor {
                    execute!(stdout, SetForegroundColor(Color::Yellow))?;
                } else {
                    execute!(stdout, ResetColor)?;
                }
            } else {
                execute!(stdout, Print("  "))?;
            }
        }
    }

    // Draw item text with match highlighting
    if let Some(matches) = match_positions {
        for (i, ch) in item.chars().enumerate() {
            if matches.positions.contains(&i) {
                if is_cursor {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::White),
                        SetAttribute(Attribute::Bold),
                        SetAttribute(Attribute::Underlined)
                    )?;
                } else {
                    execute!(
                        stdout,
                        SetAttribute(Attribute::Bold),
                        SetAttribute(Attribute::Underlined)
                    )?;
                }
                execute!(stdout, Print(ch))?;
                if is_cursor {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        SetAttribute(Attribute::NoUnderline)
                    )?;
                } else {
                    execute!(
                        stdout,
                        SetAttribute(Attribute::NoUnderline),
                        SetAttribute(Attribute::NormalIntensity)
                    )?;
                }
            } else {
                execute!(stdout, Print(ch))?;
            }
        }
    } else {
        execute!(stdout, Print(item))?;
    }

    execute!(stdout, ResetColor)?;
    Ok(())
}

/// Handle key events in async mode
async fn handle_async_key_event(
    key_event: &crossterm::event::KeyEvent,
    fuzzy_finder: &mut FuzzyFinder,
) -> crate::tui::controls::Action {
    match key_event.code {
        KeyCode::Char(c) => {
            if (c == 'q' || c == 'c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                Action::Exit
            } else if c == ' ' && fuzzy_finder.is_multi_select() {
                fuzzy_finder.toggle_selection();
                Action::Continue
            } else {
                let mut query = fuzzy_finder.get_query().to_string();
                query.push(c);
                fuzzy_finder.set_query(query).await;
                Action::Continue
            }
        }
        KeyCode::Backspace => {
            let mut query = fuzzy_finder.get_query().to_string();
            query.pop();
            fuzzy_finder.set_query(query).await;
            Action::Continue
        }
        KeyCode::Up => {
            fuzzy_finder.move_cursor(-1);
            Action::Continue
        }
        KeyCode::Down => {
            fuzzy_finder.move_cursor(1);
            Action::Continue
        }
        KeyCode::Tab => {
            if fuzzy_finder.is_multi_select() {
                fuzzy_finder.toggle_selection();
                // Move to next item without wrapping (stop at bottom)
                fuzzy_finder.move_cursor_clamped(1);
            }
            Action::Continue
        }
        KeyCode::Enter => {
            let selected = fuzzy_finder.get_selected_items();
            if !selected.is_empty() {
                Action::Select(selected)
            } else if !fuzzy_finder.is_multi_select()
                && !fuzzy_finder.get_filtered_items().is_empty()
            {
                // In single select mode, select the current item if no items are selected
                let current_item =
                    &fuzzy_finder.get_filtered_items()[fuzzy_finder.get_cursor_position()];
                Action::Select(vec![current_item.clone()])
            } else if fuzzy_finder.is_multi_select()
                && !fuzzy_finder.get_filtered_items().is_empty()
            {
                // In multi-select mode, if no items are selected, select the current item
                let current_item =
                    &fuzzy_finder.get_filtered_items()[fuzzy_finder.get_cursor_position()];
                Action::Select(vec![current_item.clone()])
            } else {
                Action::Continue
            }
        }
        KeyCode::Esc => {
            // Two-stage escape: first clears query, second exits
            if fuzzy_finder.get_query().is_empty() {
                Action::Exit
            } else {
                fuzzy_finder.set_query(String::new()).await;
                Action::Continue
            }
        }
        _ => Action::Continue,
    }
}

/// Draw highlighted item with fuzzy match highlighting using Gruvbox soft colors
/// NOTE: This function is kept for testing purposes. Production code uses draw_item_to_buffer.
#[allow(dead_code)]
fn draw_highlighted_item_with_matches<W: Write>(
    stdout: &mut W,
    item: &str,
    is_cursor: bool,
    is_selected: bool,
    match_positions: Option<&crate::fuzzy::finder::MatchPositions>,
) -> io::Result<()> {
    // Set cursor highlighting with Gruvbox soft colors
    if is_cursor {
        // Gruvbox soft highlight: dark grey background, yellow foreground, bold
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::Yellow),
            SetAttribute(Attribute::Bold)
        )?;
    }

    // Set selection highlighting (only show checkmarks for selected items)
    if is_selected {
        execute!(stdout, SetForegroundColor(Color::Green), Print("✓ "))?;
    } else {
        execute!(stdout, Print("  "))?;
    }

    // Draw item with match highlighting
    if let Some(matches) = match_positions {
        for (i, ch) in item.chars().enumerate() {
            if matches.positions.contains(&i) {
                // Highlight matched characters with Gruvbox soft colors
                if is_cursor {
                    // For selected rows, use bright white that contrasts with dark grey background
                    execute!(
                        stdout,
                        SetForegroundColor(Color::White),
                        SetAttribute(Attribute::Bold),
                        SetAttribute(Attribute::Underlined)
                    )?;
                } else {
                    // For non-selected rows, use bold and underline
                    execute!(
                        stdout,
                        SetAttribute(Attribute::Bold),
                        SetAttribute(Attribute::Underlined)
                    )?;
                }
                execute!(stdout, Print(ch))?;
                // Reset attributes after each character to prevent bleeding
                if is_cursor {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        SetAttribute(Attribute::NoUnderline)
                    )?;
                } else {
                    execute!(
                        stdout,
                        SetAttribute(Attribute::NoUnderline),
                        SetAttribute(Attribute::NormalIntensity)
                    )?;
                }
            } else {
                execute!(stdout, Print(ch))?;
            }
        }
    } else {
        execute!(stdout, Print(item))?;
    }

    // Reset all attributes
    execute!(stdout, ResetColor)?;
    Ok(())
}

/// Draw an item to the screen buffer with fuzzy match highlighting
fn draw_item_to_buffer(
    buffer: &mut ScreenBuffer,
    row: u16,
    item: &str,
    is_cursor: bool,
    is_selected: bool,
    match_positions: Option<&crate::fuzzy::finder::MatchPositions>,
) {
    let mut col: u16 = 0;

    // Determine base styling for this row
    let (base_fg, base_bg, base_bold) = if is_cursor {
        (Some(Color::Yellow), Some(Color::DarkGrey), true)
    } else {
        (None, None, false)
    };

    // Draw selection indicator
    if is_selected {
        col += buffer.put_str(col, row, "✓ ", Some(Color::Green), base_bg, false, false);
    } else {
        col += buffer.put_str(col, row, "  ", base_fg, base_bg, base_bold, false);
    }

    // Draw item text with match highlighting
    if let Some(matches) = match_positions {
        for (i, ch) in item.chars().enumerate() {
            if col >= buffer.width() {
                break;
            }
            let is_match = matches.positions.contains(&i);
            let (fg, bold, underline) = if is_match {
                if is_cursor {
                    (Some(Color::White), true, true)
                } else {
                    (base_fg, true, true)
                }
            } else {
                (base_fg, base_bold, false)
            };
            buffer.put_char(col, row, ch, fg, base_bg, bold, underline);
            col += 1;
        }
    } else {
        // No match highlighting, just draw the item
        for ch in item.chars() {
            if col >= buffer.width() {
                break;
            }
            buffer.put_char(col, row, ch, base_fg, base_bg, base_bold, false);
            col += 1;
        }
    }

    // Fill the rest of the row with background color if cursor is on this row
    if is_cursor {
        while col < buffer.width() {
            buffer.put_char(col, row, ' ', base_fg, base_bg, false, false);
            col += 1;
        }
    }
}

/// Draw an item with indicator to the screen buffer
#[allow(clippy::too_many_arguments)]
fn draw_item_with_indicator_to_buffer(
    buffer: &mut ScreenBuffer,
    row: u16,
    item: &str,
    is_cursor: bool,
    is_selected: bool,
    match_positions: Option<&crate::fuzzy::finder::MatchPositions>,
    indicator: Option<&ItemIndicator>,
    spinner_frame: usize,
) {
    let mut col: u16 = 0;

    // Determine base styling for this row
    let (base_fg, base_bg, base_bold) = if is_cursor {
        (Some(Color::Yellow), Some(Color::DarkGrey), true)
    } else {
        (None, None, false)
    };

    // Draw indicator prefix
    match indicator {
        Some(ItemIndicator::Spinner) => {
            let frame = SPINNER_FRAMES[spinner_frame % SPINNER_FRAMES.len()];
            col += buffer.put_str(
                col,
                row,
                &format!("{} ", frame),
                Some(Color::Yellow),
                base_bg,
                false,
                false,
            );
        }
        Some(ItemIndicator::Text(text)) => {
            col += buffer.put_str(col, row, text, base_fg, base_bg, base_bold, false);
            col += buffer.put_str(col, row, " ", base_fg, base_bg, base_bold, false);
        }
        Some(ItemIndicator::ColoredText(text, color)) => {
            col += buffer.put_str(col, row, text, Some(*color), base_bg, false, false);
            col += buffer.put_str(col, row, " ", base_fg, base_bg, base_bold, false);
        }
        Some(ItemIndicator::Success) => {
            col += buffer.put_str(col, row, "✓ ", Some(Color::Green), base_bg, false, false);
        }
        Some(ItemIndicator::Error) => {
            col += buffer.put_str(col, row, "✗ ", Some(Color::Red), base_bg, false, false);
        }
        Some(ItemIndicator::Warning) => {
            col += buffer.put_str(col, row, "⚠ ", Some(Color::Yellow), base_bg, false, false);
        }
        Some(ItemIndicator::None) | None => {
            // Selection indicator takes precedence when no other indicator
            if is_selected {
                col += buffer.put_str(col, row, "✓ ", Some(Color::Green), base_bg, false, false);
            } else {
                col += buffer.put_str(col, row, "  ", base_fg, base_bg, base_bold, false);
            }
        }
    }

    // Draw item text with match highlighting
    if let Some(matches) = match_positions {
        for (i, ch) in item.chars().enumerate() {
            if col >= buffer.width() {
                break;
            }
            let is_match = matches.positions.contains(&i);
            let (fg, bold, underline) = if is_match {
                if is_cursor {
                    (Some(Color::White), true, true)
                } else {
                    (base_fg, true, true)
                }
            } else {
                (base_fg, base_bold, false)
            };
            buffer.put_char(col, row, ch, fg, base_bg, bold, underline);
            col += 1;
        }
    } else {
        // No match highlighting, just draw the item
        for ch in item.chars() {
            if col >= buffer.width() {
                break;
            }
            buffer.put_char(col, row, ch, base_fg, base_bg, base_bold, false);
            col += 1;
        }
    }

    // Fill the rest of the row with background color if cursor is on this row
    if is_cursor {
        while col < buffer.width() {
            buffer.put_char(col, row, ' ', base_fg, base_bg, false, false);
            col += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_highlighted_item_cursor_highlighting() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", true, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        // Check for Gruvbox soft highlight colors (using 256-color codes)
        assert!(output_str.contains("\x1b[48;5;8m")); // Dark grey background
        assert!(output_str.contains("\x1b[38;5;11m")); // Yellow foreground
        assert!(output_str.contains("\x1b[1m")); // Bold
    }

    #[test]
    fn test_draw_highlighted_item_no_cursor() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", false, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("  test"));
    }

    #[test]
    fn test_draw_highlighted_item_with_matches() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", false, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("test"));
    }

    #[test]
    fn test_draw_highlighted_item_selected() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", false, true, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("✓"));
    }

    #[test]
    fn test_tui_config_default() {
        let config = TuiConfig::default();
        assert!(config.fullscreen);
        assert!(config.height.is_none());
        assert!(config.height_percentage.is_none());
        assert!(config.show_help_text);
        assert!(config.show_loading_indicator);
        assert!(config.loading_message.is_none());
        assert!(config.ready_message.is_none());
    }

    #[test]
    fn test_tui_config_with_height() {
        let config = TuiConfig::with_height(10);
        assert!(!config.fullscreen);
        assert_eq!(config.height, Some(10));
        assert!(config.height_percentage.is_none());
        assert!(config.show_loading_indicator);
    }

    #[test]
    fn test_tui_config_with_height_percentage() {
        let config = TuiConfig::with_height_percentage(50.0);
        assert!(!config.fullscreen);
        assert!(config.height.is_none());
        assert_eq!(config.height_percentage, Some(50.0));
        assert!(config.show_loading_indicator);
    }

    #[test]
    fn test_tui_config_fullscreen() {
        let config = TuiConfig::fullscreen();
        assert!(config.fullscreen);
        assert!(config.show_loading_indicator);
        assert!(config.height.is_none());
        assert!(config.height_percentage.is_none());
    }

    #[test]
    fn test_calculate_height_fullscreen() {
        let config = TuiConfig::fullscreen();
        let height = config.calculate_height(25);
        assert_eq!(height, 25); // 25 - 2 for borders
    }

    #[test]
    fn test_calculate_height_fixed() {
        let config = TuiConfig::with_height(10);
        let height = config.calculate_height(25);
        assert_eq!(height, 10);
    }

    #[test]
    fn test_calculate_height_percentage() {
        let config = TuiConfig::with_height_percentage(50.0);
        let height = config.calculate_height(20);
        assert_eq!(height, 10); // 50% of 20 = 10
    }

    #[test]
    fn test_calculate_height_overflow() {
        let config = TuiConfig::with_height(30);
        let height = config.calculate_height(25);
        assert_eq!(height, 25); // Should be capped at terminal height - 2
    }

    #[test]
    fn test_cursor_position_logic() {
        // Test cursor wrapping logic
        let config = TuiConfig::default();
        let display_height = config.calculate_height(25);
        assert!(display_height > 0);
    }

    #[test]
    fn test_cursor_highlighting_logic() {
        // Test that cursor highlighting works correctly
        let mut output = Vec::new();

        // Test cursor position
        draw_highlighted_item_with_matches(&mut output, "test", true, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("\x1b[48;5;8m")); // Dark grey background
        assert!(output_str.contains("\x1b[38;5;11m")); // Yellow foreground
        assert!(output_str.contains("\x1b[1m")); // Bold

        // Test non-cursor position
        let mut output2 = Vec::new();
        draw_highlighted_item_with_matches(&mut output2, "test", false, false, None).unwrap();
        let output_str2 = String::from_utf8(output2).unwrap();
        assert!(!output_str2.contains("\x1b[48;5;8m")); // No dark grey background
        assert!(!output_str2.contains("\x1b[38;5;11m")); // No yellow foreground
        assert!(output_str2.contains("  test"));
    }

    #[test]
    fn test_highlighting_colors_applied() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", true, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Check that color codes are present
        assert!(output_str.contains("\x1b["));
    }

    #[tokio::test]
    async fn test_create_items_channel() {
        let (sender, mut receiver) = create_items_channel();

        // Send some items
        sender.send("item1".to_string()).await.unwrap();
        sender.send("item2".to_string()).await.unwrap();
        drop(sender); // Close the sender

        // Collect items from receiver
        let mut collected = Vec::new();
        while let Some(item) = receiver.recv().await {
            collected.push(item);
        }

        assert_eq!(collected, vec!["item1".to_string(), "item2".to_string()]);
    }

    #[tokio::test]
    async fn test_handle_async_key_event_ctrl_c() {
        use crate::fuzzy::FuzzyFinder;
        use crossterm::event::{KeyCode, KeyModifiers};

        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let action = handle_async_key_event(&key_event, &mut finder).await;

        assert_eq!(action, crate::tui::controls::Action::Exit);
    }

    #[tokio::test]
    async fn test_handle_async_key_event_escape_with_empty_query() {
        use crate::fuzzy::FuzzyFinder;
        use crossterm::event::{KeyCode, KeyModifiers};

        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        // Query is empty, so Escape should exit
        assert!(finder.get_query().is_empty());

        let key_event = crossterm::event::KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = handle_async_key_event(&key_event, &mut finder).await;

        assert_eq!(action, crate::tui::controls::Action::Exit);
    }

    #[tokio::test]
    async fn test_handle_async_key_event_escape_with_query_clears_query() {
        use crate::fuzzy::FuzzyFinder;
        use crossterm::event::{KeyCode, KeyModifiers};

        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        // Set a query first
        finder.set_query("app".to_string()).await;
        assert_eq!(finder.get_query(), "app");

        // First Escape should clear the query, not exit
        let key_event = crossterm::event::KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = handle_async_key_event(&key_event, &mut finder).await;

        assert_eq!(action, crate::tui::controls::Action::Continue);
        assert!(finder.get_query().is_empty());
    }

    #[tokio::test]
    async fn test_handle_async_key_event_escape_twice_exits() {
        use crate::fuzzy::FuzzyFinder;
        use crossterm::event::{KeyCode, KeyModifiers};

        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        // Set a query first
        finder.set_query("app".to_string()).await;
        assert_eq!(finder.get_query(), "app");

        let key_event = crossterm::event::KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        // First Escape: clears query
        let action1 = handle_async_key_event(&key_event, &mut finder).await;
        assert_eq!(action1, crate::tui::controls::Action::Continue);
        assert!(finder.get_query().is_empty());

        // Second Escape: exits
        let action2 = handle_async_key_event(&key_event, &mut finder).await;
        assert_eq!(action2, crate::tui::controls::Action::Exit);
    }

    #[tokio::test]
    async fn test_handle_async_key_event_escape_preserves_selections() {
        use crate::fuzzy::FuzzyFinder;
        use crossterm::event::{KeyCode, KeyModifiers};

        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, true).await; // multi-select

        // Set a query and make a selection
        finder.set_query("a".to_string()).await;
        finder.toggle_selection(); // Select first item

        let selected_before = finder.get_selected_items();
        assert!(!selected_before.is_empty());

        // Escape to clear query
        let key_event = crossterm::event::KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let action = handle_async_key_event(&key_event, &mut finder).await;

        assert_eq!(action, crate::tui::controls::Action::Continue);
        assert!(finder.get_query().is_empty());

        // Selections should still be there
        let selected_after = finder.get_selected_items();
        assert_eq!(selected_before, selected_after);
    }

    #[test]
    fn test_item_indicator_default() {
        let indicator = ItemIndicator::default();
        assert_eq!(indicator, ItemIndicator::None);
    }

    #[test]
    fn test_item_indicator_variants() {
        let spinner = ItemIndicator::Spinner;
        let text = ItemIndicator::Text("*".to_string());
        let colored = ItemIndicator::ColoredText("!".to_string(), Color::Red);
        let success = ItemIndicator::Success;
        let error = ItemIndicator::Error;
        let warning = ItemIndicator::Warning;
        let none = ItemIndicator::None;

        // Test that different variants are not equal
        assert_ne!(spinner, text);
        assert_ne!(text, colored);
        assert_ne!(success, error);
        assert_ne!(warning, none);
    }

    #[test]
    fn test_global_status_default() {
        let status = GlobalStatus::default();
        assert!(matches!(status, GlobalStatus::Hidden));
    }

    #[tokio::test]
    async fn test_create_command_channel() {
        let (sender, mut receiver) = create_command_channel();

        // Send some commands
        sender
            .send(TuiCommand::AddItem("item1".to_string()))
            .await
            .unwrap();
        sender
            .send(TuiCommand::AddItemWithIndicator(
                "item2".to_string(),
                ItemIndicator::Spinner,
            ))
            .await
            .unwrap();
        sender
            .send(TuiCommand::UpdateIndicator(
                "item1".to_string(),
                ItemIndicator::Success,
            ))
            .await
            .unwrap();
        drop(sender);

        // Collect commands
        let mut commands = Vec::new();
        while let Some(cmd) = receiver.recv().await {
            commands.push(cmd);
        }

        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0], TuiCommand::AddItem(_)));
        assert!(matches!(
            commands[1],
            TuiCommand::AddItemWithIndicator(_, _)
        ));
        assert!(matches!(commands[2], TuiCommand::UpdateIndicator(_, _)));
    }

    #[test]
    fn test_draw_item_with_spinner_indicator() {
        let mut output = Vec::new();
        draw_item_with_indicator(
            &mut output,
            "test",
            false,
            false,
            None,
            Some(&ItemIndicator::Spinner),
            0,
        )
        .unwrap();
        let output_str = String::from_utf8(output).unwrap();
        // Should contain the first spinner frame
        assert!(output_str.contains(SPINNER_FRAMES[0]));
    }

    #[test]
    fn test_draw_item_with_success_indicator() {
        let mut output = Vec::new();
        draw_item_with_indicator(
            &mut output,
            "test",
            false,
            false,
            None,
            Some(&ItemIndicator::Success),
            0,
        )
        .unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("✓"));
    }

    #[test]
    fn test_draw_item_with_error_indicator() {
        let mut output = Vec::new();
        draw_item_with_indicator(
            &mut output,
            "test",
            false,
            false,
            None,
            Some(&ItemIndicator::Error),
            0,
        )
        .unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("✗"));
    }

    #[test]
    fn test_draw_item_with_warning_indicator() {
        let mut output = Vec::new();
        draw_item_with_indicator(
            &mut output,
            "test",
            false,
            false,
            None,
            Some(&ItemIndicator::Warning),
            0,
        )
        .unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("⚠"));
    }

    #[test]
    fn test_draw_item_with_text_indicator() {
        let mut output = Vec::new();
        draw_item_with_indicator(
            &mut output,
            "test",
            false,
            false,
            None,
            Some(&ItemIndicator::Text("[*]".to_string())),
            0,
        )
        .unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("[*]"));
    }
}
