import { useRef, useCallback } from "react";
import MonacoEditor from "@monaco-editor/react";
import { useStore } from "@/hooks/useStore";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { X, Save, Monitor, Pin } from "lucide-react";
import { Viewfinder } from "./Viewfinder";

export function TabbedEditor() {
  const { openFiles, activeFile, fileContents, setActiveFile, closeFile, saveFile } = useStore();
  const editorRef = useRef<any>(null);

  // null activeFile means show the Game tab
  const showingGame = activeFile === null;

  const handleEditorMount = (editor: any) => {
    editorRef.current = editor;
  };

  const handleSave = useCallback(() => {
    if (activeFile && editorRef.current) {
      const content = editorRef.current.getValue();
      saveFile(activeFile, content);
    }
  }, [activeFile, saveFile]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      handleSave();
    }
  };

  const getLanguage = (path: string): string => {
    if (path.endsWith(".gd")) return "python";
    if (path.endsWith(".tscn") || path.endsWith(".tres")) return "ini";
    if (path.endsWith(".json")) return "json";
    if (path.endsWith(".md")) return "markdown";
    return "plaintext";
  };

  const getFileName = (path: string): string => path.split(/[\\/]/).pop() || path;

  return (
    <div className="flex flex-col h-full bg-[#0f0f0f]" onKeyDown={handleKeyDown}>
      {/* Tabs */}
      <div className="flex items-center bg-[#0a0a0a] border-b border-zinc-900/30 overflow-hidden shrink-0 h-10">
        <div className="flex-1 flex overflow-x-auto no-scrollbar h-full">
          {/* Pinned Game Tab */}
          <div
            className={cn(
              "group flex items-center gap-2 px-5 h-full text-[9px] font-black uppercase tracking-widest cursor-pointer transition-all border-b-2",
              showingGame
                ? "text-zinc-100 border-zinc-400"
                : "text-zinc-800 hover:text-zinc-500 border-transparent"
            )}
            onClick={() => setActiveFile(null)}
          >
            <Monitor className="w-3.5 h-3.5" />
            <span>Game</span>
          </div>

          {/* File Tabs */}
          {openFiles.map((file) => (
            <div
              key={file}
              className={cn(
                "group flex items-center gap-3 px-5 h-full text-[9px] font-black uppercase tracking-widest cursor-pointer transition-all border-b-2",
                file === activeFile
                  ? "text-zinc-100 border-zinc-400"
                  : "text-zinc-800 hover:text-zinc-500 border-transparent"
              )}
              onClick={() => setActiveFile(file)}
            >
              <span className="truncate max-w-[100px]">{getFileName(file)}</span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  closeFile(file);
                }}
                className="opacity-0 group-hover:opacity-100 hover:text-red-500 transition-all p-0.5"
              >
                <X className="w-3 h-3" />
              </button>
            </div>
          ))}
        </div>

        {/* Sync button */}
        {activeFile && (
          <div className="px-3 border-l border-zinc-900/30 h-full flex items-center">
            <button
              className="text-[8px] font-black uppercase tracking-widest text-zinc-800 hover:text-zinc-200 transition-all flex items-center gap-2"
              onClick={handleSave}
            >
              <Save className="w-3 h-3" />
              <span>Sync</span>
            </button>
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden relative">
        {showingGame ? (
          <Viewfinder />
        ) : activeFile ? (
          <MonacoEditor
            height="100%"
            language={getLanguage(activeFile)}
            value={fileContents[activeFile] || ""}
            theme="vs-dark"
            onMount={handleEditorMount}
            options={{
              fontSize: 12,
              fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              automaticLayout: true,
              tabSize: 4,
              padding: { top: 16, bottom: 16 },
              lineNumbers: "on",
              lineNumbersMinChars: 4,
              glyphMargin: false,
              folding: true,
              renderLineHighlight: "all",
              lineHeight: 20,
              cursorBlinking: "smooth",
              cursorSmoothCaretAnimation: "on",
              smoothScrolling: true,
              scrollbar: {
                verticalScrollbarSize: 4,
                horizontalScrollbarSize: 4,
                vertical: 'visible',
                horizontal: 'visible',
                useShadows: false
              },
            }}
          />
        ) : null}
      </div>
    </div>
  );
}
