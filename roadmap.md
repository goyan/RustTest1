# Disk Dashboard - Roadmap

## Project Management

### Version History
| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2024-12 | Initial release with disk monitoring |
| 0.2.0 | 2024-12 | File browser, delete, navigation |
| 0.3.0 | 2024-12 | Recursive folder sizes, file type icons |
| 0.4.0 | 2024-12 | Async folder sizes, smart usefulness, file open |

### Estimated Development Cost
- AI Development: ~$30-40 USD (Claude Opus 4.5)
- Estimated Tokens: ~800k input, ~200k output
- Development Sessions: 3-4 sessions
- Lines of Code: ~1930

---

## Phase 1: Core Features (Complete)
- [x] Disk monitoring with usage display
- [x] File browser with navigation
- [x] Sort by Name, Size, Category, Usefulness
- [x] File categorization and scoring
- [x] Delete with confirmation
- [x] Protected system folders
- [x] Toast notifications

## Phase 2: Enhanced UX (Complete)
- [x] Pie chart visualization
- [x] Progress bars for relative sizes
- [x] File type icons (images, videos, code, etc.)
- [x] Empty folder detection
- [x] Search/filter
- [x] Keyboard shortcuts
- [x] Click to open files with default app

## Phase 3: Performance (Complete)
- [x] Recursive folder size calculation
- [x] Size caching with HashMap
- [x] Depth-limited recursion
- [x] **Async background calculation** (non-blocking UI)
- [ ] Progress indicator during calculation
- [ ] Lazy loading for large directories

## Phase 4: Smart Features (Planned)
- [x] **Smart usefulness scoring** (file type, size, location)
- [ ] AI-powered cleanup suggestions
- [ ] Duplicate file detection
- [ ] Large file finder
- [ ] Old/unused file detection
- [ ] Disk space visualization (treemap)

## Phase 5: Advanced (Future)
- [ ] Multiple file selection
- [ ] Bulk operations
- [ ] File preview
- [ ] Disk health monitoring
- [ ] Export reports (CSV/JSON)
- [ ] Custom themes

---

## Known Issues
| Priority | Issue | Status |
|----------|-------|--------|
| Medium | Some system folders show 0 size | Access restrictions |
| Low | Column alignment on resize | Minor |

## Technical Debt
- [ ] Refactor render_file_item (too long)
- [ ] Add more unit tests
- [ ] Error handling improvements
- [ ] Documentation

---

## Sprint Notes

### Current Sprint (Complete)
- Fix progress bar positioning ✓
- Add file type icons ✓
- Implement async folder size calculation ✓
- Smart usefulness scoring ✓
- Click to open files ✓
- Fix Windows folder category ✓

### Next Sprint
- Add progress indicator for folder sizes
- E2E test automation refinement
- Improve large directory handling
