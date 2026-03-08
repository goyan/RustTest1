use std::path::Path;
use std::fs;

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    const TB: u64 = 1024 * 1024 * 1024 * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes < TB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    }
}

pub fn calculate_dir_size_recursive(path: &Path) -> u64 {
    calculate_dir_size_recursive_limited(path, 2)
}

fn calculate_dir_size_recursive_limited(path: &Path, max_depth: u32) -> u64 {
    if max_depth == 0 {
        return calculate_dir_size_shallow(path);
    }

    let mut total_size: u64 = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size = total_size.saturating_add(metadata.len());
                } else if metadata.is_dir() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy().to_lowercase();
                    if name_str.starts_with("$") ||
                       name_str == "system volume information" ||
                       name_str == "windows" {
                        continue;
                    }
                    total_size = total_size.saturating_add(
                        calculate_dir_size_recursive_limited(&entry.path(), max_depth - 1)
                    );
                }
            }
        }
    }

    total_size
}

fn calculate_dir_size_shallow(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .filter(|m| m.is_file())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or(0)
}
