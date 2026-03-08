import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { FileItem, SortColumn, SortDirection, BatchDeleteResult } from "../types";
import { FileItemRow } from "./FileItem";
import { Breadcrumb } from "./Breadcrumb";
import { SearchBar } from "./SearchBar";
import { DeleteDialog } from "./dialogs/DeleteDialog";
import { BatchDeleteDialog } from "./dialogs/BatchDeleteDialog";
import { ErrorDialog } from "./dialogs/ErrorDialog";

interface FileBrowserProps {
  path: string;
  onNavigate: (path: string) => void;
  onToast: (message: string, duration?: number) => void;
}

function sortItems(items: FileItem[], column: SortColumn, direction: SortDirection): FileItem[] {
  return [...items].sort((a, b) => {
    // Directories always first
    if (a.is_dir && !b.is_dir) return -1;
    if (!a.is_dir && b.is_dir) return 1;

    let cmp = 0;
    switch (column) {
      case "Name":
        cmp = a.name.toLowerCase().localeCompare(b.name.toLowerCase());
        break;
      case "Size":
        cmp = a.size - b.size;
        break;
      case "Category": {
        const order: Record<string, number> = { MustKeep: 0, System: 1, Regular: 2, Useless: 3, Unknown: 4 };
        cmp = (order[a.category] ?? 4) - (order[b.category] ?? 4);
        break;
      }
      case "Usefulness":
        cmp = a.usefulness - b.usefulness;
        break;
    }
    return direction === "Ascending" ? cmp : -cmp;
  });
}

