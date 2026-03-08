use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum FileCategory {
    MustKeep,    // Critical system files, important user data
    System,      // System files that should generally be kept
    Regular,     // Normal files
    Useless,     // Temp files, cache, logs, etc.
    Unknown,     // Can't determine
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortColumn {
    Name,
    Size,
    Category,
    Usefulness,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone)]
pub struct FileItem {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,              // For files: file size. For folders: total size of contents
    pub is_dir: bool,
    pub category: FileCategory,
    pub usefulness: f32,        // 0-100 score
    pub modified: Option<SystemTime>,
    pub child_count: Option<usize>, // For directories: number of items inside
}
