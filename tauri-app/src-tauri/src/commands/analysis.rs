use std::path::Path;
use crate::utils::calculate_dir_size_recursive;

#[tauri::command]
pub fn get_folder_size(path: String) -> Result<u64, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir_path.is_dir() {
        return Err("Not a directory".to_string());
    }
    Ok(calculate_dir_size_recursive(dir_path))
}
