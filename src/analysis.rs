use std::path::Path;
use crate::models::FileCategory;

pub fn analyze_file(path: &Path, name: &str, is_dir: bool, size: u64) -> (FileCategory, f32) {
    let name_lower = name.to_lowercase();
    let path_str = path.to_string_lossy().to_lowercase();

    // System and critical files - NEVER delete these
    if path_str.contains("windows\\system32") ||
       path_str.contains("windows\\syswow64") ||
       path_str.contains("program files") ||
       path_str.contains("programdata") ||
       name_lower == "windows" ||
       name_lower == "boot" ||
       name_lower == "bootmgr" ||
       name_lower == "pagefile.sys" ||
       name_lower == "hiberfil.sys" ||
       name_lower == "$recycle.bin" ||
       name_lower == "system volume information" ||
       name_lower == "recovery" ||
       name_lower.starts_with("$") {
        return (FileCategory::MustKeep, 100.0);
    }

    // Temp files and cache - useless (safe to delete)
    if name_lower.contains("temp") ||
       name_lower.contains("cache") ||
       name_lower.contains("tmp") ||
       name_lower.ends_with(".tmp") ||
       name_lower.ends_with(".log") ||
       path_str.contains("\\temp\\") ||
       path_str.contains("\\cache\\") ||
       path_str.contains("\\tmp\\") ||
       name_lower.starts_with("~$") {
        return (FileCategory::Useless, 5.0);
    }

    // System files
    if name_lower.ends_with(".sys") ||
       name_lower.ends_with(".dll") ||
       name_lower.ends_with(".exe") && path_str.contains("windows") ||
       name_lower.ends_with(".inf") ||
       name_lower.ends_with(".cat") {
        return (FileCategory::System, 85.0);
    }

    // Get file extension for detailed analysis
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    // Important user data - high usefulness
    let important_extensions = ["doc", "docx", "xls", "xlsx", "ppt", "pptx", "pdf",
                                "txt", "md", "rtf", "odt", "ods", "odp"];
    if important_extensions.contains(&ext.as_str()) {
        return (FileCategory::Regular, 90.0);
    }

    // Photos - very important to users
    let photo_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "raw", "cr2", "nef", "arw"];
    if photo_extensions.contains(&ext.as_str()) {
        return (FileCategory::Regular, 95.0);
    }

    // Videos - important but large
    let video_extensions = ["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v"];
    if video_extensions.contains(&ext.as_str()) {
        // Larger videos slightly less useful (more likely to be deletable)
        let usefulness = if size > 1_000_000_000 { 70.0 } else { 85.0 };
        return (FileCategory::Regular, usefulness);
    }

    // Audio - important
    let audio_extensions = ["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma"];
    if audio_extensions.contains(&ext.as_str()) {
        return (FileCategory::Regular, 80.0);
    }

    // Code and projects - important for developers
    let code_extensions = ["rs", "py", "js", "ts", "java", "c", "cpp", "h", "cs", "go",
                          "html", "css", "json", "xml", "yaml", "toml", "sql"];
    if code_extensions.contains(&ext.as_str()) {
        return (FileCategory::Regular, 85.0);
    }

    // Archives - depends on size, often can be deleted after extraction
    let archive_extensions = ["zip", "rar", "7z", "tar", "gz", "bz2"];
    if archive_extensions.contains(&ext.as_str()) {
        let usefulness = if size > 1_000_000_000 { 30.0 }  // >1GB - likely can delete
                        else if size > 100_000_000 { 45.0 }  // >100MB
                        else { 55.0 };
        return (FileCategory::Regular, usefulness);
    }

    // ISOs and disk images - usually can be deleted
    if ext == "iso" || ext == "dmg" || ext == "img" {
        return (FileCategory::Regular, 25.0);
    }

    // Executables and installers - often safe to delete after install
    let installer_extensions = ["exe", "msi", "bat", "cmd", "ps1"];
    if installer_extensions.contains(&ext.as_str()) {
        // Installers in Downloads are less useful
        if path_str.contains("downloads") {
            return (FileCategory::Regular, 35.0);
        }
        return (FileCategory::Regular, 60.0);
    }

    // Old backup files
    if name_lower.ends_with(".bak") || name_lower.ends_with(".old") || name_lower.contains("backup") {
        return (FileCategory::Regular, 40.0);
    }

    // Folders - base usefulness on contents
    if is_dir {
        // Node modules, build folders - low usefulness
        if name_lower == "node_modules" || name_lower == "target" ||
           name_lower == "build" || name_lower == "dist" || name_lower == ".git" {
            return (FileCategory::Regular, 30.0);
        }
        // User folders - high usefulness
        if name_lower == "documents" || name_lower == "pictures" ||
           name_lower == "music" || name_lower == "videos" {
            return (FileCategory::Regular, 95.0);
        }
        // Downloads - medium, often contains deletable files
        if name_lower == "downloads" {
            return (FileCategory::Regular, 50.0);
        }
        // Default folder usefulness
        return (FileCategory::Regular, 65.0);
    }

    // Default for unknown files - base on size
    let usefulness = if size > 500_000_000 { 45.0 }  // >500MB - might want to check
                    else if size > 100_000_000 { 55.0 }  // >100MB
                    else { 60.0 };
    (FileCategory::Regular, usefulness)
}