export function FileBrowser({ path, onNavigate, onToast }: FileBrowserProps) {
  const [items, setItems] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [sortColumn, setSortColumn] = useState<SortColumn>("Size");
  const [sortDirection, setSortDirection] = useState<SortDirection>("Descending");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [pendingDelete, setPendingDelete] = useState<FileItem | null>(null);
  const [pendingBatchDelete, setPendingBatchDelete] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [folderSizes, setFolderSizes] = useState<Map<string, number>>(new Map());
  const searchRef = useRef<HTMLInputElement>(null);
  const requestedSizes = useRef(new Set<string>());

  const loadDirectory = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<FileItem[]>("load_directory", { path });
      setItems(result);
      setSelectedItems(new Set());
      setFolderSizes(new Map());
      setSearchQuery("");
    } catch (err) {
      console.error("Failed to load directory:", err);
    } finally {
      setLoading(false);
    }
  }, [path]);

  useEffect(() => {
    loadDirectory();
  }, [loadDirectory]);

  // Reset folder size tracking on path change
  useEffect(() => {
    requestedSizes.current.clear();
    setFolderSizes(new Map());
  }, [path]);

  // Async folder size calculation
  useEffect(() => {
    let cancelled = false;
    const dirs = items.filter((item) => item.is_dir && item.size === 0);

    dirs.forEach(async (item) => {
      if (requestedSizes.current.has(item.path)) return;
      requestedSizes.current.add(item.path);
      try {
        const size = await invoke<number>("get_folder_size", { path: item.path });
        if (!cancelled) {
          setFolderSizes((prev) => new Map(prev).set(item.path, size));
        }
      } catch {
        // Ignore errors for inaccessible folders
      }
    });

    return () => { cancelled = true; };
  }, [items]);

  // Keyboard shortcut: Ctrl+F to focus search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "f") {
        e.preventDefault();
        searchRef.current?.focus();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Apply folder sizes to items, filter, and sort
  const displayItems = useMemo(() => {
    let processed = items.map((item) => {
      if (item.is_dir && item.size === 0 && folderSizes.has(item.path)) {
        return { ...item, size: folderSizes.get(item.path)! };
      }
      return item;
    });

    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      processed = processed.filter((item) =>
        item.name.toLowerCase().includes(q)
      );
    }

    return sortItems(processed, sortColumn, sortDirection);
  }, [items, folderSizes, searchQuery, sortColumn, sortDirection]);

  const maxSize = useMemo(() => {
    let max = 1;
    for (const i of displayItems) {
      if (i.size > max) max = i.size;
    }
    return max;
  }, [displayItems]);

  const toggleSort = (column: SortColumn) => {
    if (sortColumn === column) {
      setSortDirection((d) => d === "Ascending" ? "Descending" : "Ascending");
    } else {
      setSortColumn(column);
      setSortDirection("Ascending");
    }
  };

  const sortArrow = (column: SortColumn) => {
    if (sortColumn !== column) return "";
    return sortDirection === "Ascending" ? " \u25B2" : " \u25BC";
  };

  const toggleSelectAll = () => {
    const allSelected = displayItems.length > 0 && displayItems.every((i) => selectedItems.has(i.path));
    if (allSelected) {
      setSelectedItems(new Set());
    } else {
      setSelectedItems(new Set(displayItems.map((i) => i.path)));
    }
  };

  const toggleSelect = (path: string) => {
    setSelectedItems((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const handleFileClick = async (item: FileItem) => {
    if (item.is_dir) {
      if (item.child_count === 0) {
        onToast("\uD83D\uDCC2 This folder is empty");
      } else {
        onNavigate(item.path);
      }
    } else {
      try {
        await invoke("open_file", { path: item.path });
        onToast(`\uD83D\uDCC4 Opening ${item.name}`, 1500);
      } catch (err) {
        onToast(`Failed to open: ${err}`, 3000);
      }
    }
  };

  const handleDelete = async (item: FileItem) => {
    try {
      await invoke("delete_file", { path: item.path });
      onToast(`\uD83D\uDDD1\uFE0F Deleted ${item.name}`);
      loadDirectory();
    } catch (err) {
      setError(`Failed to delete: ${err}`);
    }
    setPendingDelete(null);
  };

  const handleBatchDelete = async () => {
    try {
      const paths = Array.from(selectedItems);
      const result = await invoke<BatchDeleteResult>("batch_delete", { paths });
      if (result.errors.length === 0 && result.skipped === 0) {
        onToast(`\uD83D\uDDD1\uFE0F Deleted ${result.deleted} items`);
      } else if (result.skipped > 0) {
        onToast(`\uD83D\uDD12 Skipped ${result.skipped} protected, deleted ${result.deleted}`, 3000);
      } else {
        onToast(`\u26A0\uFE0F Deleted ${result.deleted} items, ${result.errors.length} failed`, 3000);
      }
      setSelectedItems(new Set());
      loadDirectory();
    } catch (err) {
      setError(`Batch delete failed: ${err}`);
    }
    setPendingBatchDelete(false);
  };

  const parentPath = (() => {
    const parts = path.replace(/[\\/]+$/, "").split(/[\\/]/);
    if (parts.length <= 1) return null;
    parts.pop();
    const parent = parts.join("\\");
    return parent ? (parent.endsWith("\\") ? parent : parent + "\\") : null;
  })();

  const allSelected = displayItems.length > 0 && displayItems.every((i) => selectedItems.has(i.path));

  return (
    <div className="flex flex-col h-full">
      {/* Header with breadcrumb and search */}
      <div className="p-4 glass-strong rounded-xl m-3 mb-0 animate-fade-in">
        <Breadcrumb path={path} onNavigate={onNavigate} />
        <div className="mt-3">
          <SearchBar
            ref={searchRef}
            value={searchQuery}
            onChange={setSearchQuery}
            resultCount={searchQuery ? displayItems.length : undefined}
          />
        </div>
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-2 px-5 py-2.5 text-sm animate-fade-in-delay-1">
        {parentPath && (
          <button
            onClick={() => onNavigate(parentPath)}
            className="px-3 py-1.5 text-xs btn-glass rounded-lg text-gray-300 hover:text-cyan-300 transition-all"
          >
            &#x2B06;&#xFE0F; .. ({parentPath})
          </button>
        )}
        <div className="w-px h-5 bg-white/[0.06]" />
        <button
          onClick={toggleSelectAll}
          className="px-3 py-1.5 text-xs btn-glass rounded-lg text-gray-300 hover:text-cyan-300 transition-all"
        >
          {allSelected ? "\u2610 Deselect All" : "\u2611 Select All"}
        </button>
        {selectedItems.size > 0 && (
          <>
            <div className="w-px h-5 bg-white/[0.06]" />
            <span className="text-cyan-400/80 text-xs font-medium px-2 py-1 rounded-md bg-cyan-500/10">
              {selectedItems.size} selected
            </span>
            <button
              onClick={() => setPendingBatchDelete(true)}
              className="px-3 py-1.5 text-xs font-semibold bg-red-500/20 hover:bg-red-500/30 text-red-300
                         border border-red-500/30 hover:border-red-500/50 rounded-lg transition-all"
            >
              &#x1F5D1;&#xFE0F; Delete Selection
            </button>
            <button
              onClick={() => setSelectedItems(new Set())}
              className="px-3 py-1.5 text-xs btn-glass rounded-lg text-gray-400 hover:text-gray-200 transition-all"
            >
              &#x2715; Clear
            </button>
          </>
        )}
      </div>

      {/* Column headers */}
      <div className="flex items-center px-5 py-2 glass-subtle mx-3 rounded-lg text-[11px] font-semibold uppercase tracking-wider text-gray-500 animate-fade-in-delay-2">
        <div className="w-8" />
        <div className="w-8" />
        <button onClick={() => toggleSort("Name")} className="flex-1 text-left hover:text-cyan-400 transition-colors duration-200">
          Name{sortArrow("Name")}
        </button>
        <button onClick={() => toggleSort("Size")} className="w-20 text-center hover:text-cyan-400 transition-colors duration-200">
          Size{sortArrow("Size")}
        </button>
        <button onClick={() => toggleSort("Category")} className="w-24 text-left hover:text-cyan-400 transition-colors duration-200">
          Category{sortArrow("Category")}
        </button>
        <button onClick={() => toggleSort("Usefulness")} className="w-16 text-center hover:text-cyan-400 transition-colors duration-200">
          Use{sortArrow("Usefulness")}
        </button>
      </div>

      <div className="mx-3 h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent" />

      {/* File list */}
      <div className="flex-1 overflow-y-auto px-3 py-2">
        {loading ? (
          /* Skeleton loading */
          <div className="space-y-2 animate-fade-in">
            {Array.from({ length: 8 }).map((_, i) => (
              <div key={i} className="flex items-center gap-3 px-3 py-3 rounded-lg" style={{ animationDelay: `${i * 0.05}s` }}>
                <div className="skeleton w-4 h-4 rounded" />
                <div className="skeleton w-6 h-6 rounded" />
                <div className="skeleton flex-1 h-4 rounded" style={{ maxWidth: `${60 + Math.random() * 30}%` }} />
                <div className="skeleton w-14 h-3 rounded" />
                <div className="skeleton w-16 h-4 rounded-full" />
                <div className="skeleton w-10 h-3 rounded" />
              </div>
            ))}
          </div>
        ) : displayItems.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-40 animate-fade-in">
            <div className="glass rounded-2xl p-8 text-center">
              <div className="text-3xl mb-3 opacity-30">
                {searchQuery ? "\uD83D\uDD0D" : "\uD83D\uDCC2"}
              </div>
              <p className="text-sm text-cyber-dim-purple">
                {searchQuery ? `No results for "${searchQuery}"` : "This folder is empty"}
              </p>
            </div>
          </div>
        ) : (
          displayItems.map((item, index) => (
            <FileItemRow
              key={item.path}
              item={item}
              isSelected={selectedItems.has(item.path)}
              maxSize={maxSize}
              isCalculating={item.is_dir && item.size === 0 && !folderSizes.has(item.path)}
              onToggleSelect={() => toggleSelect(item.path)}
              onClick={() => handleFileClick(item)}
              onDelete={() => setPendingDelete(item)}
              onOpenInExplorer={async () => {
                try { await invoke("open_in_explorer", { path: item.path }); } catch {}
              }}
              animationDelay={Math.min(index * 0.02, 0.3)}
            />
          ))
        )}
      </div>

      {/* Dialogs */}
      {pendingDelete && (
        <DeleteDialog
          item={pendingDelete}
          onConfirm={() => handleDelete(pendingDelete)}
          onCancel={() => setPendingDelete(null)}
        />
      )}
      {pendingBatchDelete && (
        <BatchDeleteDialog
          selectedPaths={Array.from(selectedItems)}
          onConfirm={handleBatchDelete}
          onCancel={() => setPendingBatchDelete(false)}
        />
      )}
      {error && (
        <ErrorDialog
          message={error}
          onClose={() => setError(null)}
        />
      )}
    </div>
  );
}
