use std::path::Path;
use std::fs;
use std::time::SystemTime;
use crate::models::{FileItem, BatchDeleteResult};
use crate::analysis::{analyze_file, get_file_icon, is_protected_full_path};

#[tauri::command]
pub fn load_directory(path: String) -> Result<Vec<FileItem>, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Cannot read directory: {}", e))?;

    let mut items: Vec<FileItem> = Vec::new();

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let metadata = entry.metadata().ok();

        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let modified = metadata.as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let name = entry_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let (size, child_count) = if is_dir {
            let count = fs::read_dir(&entry_path).ok().map(|rd| rd.count());
            (0u64, count)  // Size will be calculated async
        } else {
            (metadata.as_ref().map(|m| m.len()).unwrap_or(0), None)
        };

        let (category, usefulness) = analyze_file(&entry_path, &name, is_dir, size);
        let is_empty = is_dir && child_count == Some(0);
        let icon = get_file_icon(&name, is_dir, is_empty, category).to_string();

        items.push(FileItem {
            path: entry_path.to_string_lossy().to_string(),
            name,
            size,
            is_dir,
            category,
            usefulness,
            modified,
            child_count,
            icon,
        });
    }

    Ok(items)
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    let file_path = Path::new(&path);

    if is_protected_full_path(&path) {
        return Err("Cannot delete protected system file".to_string());
    }

    if !file_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if file_path.is_dir() {
        fs::remove_dir_all(file_path)
            .map_err(|e| format!("Failed to delete folder: {}", e))
    } else {
        fs::remove_file(file_path)
            .map_err(|e| format!("Failed to delete file: {}", e))
    }
}

#[tauri::command]
pub fn batch_delete(paths: Vec<String>) -> Result<BatchDeleteResult, String> {
    let mut deleted = 0u32;
    let mut skipped = 0u32;
    let mut errors = Vec::new();

    for path_str in &paths {
        if is_protected_full_path(path_str) {
            skipped += 1;
            continue;
        }

        let path = Path::new(path_str);
        let result = if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };

        match result {
            Ok(_) => deleted += 1,
            Err(e) => {
                let name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                errors.push(format!("{}: {}", name, e));
            }
        }
    }

    Ok(BatchDeleteResult { deleted, skipped, errors })
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let target = Path::new(&path);
        let dir = if target.is_dir() {
            target.to_path_buf()
        } else {
            target.parent().unwrap_or(target).to_path_buf()
        };
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }
    Ok(())
}
