import { useState, useEffect, useCallback, useRef } from "react";
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
  const historyIndexRef = useRef(historyIndex);
  historyIndexRef.current = historyIndex;
  const toastTimerRef = useRef<number | undefined>(undefined);

  const showToast = useCallback((message: string, duration = 2000) => {
    clearTimeout(toastTimerRef.current);
    setToast({ message, duration });
    toastTimerRef.current = window.setTimeout(() => setToast(null), duration);
  }, []);

  useEffect(() => {
    const ref = toastTimerRef;
    return () => { if (ref.current !== undefined) clearTimeout(ref.current); };
  }, []);

  const refreshDisks = useCallback(async () => {
    try {
      const result = await invoke<DiskInfo[]>("list_disks");
      setDisks(prev => {
        if (JSON.stringify(prev) === JSON.stringify(result)) return prev;
        return result;
      });
    } catch (err) {
      console.error("Failed to list disks:", err);
    }
  }, []);

  useEffect(() => {
    refreshDisks();
    const interval = setInterval(refreshDisks, 10000);
    return () => clearInterval(interval);
  }, [refreshDisks]);

  const navigateTo = useCallback((path: string) => {
    setCurrentPath((prev) => {
      if (prev && prev !== path) {
        setNavigationHistory((h) => {
          const newHistory = h.slice(0, historyIndexRef.current + 1);
          newHistory.push(prev);
          setHistoryIndex(newHistory.length - 1);
          return newHistory;
        });
      }
      return path;
    });
  }, []);

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

  // Mouse back/forward buttons (Extra1 = button 3, Extra2 = button 4)
  useEffect(() => {
    const handleMouseDown = (e: MouseEvent) => {
      if (e.button === 3) {
        e.preventDefault();
        navigateBack();
      } else if (e.button === 4) {
        e.preventDefault();
        navigateForward();
      }
    };
    window.addEventListener("mousedown", handleMouseDown);
    return () => window.removeEventListener("mousedown", handleMouseDown);
  }, [navigateBack, navigateForward]);

  return (
    <div className="h-screen flex flex-col bg-[#0a0812] bg-grid-pattern">
      {/* Header */}
      <header className="relative flex items-center justify-between px-5 py-3 border-b border-white/[0.06] glass-strong">
        {/* Gradient line at top */}
        <div className="absolute top-0 left-0 right-0 h-[1px] bg-gradient-to-r from-transparent via-cyan-400/40 to-transparent" />

        <div className="flex items-center gap-4">
          <div className="flex items-center gap-3">
            {/* Status indicator */}
            <div className="relative flex items-center justify-center w-8 h-8">
              <div className="absolute inset-0 rounded-lg bg-cyan-500/10 animate-pulse-glow text-cyan-400" />
              <span className="relative text-lg">&#x26A1;</span>
            </div>
            <h1 className="text-xl font-bold tracking-[0.15em] text-gradient-cyan">
              DISK DASHBOARD
            </h1>
          </div>
          <div className="hidden sm:flex items-center gap-2 ml-2">
            <div className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse-glow text-cyan-400" />
            <span className="text-[11px] font-medium tracking-wider text-cyber-magenta/70 uppercase">
              System Analysis Active
            </span>
          </div>
        </div>

        {currentPath && (
          <button
            onClick={goHome}
            className="px-4 py-1.5 text-sm font-medium tracking-wider border border-cyan-400/30 text-cyan-300
                       glass rounded-lg hover:border-cyan-400/60 hover:glow-cyan-soft transition-all duration-200"
          >
            &#x2302; HOME
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
            <div className="flex flex-col items-center justify-center h-full animate-fade-in">
              <div className="glass rounded-2xl p-12 text-center max-w-md">
                <div className="text-5xl mb-6 opacity-40">&#x1F4BE;</div>
                <h2 className="text-2xl font-semibold text-gray-300 mb-3 tracking-wide">Select a Disk</h2>
                <p className="text-sm text-cyber-dim-purple leading-relaxed">
                  Choose a disk from the left panel to explore its contents and analyze storage usage.
                </p>
              </div>
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
