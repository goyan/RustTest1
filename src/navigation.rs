use std::path::{Path, PathBuf};
use std::fs;
use std::thread;
use crate::DiskDashboard;
use crate::models::FileItem;
use crate::analysis::analyze_file;
use crate::utils::calculate_dir_size_recursive;

impl DiskDashboard {
    pub fn navigate_to(&mut self, path: PathBuf) {
        // Add to history
        if let Some(ref current) = self.current_path {
            if current != &path {
                // Remove any forward history
                self.navigation_history.truncate(self.history_index + 1);
                self.navigation_history.push(current.clone());
                self.history_index = self.navigation_history.len() - 1;
            }
        }
        self.current_path = Some(path);
    }

    pub fn navigate_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(path) = self.navigation_history.get(self.history_index) {
                self.current_path = Some(path.clone());
            }
        } else if let Some(parent) = self.current_path.as_ref().and_then(|p| p.parent()) {
            self.current_path = Some(parent.to_path_buf());
        }
    }

    pub fn navigate_forward(&mut self) {
        if self.history_index < self.navigation_history.len() - 1 {
            self.history_index += 1;
            if let Some(path) = self.navigation_history.get(self.history_index) {
                self.current_path = Some(path.clone());
            }
        }
    }

    pub fn load_directory(&mut self, path: &Path) {
        self.loading = true;
        self.file_items.clear();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let metadata = entry.metadata().ok();

                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let modified = metadata.as_ref().and_then(|m| m.modified().ok());

                let name = entry_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                // Calculate size: for files use metadata, for dirs calculate recursive size
                let (size, child_count) = if is_dir {
                    let count = fs::read_dir(&entry_path).ok().map(|rd| rd.count());
                    // Use cached recursive size or calculate it
                    let dir_size = self.get_folder_size_recursive(&entry_path);
                    (dir_size, count)
                } else {
                    (metadata.as_ref().map(|m| m.len()).unwrap_or(0), None)
                };

                let (category, usefulness) = analyze_file(&entry_path, &name, is_dir, size);

                self.file_items.push(FileItem {
                    path: entry_path,
                    name,
                    size,
                    is_dir,
                    category,
                    usefulness,
                    modified,
                    child_count,
                });
            }
        }

        // Apply filtering and sorting
        self.apply_filter_and_sort();
        self.loading = false;
    }

    pub fn apply_filter_and_sort(&mut self) {
        // Filter items based on search query
        if self.search_query.is_empty() {
            self.filtered_items = self.file_items.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_items = self.file_items.iter()
                .filter(|item| {
                    item.name.to_lowercase().contains(&query_lower) ||
                    item.path.to_string_lossy().to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect();
        }

        // Apply sorting to filtered items
        self.sort_file_items();
    }

    pub fn sort_file_items(&mut self) {
        self.filtered_items.sort_by(|a, b| {
            // Directories always first
            match (a.is_dir, b.is_dir) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }

            let ordering = match self.sort_column {
                crate::models::SortColumn::Name => a.name.cmp(&b.name),
                crate::models::SortColumn::Size => a.size.cmp(&b.size),
                crate::models::SortColumn::Category => {
                    let a_val = a.category as u8;
                    let b_val = b.category as u8;
                    a_val.cmp(&b_val)
                },
                crate::models::SortColumn::Usefulness => a.usefulness.partial_cmp(&b.usefulness).unwrap_or(std::cmp::Ordering::Equal),
            };

            match self.sort_direction {
                crate::models::SortDirection::Ascending => ordering,
                crate::models::SortDirection::Descending => ordering.reverse(),
            }
        });
    }

    /// Get folder size - returns cached value or starts async calculation
    pub fn get_folder_size_recursive(&mut self, path: &Path) -> u64 {
        // Check cache first
        if let Some(&size) = self.folder_size_cache.get(path) {
            return size;
        }

        // Check if calculation is already pending
        if self.pending_size_calculations.contains(path) {
            return 0; // Return 0 while calculating
        }

        // Start async calculation
        let path_buf = path.to_path_buf();
        let sender = self.size_sender.clone();
        self.pending_size_calculations.insert(path_buf.clone());

        thread::spawn(move || {
            let size = calculate_dir_size_recursive(&path_buf);
            let _ = sender.send((path_buf, size));
        });

        0 // Return 0 while calculating
    }
}
