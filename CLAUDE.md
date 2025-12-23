# Claude Code Instructions

## Project Overview
Disk Dashboard - A Rust/egui application for disk monitoring and file management.

## Key Files
- `src/main.rs` - Main application code (~1930 lines)
- `reqs.md` - Requirements and feature tracking
- `roadmap.md` - Project roadmap and PM information
- `e2e_test.py` - E2E test script (pyautogui)

## Current Features (v0.4.0)
- Disk monitoring with pie chart visualization
- File browser with sorting (Name, Size, Category, Usefulness)
- **Async folder size calculation** (background threads, non-blocking)
- **Smart usefulness scoring** (based on file type, size, location)
- **Click to open files** with default application
- File type icons (images, videos, code, archives, etc.)
- Protected system folders (Windows, Program Files, etc.)
- Delete with confirmation dialog
- Search/filter (Ctrl+F)
- Keyboard shortcuts (Alt+Arrow, Ctrl+Home)

## Before Making Changes
1. Read `reqs.md` for current features and pending items
2. Check existing code patterns in `src/main.rs`
3. Run `cargo build --release` to verify changes
4. Test with `./target/release/disk-dashboard.exe`

## Code Style
- Use egui immediate mode patterns
- Detect hover BEFORE rendering (not after)
- Use fixed-width columns for alignment
- Cache expensive calculations (folder sizes in HashMap)
- Use async/threads for blocking operations

## Architecture Notes
- `DiskDashboard` struct holds all state
- `size_sender/receiver` - mpsc channel for async folder sizes
- `pending_size_calculations` - HashSet tracking in-progress calculations
- `folder_size_cache` - HashMap for computed sizes

## Common Tasks
- Add feature: Update both code and reqs.md
- Fix bug: Test thoroughly, update if needed
- UI change: Run E2E test to verify

## Testing
```powershell
cargo test                    # Run unit tests
cargo build --release         # Release build
python e2e_test.py           # E2E UI test (captures screenshots)
```

## E2E Test
The `e2e_test.py` script:
- Uses pyautogui for automation
- Dynamic coordinates (works at any window size)
- Captures screenshots to `test_*.png`
- Tests: disk selection, sorting, navigation, hover

## Development Cost
- AI Development: ~$30-40 USD (Claude Opus 4.5)
- Lines of Code: ~1930
- See `roadmap.md` for full details
