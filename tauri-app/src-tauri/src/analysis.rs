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

    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let important_extensions = ["doc", "docx", "xls", "xlsx", "ppt", "pptx", "pdf",
                                "txt", "md", "rtf", "odt", "ods", "odp"];
    if important_extensions.contains(&ext.as_str()) {
        return (FileCategory::Regular, 90.0);
    }

    let photo_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "raw", "cr2", "nef", "arw"];
    if photo_extensions.contains(&ext.as_str()) {
        return (FileCategory::Regular, 95.0);
    }

    let video_extensions = ["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v"];
    if video_extensions.contains(&ext.as_str()) {
        let usefulness = if size > 1_000_000_000 { 70.0 } else { 85.0 };
        return (FileCategory::Regular, usefulness);
    }

    let audio_extensions = ["mp3", "wav", "flac", "ogg", "aac", "m4a", "wma"];
    if audio_extensions.contains(&ext.as_str()) {
        return (FileCategory::Regular, 80.0);
    }

    let code_extensions = ["rs", "py", "js", "ts", "java", "c", "cpp", "h", "cs", "go",
                          "html", "css", "json", "xml", "yaml", "toml", "sql"];
    if code_extensions.contains(&ext.as_str()) {
        return (FileCategory::Regular, 85.0);
    }

    let archive_extensions = ["zip", "rar", "7z", "tar", "gz", "bz2"];
    if archive_extensions.contains(&ext.as_str()) {
        let usefulness = if size > 1_000_000_000 { 30.0 }
                        else if size > 100_000_000 { 45.0 }
                        else { 55.0 };
        return (FileCategory::Regular, usefulness);
    }

    if ext == "iso" || ext == "dmg" || ext == "img" {
        return (FileCategory::Regular, 25.0);
    }

    let installer_extensions = ["exe", "msi", "bat", "cmd", "ps1"];
    if installer_extensions.contains(&ext.as_str()) {
        if path_str.contains("downloads") {
            return (FileCategory::Regular, 35.0);
        }
        return (FileCategory::Regular, 60.0);
    }

    if name_lower.ends_with(".bak") || name_lower.ends_with(".old") || name_lower.contains("backup") {
        return (FileCategory::Regular, 40.0);
    }

    if is_dir {
        if name_lower == "node_modules" || name_lower == "target" ||
           name_lower == "build" || name_lower == "dist" || name_lower == ".git" {
            return (FileCategory::Regular, 30.0);
        }
        if name_lower == "documents" || name_lower == "pictures" ||
           name_lower == "music" || name_lower == "videos" {
            return (FileCategory::Regular, 95.0);
        }
        if name_lower == "downloads" {
            return (FileCategory::Regular, 50.0);
        }
        return (FileCategory::Regular, 65.0);
    }

    let usefulness = if size > 500_000_000 { 45.0 }
                    else if size > 100_000_000 { 55.0 }
                    else { 60.0 };
    (FileCategory::Regular, usefulness)
}

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

pub fn get_file_icon(name: &str, is_dir: bool, is_empty_folder: bool, category: FileCategory) -> &'static str {
    if is_dir {
        return if is_empty_folder { "\u{1F4C2}" } else { "\u{1F4C1}" };
    }

    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => "\u{1F5BC}\u{FE0F}",
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => "\u{1F3AC}",
        "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" => "\u{1F3B5}",
        "pdf" => "\u{1F4D5}",
        "doc" | "docx" => "\u{1F4D8}",
        "xls" | "xlsx" => "\u{1F4D7}",
        "ppt" | "pptx" => "\u{1F4D9}",
        "txt" | "md" | "rtf" => "\u{1F4DD}",
        "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "cs" | "go" => "\u{1F4BB}",
        "html" | "css" | "json" | "xml" | "yaml" | "toml" => "\u{1F310}",
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "\u{1F4E6}",
        "exe" | "msi" | "bat" | "cmd" | "ps1" | "sh" => "\u{26A1}",
        _ => match category {
            FileCategory::MustKeep => "\u{1F512}",
            FileCategory::System => "\u{2699}\u{FE0F}",
            FileCategory::Regular => "\u{1F4C4}",
            FileCategory::Useless => "\u{1F5D1}\u{FE0F}",
            FileCategory::Unknown => "\u{2753}",
        }
    }
}