/// Categorize a file based on its path string, name, type, and size.
/// Delegates to `analyze_file` — this is a convenience wrapper for tests.
#[allow(dead_code)] // Used in tests
pub fn categorize_file(path: &str, name: &str, is_dir: bool, size: u64) -> (FileCategory, f32) {
    analyze_file(Path::new(path), name, is_dir, size)
}

/// Check if a path is a protected system path (name only)
#[allow(dead_code)] // Used in tests
pub fn is_protected_path(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.starts_with("$") ||
    name_lower == "system volume information" ||
    name_lower == "recovery" ||
    name_lower == "boot" ||
    name_lower == "bootmgr" ||
    name_lower == "pagefile.sys" ||
    name_lower == "hiberfil.sys"
}

/// Check if a full path is protected (includes Windows folder and Program Files)
#[allow(dead_code)] // Used in tests
pub fn is_protected_full_path(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_lowercase())
        .unwrap_or_default();

    name.starts_with("$") ||
    name == "system volume information" ||
    name == "recovery" ||
    name == "boot" ||
    name == "bootmgr" ||
    name == "pagefile.sys" ||
    name == "hiberfil.sys" ||
    path_lower.contains("\\windows\\") ||
    path_lower.ends_with("\\windows") ||
    path_lower.contains("program files")
}

/// Check if a folder should block navigation (empty folder)
#[allow(dead_code)] // Used in tests
pub fn should_block_folder_entry(child_count: Option<usize>) -> bool {
    child_count == Some(0)
}

/// Get the icon for a file based on its extension and category
#[allow(dead_code)] // Used in tests
pub fn get_file_icon(name: &str, is_dir: bool, is_empty_folder: bool, category: FileCategory) -> &'static str {
    if is_dir {
        return if is_empty_folder { "📂" } else { "📁" };
    }

    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => "🖼️",
        // Videos
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => "🎬",
        // Audio
        "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" => "🎵",
        // Documents
        "pdf" => "📕",
        "doc" | "docx" => "📘",
        "xls" | "xlsx" => "📗",
        "ppt" | "pptx" => "📙",
        "txt" | "md" | "rtf" => "📝",
        // Code
        "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "cs" | "go" => "💻",
        "html" | "css" | "json" | "xml" | "yaml" | "toml" => "🌐",
        // Archives
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "📦",
        // Executables
        "exe" | "msi" | "bat" | "cmd" | "ps1" | "sh" => "⚡",
        // Default by category
        _ => match category {
            FileCategory::MustKeep => "🔒",
            FileCategory::System => "⚙️",
            FileCategory::Regular => "📄",
            FileCategory::Useless => "🗑️",
            FileCategory::Unknown => "❓",
        }
    }
}
