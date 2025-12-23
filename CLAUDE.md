# Claude Code Instructions

## Project Overview
Disk Dashboard - A Rust/egui application for disk monitoring and file management.

## Key Files
- `src/main.rs` - Main application code (~1800 lines)
- `reqs.md` - Requirements and feature tracking
- `roadmap.md` - Project roadmap and PM information
- `README.md` - User documentation

## Before Making Changes
1. Read `reqs.md` for current features and pending items
2. Check existing code patterns in `src/main.rs`
3. Run `cargo build --release` to verify changes
4. Test with `./target/release/disk-dashboard.exe`

## Code Style
- Use egui immediate mode patterns
- Detect hover BEFORE rendering (not after)
- Use fixed-width columns for alignment
- Cache expensive calculations (folder sizes)

## Common Tasks
- Add feature: Update both code and reqs.md
- Fix bug: Test thoroughly, update if needed
- UI change: Take screenshot to verify

## Testing
```powershell
cargo test        # Run unit tests
cargo build       # Debug build
cargo build --release  # Release build
```

## Screenshot Tool
```powershell
powershell -ExecutionPolicy Bypass -File screenshot.ps1
```
