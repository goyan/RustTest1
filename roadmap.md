# Disk Dashboard - Roadmap

## Project Management

### Version History
| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2024-12 | Initial release with disk monitoring |
| 0.2.0 | 2024-12 | File browser, delete, navigation |
| 0.3.0 | 2024-12 | Recursive folder sizes, file type icons |

### Estimated Development Cost
- AI Development: ~$30-50 USD (Claude Opus 4.5)
- Development Time: ~4-6 hours
- Lines of Code: ~1800

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

## Phase 3: Performance (In Progress)
- [x] Recursive folder size calculation
- [x] Size caching with HashMap
- [x] Depth-limited recursion
- [ ] **Async background calculation**
- [ ] Progress indicator during calculation
- [ ] Lazy loading for large directories

## Phase 4: Smart Features (Planned)
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
| High | Large folders cause brief UI freeze | Needs async |
| Medium | Some system folders show 0 size | Access restrictions |
| Low | Column alignment on resize | Minor |

## Technical Debt
- [ ] Refactor render_file_item (too long)
- [ ] Add more unit tests
- [ ] Error handling improvements
- [ ] Documentation

---

## Sprint Notes

### Current Sprint
- Fix progress bar positioning ✓
- Add file type icons ✓
- Create roadmap.md ✓

### Next Sprint
- Implement async folder size calculation
- Add progress indicator
- Improve large directory handling
