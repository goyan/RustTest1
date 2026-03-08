#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::collections::HashSet;
    use crate::models::*;
    use crate::analysis::*;
    use crate::utils::*;

    // ==================== Size Formatting Tests ====================

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 100), "100.0 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 500), "500.0 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 2), "2.00 GB");
    }

    #[test]
    fn test_format_size_terabytes() {
        assert_eq!(format_size(1024u64 * 1024 * 1024 * 1024), "1.00 TB");
        assert_eq!(format_size(1024u64 * 1024 * 1024 * 1024 * 5), "5.00 TB");
    }

    // ==================== Empty Folder Navigation Tests ====================

    #[test]
    fn test_empty_folder_blocks_navigation() {
        assert!(should_block_folder_entry(Some(0)));
    }

    #[test]
    fn test_non_empty_folder_allows_navigation() {
        assert!(!should_block_folder_entry(Some(1)));
        assert!(!should_block_folder_entry(Some(10)));
        assert!(!should_block_folder_entry(Some(100)));
    }

    #[test]
    fn test_unknown_folder_count_allows_navigation() {
        assert!(!should_block_folder_entry(None));
    }

    // ==================== Protected Path Tests (Name Only) ====================

    #[test]
    fn test_recycle_bin_is_protected() {
        assert!(is_protected_path("$RECYCLE.BIN"));
        assert!(is_protected_path("$Recycle.Bin"));
        assert!(is_protected_path("$recycle.bin"));
    }

    #[test]
    fn test_system_volume_info_is_protected() {
        assert!(is_protected_path("System Volume Information"));
        assert!(is_protected_path("system volume information"));
    }

    #[test]
    fn test_system_files_are_protected() {
        assert!(is_protected_path("pagefile.sys"));
        assert!(is_protected_path("hiberfil.sys"));
        assert!(is_protected_path("bootmgr"));
        assert!(is_protected_path("Recovery"));
        assert!(is_protected_path("boot"));
    }

    #[test]
    fn test_dollar_prefix_is_protected() {
        assert!(is_protected_path("$WinREAgent"));
        assert!(is_protected_path("$SysReset"));
        assert!(is_protected_path("$Windows.~BT"));
    }

    #[test]
    fn test_normal_folders_not_protected_by_name() {
        assert!(!is_protected_path("Documents"));
        assert!(!is_protected_path("Users"));
        assert!(!is_protected_path("my_project"));
    }

    // ==================== Protected Full Path Tests ====================

    #[test]
    fn test_windows_folder_is_protected() {
        assert!(is_protected_full_path("C:\\Windows"));
        assert!(is_protected_full_path("C:\\WINDOWS"));
        assert!(is_protected_full_path("c:\\windows"));
    }

    #[test]
    fn test_windows_subfolder_is_protected() {
        assert!(is_protected_full_path("C:\\Windows\\System32"));
        assert!(is_protected_full_path("C:\\Windows\\Panther"));
        assert!(is_protected_full_path("C:\\Windows\\Fonts"));
        assert!(is_protected_full_path("C:\\Windows\\SysWOW64\\file.dll"));
    }

    #[test]
    fn test_program_files_is_protected() {
        assert!(is_protected_full_path("C:\\Program Files\\App"));
        assert!(is_protected_full_path("C:\\Program Files (x86)\\App"));
        assert!(is_protected_full_path("D:\\Program Files\\Something"));
    }

    #[test]
    fn test_user_folders_not_protected() {
        assert!(!is_protected_full_path("C:\\Users\\John\\Documents"));
        assert!(!is_protected_full_path("D:\\Projects\\myapp"));
        assert!(!is_protected_full_path("C:\\Data\\file.txt"));
    }

    // ==================== File Categorization Tests ====================

    #[test]
    fn test_categorize_system_files_mustkeep() {
        let (cat, score) = categorize_file("C:\\Windows\\System32\\kernel32.dll", "kernel32.dll", false, 1000);
        assert!(matches!(cat, FileCategory::MustKeep));
        assert_eq!(score, 100.0);

        let (cat, _) = categorize_file("C:\\pagefile.sys", "pagefile.sys", false, 1000);
        assert!(matches!(cat, FileCategory::MustKeep));
    }

    #[test]
    fn test_categorize_recycle_bin_mustkeep() {
        let (cat, score) = categorize_file("C:\\$RECYCLE.BIN", "$RECYCLE.BIN", true, 0);
        assert!(matches!(cat, FileCategory::MustKeep));
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_categorize_temp_files_useless() {
        let (cat, score) = categorize_file("C:\\temp\\file.tmp", "file.tmp", false, 100);
        assert!(matches!(cat, FileCategory::Useless));
        assert_eq!(score, 5.0);

        let (cat, _) = categorize_file("C:\\Users\\cache\\data", "cache", true, 0);
        assert!(matches!(cat, FileCategory::Useless));

        let (cat, _) = categorize_file("C:\\app.log", "app.log", false, 1000);
        assert!(matches!(cat, FileCategory::Useless));
    }

    #[test]
    fn test_categorize_system_dll_files() {
        let (cat, score) = categorize_file("C:\\app\\lib.dll", "lib.dll", false, 1000);
        assert!(matches!(cat, FileCategory::System));
        assert_eq!(score, 85.0);
    }

    #[test]
    fn test_categorize_photos_high_usefulness() {
        let (cat, score) = categorize_file("C:\\Photos\\vacation.jpg", "vacation.jpg", false, 5000000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 95.0);

        let (cat, score) = categorize_file("C:\\Photos\\image.png", "image.png", false, 1000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 95.0);
    }

    #[test]
    fn test_categorize_documents_high_usefulness() {
        let (cat, score) = categorize_file("C:\\Docs\\report.pdf", "report.pdf", false, 1000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 90.0);

        let (cat, score) = categorize_file("C:\\Docs\\letter.docx", "letter.docx", false, 1000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 90.0);
    }

    #[test]
    fn test_categorize_videos_size_dependent() {
        // Small video - high usefulness
        let (cat, score) = categorize_file("C:\\Videos\\clip.mp4", "clip.mp4", false, 100_000_000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 85.0);

        // Large video - lower usefulness
        let (cat, score) = categorize_file("C:\\Videos\\movie.mp4", "movie.mp4", false, 5_000_000_000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 70.0);
    }

    #[test]
    fn test_categorize_code_files() {
        let (cat, score) = categorize_file("C:\\Projects\\main.rs", "main.rs", false, 5000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 85.0);

        let (cat, score) = categorize_file("C:\\Projects\\app.py", "app.py", false, 1000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 85.0);
    }

    #[test]
    fn test_categorize_archives_size_dependent() {
        // Small archive
        let (cat, score) = categorize_file("C:\\Downloads\\file.zip", "file.zip", false, 10_000_000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 55.0);

        // Large archive
        let (cat, score) = categorize_file("C:\\Downloads\\huge.zip", "huge.zip", false, 2_000_000_000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 30.0);
    }

    #[test]
    fn test_categorize_iso_low_usefulness() {
        let (cat, score) = categorize_file("C:\\Downloads\\windows.iso", "windows.iso", false, 5_000_000_000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 25.0);
    }

    #[test]
    fn test_categorize_executables_in_downloads() {
        let (cat, score) = categorize_file("C:\\Downloads\\installer.exe", "installer.exe", false, 100_000_000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 35.0);
    }

    #[test]
    fn test_categorize_backup_files() {
        let (cat, score) = categorize_file("C:\\data.bak", "data.bak", false, 1000);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 40.0);

        let (cat, score) = categorize_file("C:\\backup_2024", "backup_2024", true, 0);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 40.0);
    }

    #[test]
    fn test_categorize_special_folders() {
        // node_modules - low usefulness
        let (cat, score) = categorize_file("C:\\project\\node_modules", "node_modules", true, 0);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 30.0);

        // Documents folder - high usefulness
        let (cat, score) = categorize_file("C:\\Users\\John\\Documents", "Documents", true, 0);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 95.0);

        // Downloads - medium
        let (cat, score) = categorize_file("C:\\Users\\John\\Downloads", "Downloads", true, 0);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 50.0);
    }

    // ==================== File Icon Tests ====================

    #[test]
    fn test_folder_icons() {
        assert_eq!(get_file_icon("folder", true, false, FileCategory::Regular), "📁");
        assert_eq!(get_file_icon("empty", true, true, FileCategory::Regular), "📂");
    }

    #[test]
    fn test_image_icons() {
        assert_eq!(get_file_icon("photo.jpg", false, false, FileCategory::Regular), "🖼️");
        assert_eq!(get_file_icon("image.png", false, false, FileCategory::Regular), "🖼️");
        assert_eq!(get_file_icon("icon.ico", false, false, FileCategory::Regular), "🖼️");
    }

    #[test]
    fn test_video_icons() {
        assert_eq!(get_file_icon("movie.mp4", false, false, FileCategory::Regular), "🎬");
        assert_eq!(get_file_icon("clip.mkv", false, false, FileCategory::Regular), "🎬");
    }

    #[test]
    fn test_audio_icons() {
        assert_eq!(get_file_icon("song.mp3", false, false, FileCategory::Regular), "🎵");
        assert_eq!(get_file_icon("audio.wav", false, false, FileCategory::Regular), "🎵");
    }

    #[test]
    fn test_document_icons() {
        assert_eq!(get_file_icon("doc.pdf", false, false, FileCategory::Regular), "📕");
        assert_eq!(get_file_icon("doc.docx", false, false, FileCategory::Regular), "📘");
        assert_eq!(get_file_icon("data.xlsx", false, false, FileCategory::Regular), "📗");
        assert_eq!(get_file_icon("slides.pptx", false, false, FileCategory::Regular), "📙");
        assert_eq!(get_file_icon("notes.txt", false, false, FileCategory::Regular), "📝");
    }

    #[test]
    fn test_code_icons() {
        assert_eq!(get_file_icon("main.rs", false, false, FileCategory::Regular), "💻");
        assert_eq!(get_file_icon("app.py", false, false, FileCategory::Regular), "💻");
        assert_eq!(get_file_icon("index.html", false, false, FileCategory::Regular), "🌐");
        assert_eq!(get_file_icon("config.json", false, false, FileCategory::Regular), "🌐");
    }

    #[test]
    fn test_archive_icons() {
        assert_eq!(get_file_icon("files.zip", false, false, FileCategory::Regular), "📦");
        assert_eq!(get_file_icon("backup.7z", false, false, FileCategory::Regular), "📦");
    }

    #[test]
    fn test_executable_icons() {
        assert_eq!(get_file_icon("app.exe", false, false, FileCategory::Regular), "⚡");
        assert_eq!(get_file_icon("script.bat", false, false, FileCategory::Regular), "⚡");
    }

    #[test]
    fn test_category_fallback_icons() {
        assert_eq!(get_file_icon("unknown.xyz", false, false, FileCategory::MustKeep), "🔒");
        assert_eq!(get_file_icon("driver.sys", false, false, FileCategory::System), "⚙️");
        assert_eq!(get_file_icon("file.dat", false, false, FileCategory::Regular), "📄");
        assert_eq!(get_file_icon("cache.dat", false, false, FileCategory::Useless), "🗑️");
    }

    // ==================== Sorting Tests ====================

    fn create_test_item(name: &str, size: u64, is_dir: bool, category: FileCategory, usefulness: f32) -> FileItem {
        FileItem {
            path: PathBuf::from(format!("C:\\{}", name)),
            name: name.to_string(),
            size,
            is_dir,
            category,
            usefulness,
            modified: None,
            child_count: None,
        }
    }

    #[test]
    fn test_sort_directories_first() {
        let dir = create_test_item("folder", 0, true, FileCategory::Regular, 50.0);
        let file = create_test_item("file.txt", 1000, false, FileCategory::Regular, 50.0);

        let result = compare_file_items(&dir, &file, SortColumn::Name, true);
        assert_eq!(result, std::cmp::Ordering::Less);

        let result = compare_file_items(&file, &dir, SortColumn::Name, true);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_sort_by_name_ascending() {
        let a = create_test_item("apple", 100, false, FileCategory::Regular, 50.0);
        let b = create_test_item("banana", 100, false, FileCategory::Regular, 50.0);

        let result = compare_file_items(&a, &b, SortColumn::Name, true);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    #[test]
    fn test_sort_by_name_descending() {
        let a = create_test_item("apple", 100, false, FileCategory::Regular, 50.0);
        let b = create_test_item("banana", 100, false, FileCategory::Regular, 50.0);

        let result = compare_file_items(&a, &b, SortColumn::Name, false);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_sort_by_size_ascending() {
        let small = create_test_item("small.txt", 100, false, FileCategory::Regular, 50.0);
        let large = create_test_item("large.txt", 10000, false, FileCategory::Regular, 50.0);

        let result = compare_file_items(&small, &large, SortColumn::Size, true);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    #[test]
    fn test_sort_by_size_descending() {
        let small = create_test_item("small.txt", 100, false, FileCategory::Regular, 50.0);
        let large = create_test_item("large.txt", 10000, false, FileCategory::Regular, 50.0);

        let result = compare_file_items(&small, &large, SortColumn::Size, false);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_sort_by_category() {
        let mustkeep = create_test_item("system", 100, false, FileCategory::MustKeep, 100.0);
        let useless = create_test_item("temp", 100, false, FileCategory::Useless, 5.0);

        let result = compare_file_items(&mustkeep, &useless, SortColumn::Category, true);
        assert_eq!(result, std::cmp::Ordering::Less);
    }

    #[test]
    fn test_sort_by_usefulness() {
        let high = create_test_item("important", 100, false, FileCategory::Regular, 95.0);
        let low = create_test_item("junk", 100, false, FileCategory::Regular, 25.0);

        let result = compare_file_items(&low, &high, SortColumn::Usefulness, true);
        assert_eq!(result, std::cmp::Ordering::Less);

        let result = compare_file_items(&low, &high, SortColumn::Usefulness, false);
        assert_eq!(result, std::cmp::Ordering::Greater);
    }

    // ==================== Search/Filter Tests ====================

    #[test]
    fn test_filter_empty_query_returns_all() {
        let items = vec![
            create_test_item("file1.txt", 100, false, FileCategory::Regular, 50.0),
            create_test_item("file2.txt", 200, false, FileCategory::Regular, 50.0),
        ];

        let filtered = filter_items(&items, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_name() {
        let items = vec![
            create_test_item("document.pdf", 100, false, FileCategory::Regular, 50.0),
            create_test_item("image.png", 200, false, FileCategory::Regular, 50.0),
            create_test_item("document_backup.pdf", 300, false, FileCategory::Regular, 50.0),
        ];

        let filtered = filter_items(&items, "document");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|i| i.name.contains("document")));
    }

    #[test]
    fn test_filter_case_insensitive() {
        let items = vec![
            create_test_item("Document.PDF", 100, false, FileCategory::Regular, 50.0),
            create_test_item("IMAGE.PNG", 200, false, FileCategory::Regular, 50.0),
        ];

        let filtered = filter_items(&items, "document");
        assert_eq!(filtered.len(), 1);

        let filtered = filter_items(&items, "DOCUMENT");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_no_matches() {
        let items = vec![
            create_test_item("file1.txt", 100, false, FileCategory::Regular, 50.0),
            create_test_item("file2.txt", 200, false, FileCategory::Regular, 50.0),
        ];

        let filtered = filter_items(&items, "xyz");
        assert_eq!(filtered.len(), 0);
    }

    // ==================== Multi-Selection Tests ====================

    #[test]
    fn test_hashset_selection() {
        let mut selected: HashSet<PathBuf> = HashSet::new();

        // Add items
        selected.insert(PathBuf::from("C:\\file1.txt"));
        selected.insert(PathBuf::from("C:\\file2.txt"));
        assert_eq!(selected.len(), 2);

        // Toggle (remove existing)
        selected.remove(&PathBuf::from("C:\\file1.txt"));
        assert_eq!(selected.len(), 1);
        assert!(!selected.contains(&PathBuf::from("C:\\file1.txt")));
        assert!(selected.contains(&PathBuf::from("C:\\file2.txt")));

        // Clear
        selected.clear();
        assert_eq!(selected.len(), 0);
    }

    #[test]
    fn test_range_selection() {
        let items = vec![
            create_test_item("file0.txt", 100, false, FileCategory::Regular, 50.0),
            create_test_item("file1.txt", 100, false, FileCategory::Regular, 50.0),
            create_test_item("file2.txt", 100, false, FileCategory::Regular, 50.0),
            create_test_item("file3.txt", 100, false, FileCategory::Regular, 50.0),
            create_test_item("file4.txt", 100, false, FileCategory::Regular, 50.0),
        ];

        let mut selected: HashSet<PathBuf> = HashSet::new();
        let anchor = 1;
        let end = 3;

        // Select range from anchor to end
        for idx in anchor..=end {
            selected.insert(items[idx].path.clone());
        }

        assert_eq!(selected.len(), 3);
        assert!(selected.contains(&items[1].path));
        assert!(selected.contains(&items[2].path));
        assert!(selected.contains(&items[3].path));
    }

    // ==================== FileItem Tests ====================

    #[test]
    fn test_file_item_empty_folder_detection() {
        let empty_folder = FileItem {
            path: PathBuf::from("C:\\empty"),
            name: "empty".to_string(),
            size: 0,
            is_dir: true,
            category: FileCategory::Regular,
            usefulness: 50.0,
            modified: None,
            child_count: Some(0),
        };
        assert!(empty_folder.child_count == Some(0));

        let non_empty_folder = FileItem {
            path: PathBuf::from("C:\\full"),
            name: "full".to_string(),
            size: 1000,
            is_dir: true,
            category: FileCategory::Regular,
            usefulness: 50.0,
            modified: None,
            child_count: Some(5),
        };
        assert!(non_empty_folder.child_count != Some(0));
    }

    // ==================== Navigation Tests ====================

    #[test]
    fn test_navigation_forward() {
        let history = vec![
            PathBuf::from("C:\\"),
            PathBuf::from("C:\\Users"),
            PathBuf::from("C:\\Users\\John"),
        ];
        let mut index = 1; // Currently at Users

        // Go forward
        if index < history.len() - 1 {
            index += 1;
        }
        assert_eq!(index, 2);
        assert_eq!(history[index], PathBuf::from("C:\\Users\\John"));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_filename() {
        let (cat, score) = categorize_file("C:\\", "", true, 0);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 65.0); // Default folder usefulness
    }

    #[test]
    fn test_very_large_file_size() {
        let huge_size: u64 = 10_000_000_000_000; // 10 TB
        let (cat, score) = categorize_file("C:\\huge.dat", "huge.dat", false, huge_size);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 45.0); // Large unknown file
    }

    #[test]
    fn test_deep_nested_path() {
        let deep_path = "C:\\a\\b\\c\\d\\e\\f\\g\\h\\i\\j\\file.txt";
        let (cat, score) = categorize_file(deep_path, "file.txt", false, 100);
        assert!(matches!(cat, FileCategory::Regular));
        assert_eq!(score, 90.0); // txt file
    }

    #[test]
    fn test_special_characters_in_name() {
        let (cat, _) = categorize_file("C:\\file (1).txt", "file (1).txt", false, 100);
        assert!(matches!(cat, FileCategory::Regular));

        let (cat, _) = categorize_file("C:\\file [backup].txt", "file [backup].txt", false, 100);
        assert!(matches!(cat, FileCategory::Regular));
    }
}
