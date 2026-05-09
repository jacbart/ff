use std::io::{self, Read, Write};

/// Get cursor position by querying stderr (fallback for when stdout is redirected)
pub fn get_cursor_position_from_stderr() -> io::Result<(u16, u16)> {
    let mut stderr = io::stderr();
    let mut stdin = io::stdin();

    // Write ANSI query for cursor position
    write!(stderr, "\x1b[6n")?;
    stderr.flush()?;

    // Read response: ESC [ rows ; cols R
    let mut buf = [0u8; 1];
    let mut response = Vec::new();
    let mut read_count = 0;

    // Read byte by byte until 'R' or limit
    while read_count < 16 {
        match stdin.read_exact(&mut buf) {
            Ok(_) => {
                response.push(buf[0]);
                if buf[0] == b'R' {
                    break;
                }
            }
            Err(_) => break,
        }
        read_count += 1;
    }

    // Parse response
    let s = String::from_utf8_lossy(&response);
    if s.starts_with("\x1b[") && s.ends_with('R') {
        let content = &s[2..s.len() - 1];
        let parts: Vec<&str> = content.split(';').collect();
        if parts.len() == 2 {
            let row = parts[0].parse::<u16>().unwrap_or(1);
            let col = parts[1].parse::<u16>().unwrap_or(1);
            // ANSI is 1-based, crossterm is 0-based
            return Ok((col.saturating_sub(1), row.saturating_sub(1)));
        }
    }

    Err(io::Error::other(
        "Failed to parse cursor position from stderr",
    ))
}

/// Get terminal size by querying stderr (fallback)
pub fn get_terminal_size_from_stderr() -> io::Result<(u16, u16)> {
    use std::process::Command;
    let output = Command::new("stty")
        .arg("size")
        .arg("-F")
        .arg("/dev/stderr")
        .output();

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = output_str.split_whitespace().collect();
        if parts.len() == 2 {
            let rows = parts[0].parse::<u16>().unwrap_or(24);
            let cols = parts[1].parse::<u16>().unwrap_or(80);
            return Ok((cols, rows));
        }
    }

    Err(io::Error::other("Failed to get terminal size from stderr"))
}
