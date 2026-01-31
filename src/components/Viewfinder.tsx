import { useState, useEffect, useRef, useCallback } from "react";
import { useStore } from "@/hooks/useStore";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Play, RotateCcw, Monitor, Loader2, ExternalLink, X, RefreshCw } from "lucide-react";
import { ProjectWizard } from "./ProjectWizard";
import { TemplateSelector } from "./TemplateSelector";
import { cn } from "@/lib/utils";

export function Viewfinder() {
  const { projectPath, projectName, files, addConsoleOutput, clearConsole, loadFiles, setBuildStatus } = useStore();
  const [isRunning, setIsRunning] = useState(false);
  const [creating, setCreating] = useState(false);
  const [includeCharacter, setIncludeCharacter] = useState(true);
  const [downloadProgress, setDownloadProgress] = useState<number | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);
  const [autoRebuild, setAutoRebuild] = useState(true);
  const [pendingChanges, setPendingChanges] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  // Check if this is a valid Godot project
  const hasGodotProject = files.some((f) => f.name === "project.godot");

  // Auto-focus game when preview starts
  useEffect(() => {
    if (previewUrl && iframeRef.current) {
      // Focus iframe and send message to focus canvas inside
      const focusGame = () => {
        iframeRef.current?.focus();
        iframeRef.current?.contentWindow?.postMessage({ type: 'kobold-focus' }, '*');
      };
      const timer = setTimeout(focusGame, 300);
      return () => clearTimeout(timer);
    }
  }, [previewUrl]);

  // File watcher - listen for changes when preview is running
  useEffect(() => {
    if (!previewUrl || !projectPath) return;
    
    // Start file watcher
    invoke("start_file_watcher", { projectPath }).catch(console.error);
    
    // Listen for file changes
    const unlisten = listen<string[]>("project-files-changed", (event) => {
      const files = event.payload;
      console.log("[Viewfinder] Files changed:", files);
      
      if (autoRebuild && !isExporting) {
        // Auto rebuild
        setBuildStatus("building", "Rebuilding...");
        runPreview(true); // Force rebuild
      } else {
        // Show pending changes indicator
        setPendingChanges(true);
      }
    });
    
    return () => {
      invoke("stop_file_watcher").catch(console.error);
      unlisten.then(fn => fn());
    };
  }, [previewUrl, projectPath, autoRebuild, isExporting]);

  const initializeProject = async (config: { template: string; dimension: string }) => {
    if (!projectPath) return;
    setCreating(true);
    setDownloadProgress(null);
    setBuildStatus("building", `Creating ${config.template}...`);
    
    try {
      // Create base project
      await invoke("initialize_godot_project", {
        projectPath,
        dimension: config.dimension,
        template: config.template,
      });
      
      // Download Quaternius character for 3D projects
      if (config.dimension === "3d" && includeCharacter) {
        setBuildStatus("building", "Downloading character...");
        
        // Listen for download progress
        const unlisten = await listen<{ percent: number }>("download-progress", (event) => {
          setDownloadProgress(event.payload.percent);
        });
        
        try {
          console.log("[Viewfinder] Downloading Quaternius character...");
          const result = await invoke("setup_3d_character", { projectPath });
          console.log("[Viewfinder] Character setup complete:", result);
        } catch (downloadError) {
          console.error("Character download failed:", downloadError);
          // Non-fatal - project still works with placeholder
        }
        
        unlisten();
        setDownloadProgress(null);
      }
      
      await loadFiles(projectPath);
      setBuildStatus("success", "Project created");
    } catch (e) {
      console.error("Failed to initialize project:", e);
      setBuildStatus("error", "Failed to create");
    }
    setCreating(false);
    setDownloadProgress(null);
  };

  const runPreview = useCallback(async (forceRebuild = false) => {
    console.log("[runPreview] Called with projectPath:", projectPath, "forceRebuild:", forceRebuild);
    
    if (!projectPath) {
      console.log("[runPreview] No projectPath, returning early");
      return;
    }

    clearConsole();
    setIsExporting(true);
    setExportError(null);
    setPendingChanges(false);
    
    try {
      // Export the project (templates are checked at app startup)
      console.log("[runPreview] Starting export...");
      setBuildStatus("building", "Exporting...");
      addConsoleOutput("Exporting to HTML5...");
      let exportResult = await invoke<string>("export_project_web", { projectPath, force: forceRebuild });
      console.log("[runPreview] Export result:", exportResult);
      
      // Check if cached
      const isCached = exportResult.startsWith("CACHED:");
      if (isCached) {
        exportResult = exportResult.replace("CACHED:", "");
        addConsoleOutput("Using cached build (no changes detected)");
      } else {
        addConsoleOutput(`Exported to: ${exportResult}`);
      }
      
      const exportPath = exportResult;
      
      // Start the preview server
      console.log("[runPreview] Starting preview server for:", exportPath);
      const port = await invoke<number>("start_preview_server", { exportPath });
      console.log("[runPreview] Server started on port:", port);
      addConsoleOutput(`Preview server started on port ${port}`);
      
      setBuildStatus("success", "Ready");
      
      const url = `http://127.0.0.1:${port}`;
      setPreviewUrl(url);
      useStore.getState().setPreviewUrl(url); // Sync to global store for play mode
      
      // Register capture function for play mode
      useStore.getState().setCaptureFrame(() => {
        return new Promise((resolve) => {
          console.log('[Viewfinder] captureFrame called');
          const iframe = iframeRef.current;
          if (!iframe?.contentWindow) {
            console.log('[Viewfinder] No iframe or contentWindow');
            resolve(null);
            return;
          }
          
          const handler = (event: MessageEvent) => {
            console.log('[Viewfinder] Message received:', event.data?.type);
            if (event.data?.type === 'kobold-capture-result') {
              window.removeEventListener('message', handler);
              console.log('[Viewfinder] Capture success, size:', event.data.data?.length);
              resolve(event.data.data);
            } else if (event.data?.type === 'kobold-capture-error') {
              window.removeEventListener('message', handler);
              console.log('[Viewfinder] Capture error:', event.data.error);
              resolve(null);
            }
          };
          
          window.addEventListener('message', handler);
          console.log('[Viewfinder] Posting kobold-capture message');
          iframe.contentWindow.postMessage({ type: 'kobold-capture' }, '*');
          setTimeout(() => { 
            window.removeEventListener('message', handler); 
            console.log('[Viewfinder] Capture timeout');
            resolve(null); 
          }, 2000);
        });
      });
      
      // Register test controls function for play mode (now uses Godot actions via bridge)
      useStore.getState().setTestControls((actions: string[], duration = 1000) => {
        return new Promise((resolve) => {
          console.log('[Viewfinder] testControls called, actions:', actions, 'duration:', duration);
          const iframe = iframeRef.current;
          if (!iframe?.contentWindow) {
            console.log('[Viewfinder] No iframe or contentWindow');
            resolve(null);
            return;
          }
          
          const handler = (event: MessageEvent) => {
            console.log('[Viewfinder] Message received:', event.data?.type);
            if (event.data?.type === 'kobold-test-result') {
              window.removeEventListener('message', handler);
              console.log('[Viewfinder] Test success, bridgeUsed:', event.data.bridgeUsed);
              console.log('[Viewfinder] State before:', event.data.stateBefore);
              console.log('[Viewfinder] State after:', event.data.stateAfter);
              resolve({ 
                before: event.data.before, 
                after: event.data.after,
                stateBefore: event.data.stateBefore,
                stateAfter: event.data.stateAfter,
                bridgeUsed: event.data.bridgeUsed,
              });
            } else if (event.data?.type === 'kobold-test-error') {
              window.removeEventListener('message', handler);
              console.log('[Viewfinder] Test error:', event.data.error);
              resolve(null);
            }
          };
          
          window.addEventListener('message', handler);
          console.log('[Viewfinder] Posting kobold-test-controls message with actions');
          iframe.contentWindow.postMessage({ type: 'kobold-test-controls', actions, duration }, '*');
          setTimeout(() => { 
            window.removeEventListener('message', handler); 
            console.log('[Viewfinder] Test timeout after', duration + 3000, 'ms');
            resolve(null); 
          }, duration + 3000);
        });
      });
      
      // Register multi-angle node capture function
      useStore.getState().setCaptureNode((nodeId: string, options = {}) => {
        return new Promise((resolve) => {
          console.log('[Viewfinder] captureNode called, node:', nodeId, 'options:', options);
          const iframe = iframeRef.current;
          if (!iframe?.contentWindow) {
            console.log('[Viewfinder] No iframe or contentWindow');
            resolve(null);
            return;
          }
          
          const requestId = Date.now().toString();
          const handler = (event: MessageEvent) => {
            if (event.data?.type === 'kobold-capture-node-result' && event.data.requestId === requestId) {
              window.removeEventListener('message', handler);
              console.log('[Viewfinder] Node capture complete:', Object.keys(event.data.result?.captures || {}));
              resolve(event.data.result);
            } else if (event.data?.type === 'kobold-capture-node-error' && event.data.requestId === requestId) {
              window.removeEventListener('message', handler);
              console.log('[Viewfinder] Node capture error:', event.data.error);
              resolve(null);
            }
          };
          
          window.addEventListener('message', handler);
          iframe.contentWindow.postMessage({ type: 'kobold-capture-node', nodeId, options, requestId }, '*');
          // Timeout after 15s (async captures can take time)
          setTimeout(() => { 
            window.removeEventListener('message', handler); 
            console.log('[Viewfinder] Node capture timeout');
            resolve(null); 
          }, 15000);
        });
      });
      
      setIsRunning(true);
    } catch (e) {
      console.error("[runPreview] Error:", e);
      const error = String(e);
      addConsoleOutput(`Failed: ${error}`);
      setExportError(error);
      setBuildStatus("error", "Build failed");
    }
    setIsExporting(false);
  }, [projectPath, clearConsole, addConsoleOutput, setBuildStatus]);

  const runExternal = async () => {
    if (!projectPath) return;

    clearConsole();
    addConsoleOutput(`$ godot --path "${projectPath}"`);
    setIsRunning(true);

    try {
      const output = await invoke<string>("run_godot", { projectPath });
      output.split("\n").forEach((line) => {
        if (line.trim()) addConsoleOutput(line);
      });
    } catch (e) {
      addConsoleOutput(`error: ${e}`);
    }
    setIsRunning(false);
  };

  const stopPreview = () => {
    setPreviewUrl(null);
    useStore.getState().setPreviewUrl(null);
    useStore.getState().setCaptureFrame(null);
    useStore.getState().setTestControls(null);
    useStore.getState().setCaptureNode(null);
    setIsRunning(false);
    addConsoleOutput("Preview stopped");
  };

  const forceRefresh = async () => {
    if (!projectPath) return;
    stopPreview();
    addConsoleOutput("Clearing cache and rebuilding...");
    try {
      await invoke("clear_export_cache", { projectPath });
      await runPreview(true); // Force re-export
    } catch (e) {
      addConsoleOutput(`Refresh failed: ${e}`);
    }
  };

  // Show wizard when no project is open
  if (!projectPath) {
    return (
      <div className="flex flex-col h-full bg-[#0a0a0a]">
        <ProjectWizard />
      </div>
    );
  }

  // Show quick setup when folder has no Godot project
  if (!hasGodotProject) {
    return (
      <div className="flex flex-col h-full bg-[#0a0a0a]">
        <div className="flex items-center justify-between px-3 py-1.5 bg-[#141414] border-b border-zinc-900/50">
          <div className="flex items-center gap-2 text-[11px] font-medium text-[#666] uppercase tracking-wider">
            <Monitor className="w-3.5 h-3.5" />
            {projectName}
          </div>
        </div>

        <div className="flex-1 flex items-center justify-center p-6">
          <TemplateSelector
            existingPath={projectPath}
            existingName={projectName}
            onComplete={initializeProject}
            loading={creating}
            progress={downloadProgress}
            show3DCharacterOption={true}
            includeCharacter={includeCharacter}
            onCharacterOptionChange={setIncludeCharacter}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-[#0f0f0f]">
      {/* Quick Setup Toolbar */}
      <div className="flex items-center justify-between px-4 py-1.5 bg-zinc-900/10 border-b border-zinc-900/20 backdrop-blur-sm">
        <div className="flex items-center gap-3">
          <div className="w-1.5 h-1.5 rounded-full bg-zinc-900" />
          <span className="text-[9px] font-black text-zinc-800 uppercase tracking-[0.3em]">
            {projectName || "System Ready"}
          </span>
        </div>
        <div className="flex items-center gap-1">
          {isRunning || previewUrl ? (
            <>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 text-zinc-800 hover:text-zinc-400 hover:bg-zinc-900/50 transition-all"
                onClick={forceRefresh}
                title="Rebuild"
              >
                <RefreshCw className="w-3 h-3" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 text-zinc-800 hover:text-red-500 hover:bg-red-500/5 transition-all"
                onClick={stopPreview}
                title="Stop"
              >
                <X className="w-3 h-3" />
              </Button>
            </>
          ) : (
            <>
              <Button
                size="sm"
                className="h-7 px-4 rounded-full bg-zinc-900 border border-zinc-800 hover:bg-zinc-100 hover:text-black text-zinc-500 text-[8px] font-black uppercase tracking-widest transition-all active:scale-95"
                onClick={() => runPreview()}
                disabled={isExporting}
              >
                {isExporting ? (
                  <Loader2 className="w-2.5 h-2.5 mr-2 animate-spin" />
                ) : (
                  <Play className="w-2.5 h-2.5 mr-2" />
                )}
                Initialize
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 text-zinc-800 hover:text-zinc-400 transition-all"
                onClick={runExternal}
                title="External"
              >
                <ExternalLink className="w-3 h-3" />
              </Button>
            </>
          )}
          
          <div className="w-[1px] h-3 bg-zinc-900/50 mx-1" />
          
          <Button
            variant="ghost"
            size="icon"
            className={cn(
              "h-7 w-7 transition-all",
              pendingChanges ? "text-yellow-900/30 hover:text-yellow-600" : "text-zinc-900 hover:text-zinc-600"
            )}
            onClick={() => { 
              clearConsole(); 
              setPendingChanges(false);
              runPreview(true); 
            }}
          >
            <RotateCcw className="w-3 h-3" />
          </Button>
        </div>
      </div>

      {/* Viewport */}
      <div 
        className="flex-1 flex items-center justify-center relative cursor-crosshair overflow-hidden group/viewport"
        onClick={() => {
          iframeRef.current?.focus();
          iframeRef.current?.contentWindow?.postMessage({ type: 'kobold-focus' }, '*');
        }}
      >
        {/* Background Grid Pattern */}
        <div className="absolute inset-0 bg-[radial-gradient(#1a1a1a_1px,transparent_1px)] [background-size:20px_20px] opacity-20 pointer-events-none" />

        {previewUrl ? (
          <div className="relative w-full h-full p-4 md:p-8 flex items-center justify-center">
            <div className="relative w-full h-full max-w-[1024px] max-h-[768px] rounded-2xl overflow-hidden border border-zinc-900 shadow-[0_0_100px_-20px_rgba(0,0,0,0.5)] bg-[#0a0a0a] transition-all group-focus-within/viewport:border-zinc-700">
              <iframe
                ref={iframeRef}
                src={previewUrl}
                className="w-full h-full border-0 focus:outline-none"
                allow="autoplay; fullscreen; pointer-lock; keyboard-map; gamepad; cross-origin-isolated"
                allowFullScreen
                onLoad={() => {
                  setTimeout(() => {
                    iframeRef.current?.focus();
                    iframeRef.current?.contentWindow?.postMessage({ type: 'kobold-focus' }, '*');
                  }, 300);
                }}
                tabIndex={0}
              />
            </div>
          </div>
        ) : isExporting ? (
          <div className="flex flex-col items-center gap-6 animate-in fade-in duration-500">
            <div className="relative">
              <div className="absolute -inset-4 bg-zinc-500/5 rounded-full blur-2xl animate-pulse" />
              <Loader2 className="w-12 h-12 animate-spin text-zinc-800 relative z-10" />
            </div>
            <div className="space-y-1 text-center">
              <p className="text-[10px] uppercase font-black tracking-[0.3em] text-zinc-600">Compiling Environment</p>
              <p className="text-[9px] font-mono text-zinc-800 uppercase">Synchronizing assets...</p>
            </div>
          </div>
        ) : exportError ? (
          <div className="text-center max-w-sm px-8 py-12 rounded-[2rem] border border-zinc-900 bg-zinc-950/40 backdrop-blur-sm animate-in zoom-in-95 duration-300">
            <div className="w-12 h-12 rounded-full bg-red-950/20 border border-red-900/30 flex items-center justify-center mx-auto mb-6">
              <X className="w-6 h-6 text-red-900/50" />
            </div>
            <h3 className="text-sm font-bold text-zinc-200 mb-2 uppercase tracking-widest">Uplink Failed</h3>
            <p className="text-[10px] text-zinc-600 mb-8 whitespace-pre-wrap leading-relaxed font-mono">{exportError}</p>
            <div className="flex gap-3 justify-center">
              <Button size="sm" variant="outline" className="h-10 px-6 rounded-full text-[9px] font-black uppercase tracking-widest border-zinc-800 bg-transparent hover:bg-white hover:text-black transition-all" onClick={() => { setExportError(null); runPreview(); }}>
                <RotateCcw className="w-3.5 h-3.5 mr-2" />
                Re-Attempt
              </Button>
            </div>
          </div>
        ) : (
          <div className="flex flex-col items-center gap-10 animate-in fade-in slide-in-from-bottom-4 duration-1000">
            <button
              onClick={() => runPreview()}
              className="group relative"
            >
              <div className="absolute -inset-8 bg-zinc-500/[0.03] rounded-full blur-3xl group-hover:bg-zinc-500/[0.08] transition-all duration-1000" />
              <div className="relative w-24 h-24 rounded-[2.5rem] bg-[#0a0a0a] border border-zinc-900 flex items-center justify-center shadow-2xl transition-all duration-500 group-hover:border-zinc-700 group-hover:scale-105 active:scale-95">
                <Play className="w-8 h-8 text-zinc-800 fill-zinc-900 group-hover:text-zinc-200 group-hover:fill-zinc-200 transition-all duration-500 ml-1" />
              </div>
            </button>
            <div className="space-y-3 text-center">
              <h3 className="text-lg font-black tracking-tighter text-zinc-200 uppercase tracking-[0.1em]">{projectName}</h3>
              <p className="text-[10px] uppercase font-black tracking-[0.3em] text-zinc-800">Awaiting Signal</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
