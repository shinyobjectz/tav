import { useState, useEffect, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useStore } from "@/hooks/useStore";
import { Button } from "@/components/ui/button";
import {
  Minus,
  Square,
  X,
  Copy,
  FolderOpen,
  Settings,
} from "lucide-react";

interface AppBarProps {
  onSettingsClick: () => void;
}

export function AppBar({ onSettingsClick }: AppBarProps) {
  const { projectName, openProject } = useStore();
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    const syncMaximized = async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    };

    syncMaximized();

    const unlisten = appWindow.onResized(async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [appWindow]);

  const handleMinimize = useCallback(async () => {
    await appWindow.minimize();
  }, [appWindow]);

  const handleMaximize = useCallback(async () => {
    const maximized = await appWindow.isMaximized();
    if (maximized) {
      await appWindow.unmaximize();
    } else {
      await appWindow.maximize();
    }
  }, [appWindow]);

  const handleClose = useCallback(async () => {
    await appWindow.close();
  }, [appWindow]);

  const handleDoubleClick = useCallback(
    (e: React.MouseEvent) => {
      if ((e.target as HTMLElement).hasAttribute("data-tauri-drag-region")) {
        handleMaximize();
      }
    },
    [handleMaximize]
  );

  return (
    <div className="h-10 bg-[#0a0a0a] border-b border-zinc-900/50 flex items-center select-none backdrop-blur-xl z-20">
      {/* Draggable region */}
      <div
        data-tauri-drag-region
        className="flex-1 h-full flex items-center px-4 gap-4"
        onDoubleClick={handleDoubleClick}
      >
        {/* App icon & name */}
        <div className="flex items-center gap-3 group cursor-pointer" onClick={onSettingsClick}>
          <div className="relative">
            <div className="absolute -inset-2 bg-white/5 rounded-full blur-md opacity-0 group-hover:opacity-100 transition-all duration-700" />
            <img src="/tav-logo.png" alt="Tav" className="relative w-3.5 h-3.5 opacity-40 group-hover:opacity-100 transition-all duration-500" />
          </div>
          <span className="text-[9px] font-black uppercase tracking-[0.3em] text-zinc-800 group-hover:text-zinc-300 transition-all duration-500">Tav</span>
        </div>

        {/* Spacer - draggable */}
        <div data-tauri-drag-region className="flex-1 h-full" />

        {/* Project info - centered-ish */}
        {projectName && (
          <div className="absolute left-1/2 -translate-x-1/2 flex items-center gap-3 px-4 py-1.5 rounded-full bg-zinc-900/10 border border-zinc-900/20 backdrop-blur-md">
            <div className="w-1 h-1 rounded-full bg-zinc-800 animate-pulse" />
            <span className="text-[8px] font-black uppercase tracking-[0.4em] text-zinc-700">
              {projectName}
            </span>
          </div>
        )}

        {/* Action buttons */}
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-zinc-800 hover:text-zinc-200 hover:bg-zinc-900/50 transition-all"
            onClick={openProject}
            title="Establish Uplink"
          >
            <FolderOpen className="w-3.5 h-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-zinc-800 hover:text-zinc-200 hover:bg-zinc-900/50 transition-all"
            onClick={onSettingsClick}
            title="System Config"
          >
            <Settings className="w-3.5 h-3.5" />
          </Button>
        </div>
      </div>
      </div>

      {/* Window controls */}
      <div className="flex items-center h-full border-l border-zinc-900/50 ml-2">
        <button
          onClick={handleMinimize}
          className="h-full w-10 hover:bg-zinc-900 transition-all flex items-center justify-center group"
          title="Minimize"
        >
          <Minus className="w-3 h-3 text-zinc-700 group-hover:text-zinc-300" />
        </button>
        <button
          onClick={handleMaximize}
          className="h-full w-10 hover:bg-zinc-900 transition-all flex items-center justify-center group"
          title={isMaximized ? "Restore" : "Maximize"}
        >
          {isMaximized ? (
            <Copy className="w-2.5 h-2.5 text-zinc-700 group-hover:text-zinc-300 rotate-180" />
          ) : (
            <Square className="w-2.5 h-2.5 text-zinc-700 group-hover:text-zinc-300" />
          )}
        </button>
        <button
          onClick={handleClose}
          className="h-full w-12 hover:bg-red-950/30 transition-all flex items-center justify-center group"
          title="Terminate"
        >
          <X className="w-3.5 h-3.5 text-zinc-700 group-hover:text-red-500" />
        </button>
      </div>
    </div>
  );
}
