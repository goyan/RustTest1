use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum FileCategory {
    MustKeep,
    System,
    Regular,
    Useless,
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortColumn {
    Name,
    Size,
    Category,
    Usefulness,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub category: FileCategory,
    pub usefulness: f32,
    pub modified: Option<u64>,  // seconds since epoch
    pub child_count: Option<usize>,
    pub icon: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BatchDeleteResult {
    pub deleted: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}
