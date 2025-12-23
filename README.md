# Disk Capacity Dashboard

A modern Rust application for real-time disk capacity monitoring and file management with a beautiful dark-themed UI.

## Features

### Disk Monitoring
- Real-time disk space monitoring with auto-refresh
- Multi-disk support showing all connected drives
- Drive names displayed (e.g., "C:\ (Windows)")
- Color-coded usage alerts (green < 75%, orange 75-90%, red > 90%)
- Pie chart visualization of total disk usage

### File Browser
- Browse files and folders on any disk
- Recursive folder size calculation with caching
- Full-width background progress bars showing relative sizes
- File categorization (MustKeep, System, Regular, Useless)
- Usefulness scoring for cleanup suggestions
- Empty folder detection with visual indicators

### File Management
- Context menu with delete option
- Protected system folder detection ($RECYCLE.BIN, System Volume Information, etc.)
- Confirmation dialogs before deletion
- Toast notifications for user feedback

### Navigation
- Breadcrumb navigation
- Back/Forward navigation (Alt+Arrow, mouse buttons)
- Search/filter files (Ctrl+F)
- Sortable columns (Name, Size, Category, Usefulness)

## Installation

### Prerequisites
- Rust toolchain (install from https://rustup.rs/)

### Build and Run

```powershell
# Clone the repository
git clone https://github.com/goyan/RustTest1.git
cd RustTest1

# Build release version
cargo build --release

# Run the application
./target/release/disk-dashboard.exe
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Alt+Left | Navigate back |
| Alt+Right | Navigate forward |
| Ctrl+Home | Go to home (disk selection) |
| Ctrl+F | Focus search |

## Project Structure

```
RustTest1/
├── Cargo.toml          # Dependencies
├── src/
│   └── main.rs         # Main application (~1700 lines)
├── README.md           # This file
└── .gitignore
```

## Dependencies

- **eframe/egui** - Immediate mode GUI framework
- **sysinfo** - System information gathering

## Development

This project was developed with AI assistance using Claude Code (Opus 4.5).

### Estimated AI Development Cost
- Session tokens used: ~45GB context
- Estimated API cost: ~$30-50 USD
- Development time: ~4 hours of iterative development

### Key Implementation Details
- Recursive folder size calculation with HashMap caching
- Custom pie chart rendering using egui painter
- Hover detection before rendering for proper immediate-mode UI
- Fixed-width column layout for header/content alignment

## Unit Tests

Run tests with:
```powershell
cargo test
```

16 tests covering:
- Size formatting
- Empty folder navigation blocking
- Protected path detection
- Toast notifications
- Navigation history

## License

Open source for personal use.
