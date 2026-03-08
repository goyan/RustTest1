import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DiskPanel } from "./components/DiskPanel";
import { FileBrowser } from "./components/FileBrowser";
import { Toast } from "./components/Toast";
import type { DiskInfo } from "./types";

function App() {
  const [disks, setDisks] = useState<DiskInfo[]>([]);
  const [currentPath, setCurrentPath] = useState<string | null>(null);
  const [currentDisk, setCurrentDisk] = useState<string | null>(null);
  const [toast, setToast] = useState<{ message: string; duration: number } | null>(null);
  const [navigationHistory, setNavigationHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);

  const showToast = useCallback((message: string, duration = 2000) => {
    setToast({ message, duration });
    setTimeout(() => setToast(null), duration);
  }, []);

  const refreshDisks = useCallback(async () => {
    try {
      const result = await invoke<DiskInfo[]>("list_disks");
      setDisks(result);
    } catch (err) {
      console.error("Failed to list disks:", err);
    }
  }, []);

  useEffect(() => {
    refreshDisks();
    const interval = setInterval(refreshDisks, 1000);
    return () => clearInterval(interval);
  }, [refreshDisks]);

  const navigateTo = useCallback((path: string) => {
    setCurrentPath((prev) => {
      if (prev && prev !== path) {
        setNavigationHistory((h) => {
          const newHistory = h.slice(0, historyIndex + 1);
          newHistory.push(prev);
          setHistoryIndex(newHistory.length - 1);
          return newHistory;
        });
      }
      return path;
    });
  }, [historyIndex]);

  const navigateBack = useCallback(() => {
    if (historyIndex >= 0) {
      const path = navigationHistory[historyIndex];
      if (path) {
        setCurrentPath(path);
        setHistoryIndex((i) => i - 1);
      }
    } else if (currentPath) {
      const parts = currentPath.replace(/\/$/, "").split(/[\\/]/);
      if (parts.length > 1) {
        parts.pop();
        const parent = parts.join("\\");
        if (parent) setCurrentPath(parent.endsWith("\\") ? parent : parent + "\\");
      }
    }
  }, [historyIndex, navigationHistory, currentPath]);

  const navigateForward = useCallback(() => {
    if (historyIndex < navigationHistory.length - 1) {
      const newIndex = historyIndex + 1;
      const path = navigationHistory[newIndex];
      if (path) {
        setCurrentPath(path);
        setHistoryIndex(newIndex);
      }
    }
  }, [historyIndex, navigationHistory]);

  const goHome = useCallback(() => {
    setCurrentPath(null);
    setCurrentDisk(null);
  }, []);

  const selectDisk = useCallback((mountPoint: string) => {
    setCurrentDisk(mountPoint);
    navigateTo(mountPoint);
  }, [navigateTo]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.altKey && e.key === "ArrowLeft") {
        e.preventDefault();
        navigateBack();
      } else if (e.altKey && e.key === "ArrowRight") {
        e.preventDefault();
        navigateForward();
      } else if (e.ctrlKey && e.key === "Home") {
        e.preventDefault();
        goHome();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [navigateBack, navigateForward, goHome]);

  return (
    <div className="h-screen flex flex-col bg-[#12101a]">
      {/* Top bar */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-cyber-cyan bg-[#0c0a12]">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold text-cyber-cyan tracking-wider">⚡ DISK DASHBOARD</h1>
          <span className="text-xs text-cyber-magenta">// SYSTEM ANALYSIS ACTIVE</span>
        </div>
        {currentPath && (
          <button
            onClick={goHome}
            className="px-3 py-1 text-sm border border-cyber-cyan text-cyber-cyan bg-[#1e1432] hover:bg-[#2a1e46] rounded transition-colors"
          >
            ⌂ HOME
          </button>
        )}
      </header>

      {/* Main content */}
      <div className="flex flex-1 overflow-hidden">
        <DiskPanel
          disks={disks}
          currentDisk={currentDisk}
          onSelectDisk={selectDisk}
        />
        <main className="flex-1 overflow-hidden">
          {currentPath ? (
            <FileBrowser
              path={currentPath}
              onNavigate={navigateTo}
              onToast={showToast}
            />
          ) : (
            <div className="flex flex-col items-center justify-center h-full">
              <h2 className="text-2xl text-gray-400 mb-4">Select a disk to browse</h2>
              <p className="text-sm text-cyber-dim-purple">Click on a disk in the left panel to explore its contents</p>
            </div>
          )}
        </main>
      </div>

      {/* Toast */}
      {toast && <Toast message={toast.message} />}
    </div>
  );
}

export default App;
