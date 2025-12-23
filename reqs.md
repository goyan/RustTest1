# Disk Dashboard - Requirements & Features

## Current Features

### Disk Panel (Left)
- [x] Show all connected disks with drive letter and name
- [x] Display usage percentage with color coding (green/orange/red)
- [x] Progress bar for each disk
- [x] Summary section with total disks, space, usage
- [x] Pie chart visualization
- [x] Scrollable panel

### File Browser (Right)
- [x] Browse files and folders
- [x] Sort by Name, Size, Category, Usefulness (with ▲▼ indicators)
- [x] File type icons based on extension
- [x] Folder icons (📁 non-empty, 📂 empty)
- [x] Recursive folder size calculation (cached, depth-limited)
- [x] Full-width progress bars showing relative size
- [x] Category badges (MustKeep, System, Regular, Useless)
- [x] Usefulness scoring
- [x] Search/filter (Ctrl+F)
- [x] Context menu with delete option
- [x] Protected system folder detection
- [x] Toast notifications
- [x] Empty folder detection and blocking

### Navigation
- [x] Breadcrumb path display
- [x] Back/Forward (Alt+Arrow)
- [x] Home button (Ctrl+Home)
- [x] Click to navigate into folders
- [x] Parent folder navigation (..)

## Pending Features

### High Priority
- [ ] Async folder size calculation (background thread)
- [ ] Progress indicator while calculating sizes
- [ ] Disk space analysis/visualization
- [ ] Smart cleanup suggestions (AI-powered)

### Medium Priority
- [ ] Multiple file selection
- [ ] Bulk delete
- [ ] File preview panel
- [ ] Disk health monitoring
- [ ] Export data to CSV/JSON

### Low Priority
- [ ] Custom themes
- [ ] Keyboard navigation in file list
- [ ] Drag and drop
- [ ] File copy/move operations
- [ ] Historical usage graphs

## Technical Requirements

### Build
- Rust 1.70+
- Windows 10/11
- cargo build --release

### Dependencies
- eframe/egui - GUI framework
- sysinfo - System information

### Performance
- Folder size depth limit: 2 levels
- Size cache for efficiency
- Skip system folders during calculation

## Known Issues
- Large folders may cause brief UI freeze during size calculation
- Some system folders may show incorrect sizes due to access restrictions
