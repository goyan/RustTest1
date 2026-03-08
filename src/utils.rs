use std::path::Path;
use std::fs;
use crate::models::{FileItem, FileCategory, SortColumn};

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

/// Calculate the total size of a directory (non-recursive, just immediate children)
pub fn calculate_dir_size_shallow(path: &Path) -> u64 {
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

/// Calculate the total size of a directory recursively (with depth limit)
pub fn calculate_dir_size_recursive(path: &Path) -> u64 {
    calculate_dir_size_recursive_limited(path, 2) // Limit to 2 levels to avoid UI freeze
}

/// Calculate directory size with depth limit to prevent crashes
pub fn calculate_dir_size_recursive_limited(path: &Path, max_depth: u32) -> u64 {
    if max_depth == 0 {
        // At max depth, just return shallow size
        return calculate_dir_size_shallow(path);
    }

    let mut total_size: u64 = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size = total_size.saturating_add(metadata.len());
                } else if metadata.is_dir() {
                    // Skip system folders that might cause issues
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy().to_lowercase();
                    if name_str.starts_with("$") ||
                       name_str == "system volume information" ||
                       name_str == "windows" {
                        continue;
                    }
                    // Recursively calculate subdirectory size
                    total_size = total_size.saturating_add(
                        calculate_dir_size_recursive_limited(&entry.path(), max_depth - 1)
                    );
                }
            }
        }
    }

    total_size
}

/// Sort comparison for file items
#[allow(dead_code)] // Used in tests
pub fn compare_file_items(a: &FileItem, b: &FileItem, sort_column: SortColumn, ascending: bool) -> std::cmp::Ordering {
    // Directories always first
    match (a.is_dir, b.is_dir) {
        (true, false) => return std::cmp::Ordering::Less,
        (false, true) => return std::cmp::Ordering::Greater,
        _ => {}
    }

    let ordering = match sort_column {
        SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        SortColumn::Size => a.size.cmp(&b.size),
        SortColumn::Category => {
            let cat_order = |c: &FileCategory| -> u8 {
                match c {
                    FileCategory::MustKeep => 0,
                    FileCategory::System => 1,
                    FileCategory::Regular => 2,
                    FileCategory::Useless => 3,
                    FileCategory::Unknown => 4,
                }
            };
            cat_order(&a.category).cmp(&cat_order(&b.category))
        }
        SortColumn::Usefulness => a.usefulness.partial_cmp(&b.usefulness).unwrap_or(std::cmp::Ordering::Equal),
    };

    if ascending { ordering } else { ordering.reverse() }
}

/// Filter items by search query
#[allow(dead_code)] // Used in tests
pub fn filter_items(items: &[FileItem], query: &str) -> Vec<FileItem> {
    if query.is_empty() {
        return items.to_vec();
    }
    let query_lower = query.to_lowercase();
    items.iter()
        .filter(|item| item.name.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}
