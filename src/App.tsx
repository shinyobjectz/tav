import { useState, useEffect, useRef, useCallback } from "react";
import { useStore } from "@/hooks/useStore";
import { AppBar } from "@/components/AppBar";
import { Sidebar } from "@/components/Sidebar";
import { TabbedEditor } from "@/components/TabbedEditor";
import { Console } from "@/components/Console";
import { Chat } from "@/components/Chat";
import { Settings } from "@/components/Settings";
import { SetupWizard } from "@/components/SetupWizard";
import { invoke } from "@tauri-apps/api/core";

export default function App() {
  const { loadLastProject, loadThreads, initPlaytestListener } = useStore();
  const [showSettings, setShowSettings] = useState(false);
  const [setupComplete, setSetupComplete] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState(200);
  const [chatWidth, setChatWidth] = useState(320);
  const [consoleHeight, setConsoleHeight] = useState(150);

  const dragging = useRef<"sidebar" | "chat" | null>(null);

  useEffect(() => {
    checkSetup();
  }, []);

  const checkSetup = async () => {
    try {
      const status = await invoke<{ godotInstalled: boolean; templatesInstalled: boolean }>("check_setup_status");
      if (status.godotInstalled && status.templatesInstalled) {
        setSetupComplete(true);
        checkAgentSetup();
        loadLastProject();
        loadThreads();
        initPlaytestListener();
      }
    } catch {
      // Setup wizard will show
    }
  };

  const handleSetupComplete = () => {
    setSetupComplete(true);
    checkAgentSetup();
    loadLastProject();
    loadThreads();
    initPlaytestListener();
  };

  const checkAgentSetup = async () => {
    try {
      const settings = await invoke<{ openrouterKey: string | null }>("get_settings");
      if (!settings.openrouterKey) {
        setShowSettings(true);
      }
    } catch {
      setShowSettings(true);
    }
  };

  const handleSettingsClose = () => {
    setShowSettings(false);
  };

  const handleMouseDown = useCallback((panel: "sidebar" | "chat") => (e: React.MouseEvent) => {
    e.preventDefault();
    dragging.current = panel;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }, []);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!dragging.current) return;

      if (dragging.current === "sidebar") {
        const newWidth = Math.max(150, Math.min(400, e.clientX));
        setSidebarWidth(newWidth);
      } else if (dragging.current === "chat") {
        const newWidth = Math.max(250, Math.min(500, window.innerWidth - e.clientX));
        setChatWidth(newWidth);
      }
    };

    const handleMouseUp = () => {
      dragging.current = null;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, []);

  return (
    <div className="h-screen flex flex-col bg-[#0a0a0a]">
      {/* Custom App Bar */}
      <AppBar onSettingsClick={() => setShowSettings(true)} />

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar - File Explorer */}
        <div
          className="bg-[#0a0a0a] border-r border-zinc-900/50 overflow-hidden relative flex-shrink-0"
          style={{ width: sidebarWidth }}
        >
          <Sidebar />
          {/* Resize Handle */}
          <div
            className="absolute top-0 right-0 w-[1px] h-full cursor-col-resize hover:bg-zinc-500/20 transition-all z-10"
            onMouseDown={handleMouseDown("sidebar")}
          />
        </div>

        {/* Center - Tabbed Editor + Console */}
        <div className="flex-1 flex flex-col overflow-hidden min-w-0 bg-[#0f0f0f]">
          {/* Tabbed Editor (includes Game tab + file tabs) */}
          <div className="flex-1 overflow-hidden">
            <TabbedEditor />
          </div>

          {/* Console Drawer */}
          <Console height={consoleHeight} onHeightChange={setConsoleHeight} />
        </div>

        {/* Right - Chat */}
        <div
          className="bg-[#0a0a0a] border-l border-zinc-900/50 overflow-hidden relative flex-shrink-0"
          style={{ width: chatWidth }}
        >
          {/* Resize Handle */}
          <div
            className="absolute top-0 left-0 w-[1px] h-full cursor-col-resize hover:bg-zinc-500/20 transition-all z-10"
            onMouseDown={handleMouseDown("chat")}
          />
          <Chat onSetupClick={() => setShowSettings(true)} />
        </div>
      </div>

      {/* Settings Modal */}
      {showSettings && <Settings onClose={handleSettingsClose} />}

      {/* Setup Wizard - blocks app until complete */}
      {!setupComplete && <SetupWizard onComplete={handleSetupComplete} />}
    </div>
  );
}
