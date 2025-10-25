use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;

/// Read input items from the specified source.
pub async fn read_input(source: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if let Some(stripped) = source.strip_prefix("unix://") {
        read_from_unix_socket(stripped).await
    } else if source.starts_with("http://") || source.starts_with("https://") {
        read_from_http_socket(source).await
    } else if let Some(stripped) = source.strip_prefix("dir:") {
        read_from_directory(stripped).await
    } else if Path::new(source).exists() {
        if Path::new(source).is_dir() {
            read_from_directory(source).await
        } else {
            read_from_file(source).await
        }
    } else {
        // Treat as space-separated list
        Ok(source.split_whitespace().map(|s| s.to_string()).collect())
    }
}

/// Process direct items provided as command line arguments.
pub fn read_direct_items(items: Vec<String>) -> Result<Vec<String>, String> {
    if items.is_empty() {
        return Err("No items provided".to_string());
    }
    Ok(items)
}

/// Process content as if it came from stdin.
pub fn process_stdin_content(content: &str) -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            items.push(trimmed.to_string());
        }
    }
    if items.is_empty() {
        return Err("No items found in stdin".to_string());
    }
    Ok(items)
}

/// Process content as if it came from a file.
pub fn process_file_content(content: &str) -> Result<Vec<String>, String> {
    let items: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();
    if items.is_empty() {
        return Err("No items found in file".to_string());
    }
    Ok(items)
}

/// Send input items from the specified source to an mpsc channel.
pub async fn send_input_to_channel(
    source: &str,
    sender: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(stripped) = source.strip_prefix("unix://") {
        send_from_unix_socket(stripped, sender).await
    } else if source.starts_with("http://") || source.starts_with("https://") {
        send_from_http_socket(source, sender).await
    } else if let Some(stripped) = source.strip_prefix("dir:") {
        send_from_directory(stripped, sender).await
    } else if Path::new(source).exists() {
        if Path::new(source).is_dir() {
            send_from_directory(source, sender).await
        } else {
            send_from_file(source, sender).await
        }
    } else {
        // Treat as space-separated list
        for item in source.split_whitespace() {
            if !item.trim().is_empty() && sender.send(item.trim().to_string()).await.is_err() {
                break; // Channel closed
            }
        }
        Ok(())
    }
}

async fn read_from_file(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path).await?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

async fn read_from_unix_socket(
    socket_path: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect to Unix socket: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    let bytes_read = reader
        .read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("Failed to read from Unix socket: {e}"))?;

    if bytes_read == 0 {
        return Ok(Vec::new());
    }

    let content = String::from_utf8(buffer)?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

async fn read_from_http_socket(url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Simple HTTP client implementation without external dependencies
    let url = url.replace("http://", "").replace("https://", "");
    let stream = tokio::net::TcpStream::connect(url)
        .await
        .map_err(|e| format!("Failed to connect to HTTP socket: {e}"))?;

    // This is a simplified implementation - in practice you'd want proper HTTP parsing
    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("Failed to read from HTTP socket: {e}"))?;

    let content = String::from_utf8_lossy(&buffer);
    Ok(content.lines().map(|s| s.to_string()).collect())
}

async fn read_from_directory(dir_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut entries = fs::read_dir(dir_path)
        .await
        .map_err(|e| format!("Failed to read directory '{dir_path}': {e}"))?;

    let mut items = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                items.push(name_str.to_string());
            }
        }
    }

    Ok(items)
}

async fn send_from_file(
    file_path: &str,
    sender: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path).await?;
    for line in content.lines() {
        if !line.trim().is_empty() && sender.send(line.trim().to_string()).await.is_err() {
            break; // Channel closed
        }
    }
    Ok(())
}

async fn send_from_unix_socket(
    socket_path: &str,
    sender: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect to Unix socket: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    let bytes_read = reader
        .read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("Failed to read from Unix socket: {e}"))?;

    if bytes_read == 0 {
        return Ok(());
    }

    let content = String::from_utf8(buffer)?;
    for line in content.lines() {
        if !line.trim().is_empty() && sender.send(line.trim().to_string()).await.is_err() {
            break; // Channel closed
        }
    }
    Ok(())
}

async fn send_from_http_socket(
    url: &str,
    sender: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Simple HTTP client implementation without external dependencies
    let url = url.replace("http://", "").replace("https://", "");
    let stream = tokio::net::TcpStream::connect(url)
        .await
        .map_err(|e| format!("Failed to connect to HTTP socket: {e}"))?;

    // This is a simplified implementation - in practice you'd want proper HTTP parsing
    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("Failed to read from HTTP socket: {e}"))?;

    let content = String::from_utf8_lossy(&buffer);
    for line in content.lines() {
        if !line.trim().is_empty() && sender.send(line.trim().to_string()).await.is_err() {
            break; // Channel closed
        }
    }
    Ok(())
}

async fn send_from_directory(
    dir_path: &str,
    sender: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = fs::read_dir(dir_path)
        .await
        .map_err(|e| format!("Failed to read directory '{dir_path}': {e}"))?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                if !name_str.trim().is_empty()
                    && sender.send(name_str.trim().to_string()).await.is_err()
                {
                    break; // Channel closed
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_input_space_separated() {
        let result = read_input("item1 item2 item3").await;
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items, vec!["item1", "item2", "item3"]);
    }

    #[tokio::test]
    async fn test_read_input_nonexistent_file() {
        let result = read_input("nonexistent_file.txt").await;
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items, vec!["nonexistent_file.txt"]);
    }

    #[tokio::test]
    async fn test_read_input_unknown_source() {
        let result = read_input("unknown_source").await;
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items, vec!["unknown_source"]);
    }
}
