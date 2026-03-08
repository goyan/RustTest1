import { useState, useEffect, useCallback, useRef } from "react";
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

  const loadDirectory = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<FileItem[]>("load_directory", { path });
      setItems(result);
      setSelectedItems(new Set());
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

  // Async folder size calculation
  useEffect(() => {
    const dirs = items.filter((item) => item.is_dir && item.size === 0);
    dirs.forEach(async (item) => {
      if (folderSizes.has(item.path)) return;
      try {
        const size = await invoke<number>("get_folder_size", { path: item.path });
        setFolderSizes((prev) => new Map(prev).set(item.path, size));
      } catch {
        // Ignore errors for inaccessible folders
      }
    });
  }, [items, folderSizes]);

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
  const displayItems = (() => {
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
  })();

  const maxSize = Math.max(...displayItems.map((i) => i.size), 1);

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
    return sortDirection === "Ascending" ? " ▲" : " ▼";
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
        onToast("📂 This folder is empty");
      } else {
        onNavigate(item.path);
      }
    } else {
      try {
        await invoke("open_file", { path: item.path });
        onToast(`📄 Opening ${item.name}`, 1500);
      } catch (err) {
        onToast(`Failed to open: ${err}`, 3000);
      }
    }
  };

  const handleDelete = async (item: FileItem) => {
    try {
      await invoke("delete_file", { path: item.path });
      onToast(`🗑️ Deleted ${item.name}`);
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
        onToast(`🗑️ Deleted ${result.deleted} items`);
      } else if (result.skipped > 0) {
        onToast(`🔒 Skipped ${result.skipped} protected, deleted ${result.deleted}`, 3000);
      } else {
        onToast(`⚠️ Deleted ${result.deleted} items, ${result.errors.length} failed`, 3000);
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
      <div className="p-3 bg-[#191920] rounded-lg m-2 mb-0">
        <Breadcrumb path={path} onNavigate={onNavigate} />
        <div className="mt-2.5">
          <SearchBar
            ref={searchRef}
            value={searchQuery}
            onChange={setSearchQuery}
            resultCount={searchQuery ? displayItems.length : undefined}
          />
        </div>
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-2 px-4 py-2 text-sm">
        {parentPath && (
          <button
            onClick={() => onNavigate(parentPath)}
            className="px-2 py-1 text-sm hover:bg-[#1e1432] rounded transition-colors text-gray-300"
          >
            ⬆️ .. ({parentPath})
          </button>
        )}
        <div className="w-px h-5 bg-[#2d3037]" />
        <button
          onClick={toggleSelectAll}
          className="px-2 py-1 bg-[#281e3c] hover:bg-[#3c2860] rounded transition-colors"
        >
          {allSelected ? "☐ Deselect All" : "☑ Select All"}
        </button>
        {selectedItems.size > 0 && (
          <>
            <div className="w-px h-5 bg-[#2d3037]" />
            <span className="text-cyber-cyan text-sm">📋 {selectedItems.size} selected</span>
            <button
              onClick={() => setPendingBatchDelete(true)}
              className="px-2 py-1 bg-[#b41e3c] hover:bg-[#d4264a] text-white font-semibold rounded transition-colors"
            >
              🗑️ Delete Selection
            </button>
            <button
              onClick={() => setSelectedItems(new Set())}
              className="px-2 py-1 bg-[#281e3c] hover:bg-[#3c2860] rounded transition-colors"
            >
              ❌ Clear
            </button>
          </>
        )}
      </div>

      {/* Column headers */}
      <div className="flex items-center px-4 py-2 bg-[#191920] mx-2 rounded text-sm">
        <div className="w-8" /> {/* checkbox space */}
        <div className="w-8" /> {/* icon space */}
        <button onClick={() => toggleSort("Name")} className="flex-1 text-left hover:text-cyber-cyan transition-colors">
          Name{sortArrow("Name")}
        </button>
        <button onClick={() => toggleSort("Size")} className="w-20 text-center hover:text-cyber-cyan transition-colors">
          Size{sortArrow("Size")}
        </button>
        <button onClick={() => toggleSort("Category")} className="w-24 text-left hover:text-cyber-cyan transition-colors">
          Cat{sortArrow("Category")}
        </button>
        <button onClick={() => toggleSort("Usefulness")} className="w-16 text-center hover:text-cyber-cyan transition-colors">
          Use{sortArrow("Usefulness")}
        </button>
      </div>

      <div className="border-b border-[#2d3037] mx-2" />

      {/* File list */}
      <div className="flex-1 overflow-y-auto px-2 py-1">
        {loading ? (
          <div className="flex items-center justify-center h-32 text-gray-400">
            <span className="animate-spin mr-2">⏳</span> Loading...
          </div>
        ) : displayItems.length === 0 ? (
          <div className="flex items-center justify-center h-32 text-cyber-dim-purple">
            {searchQuery ? `No results for "${searchQuery}"` : "No files found"}
          </div>
        ) : (
          displayItems.map((item) => (
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
