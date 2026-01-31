import { useStore, FileEntry } from "@/hooks/useStore";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  FileCode,
  FileJson,
  FileText,
  Folder,
  FolderOpen,
  Image,
  Music,
  Gamepad2,
  Film,
  Package,
  ChevronRight,
  Trash2,
} from "lucide-react";
import { useState, useEffect } from "react";

function getFileIcon(name: string, isDir: boolean) {
  if (isDir) return Folder;
  if (name.endsWith(".gd")) return FileCode;
  if (name.endsWith(".tscn")) return Film;
  if (name.endsWith(".tres")) return Package;
  if (name.endsWith(".json")) return FileJson;
  if (name.endsWith(".png") || name.endsWith(".jpg") || name.endsWith(".svg")) return Image;
  if (name.endsWith(".wav") || name.endsWith(".ogg") || name.endsWith(".mp3")) return Music;
  if (name === "project.godot") return Gamepad2;
  return FileText;
}

function FileItem({ entry, depth = 0 }: { entry: FileEntry; depth?: number }) {
  const { openFile, activeFile, selectedFiles, selectFile } = useStore();
  const [expanded, setExpanded] = useState(depth < 1);
  const isActive = activeFile === entry.path;
  const isSelected = selectedFiles.includes(entry.path);
  const Icon = getFileIcon(entry.name, entry.isDir);

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    selectFile(entry.path, e.ctrlKey || e.metaKey, e.shiftKey);
    
    if (entry.isDir) {
      setExpanded(!expanded);
    }
  };

  const handleDoubleClick = () => {
    if (!entry.isDir) {
      openFile(entry.path);
    }
  };

  return (
    <>
      <button
        className={cn(
          "w-full flex items-center gap-2 py-1.5 pr-3 text-[11px] transition-all",
          isSelected
            ? "bg-zinc-900 text-zinc-100 shadow-inner"
            : isActive
            ? "bg-zinc-950 text-zinc-200"
            : "text-zinc-600 hover:text-zinc-400 hover:bg-zinc-950/50"
        )}
        style={{ paddingLeft: 12 + depth * 14 }}
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
      >
        {entry.isDir && (
          <ChevronRight
            className={cn(
              "w-3 h-3 shrink-0 transition-transform text-zinc-800 group-hover:text-zinc-600",
              expanded && "rotate-90"
            )}
          />
        )}
        <Icon
          className={cn(
            "w-3.5 h-3.5 shrink-0",
            entry.isDir ? "text-zinc-700" : "text-zinc-800",
            (isSelected || isActive) && "text-zinc-500"
          )}
        />
        <span className="truncate tracking-tight">{entry.name}</span>
      </button>
      {entry.isDir && expanded && entry.children?.map((child) => (
        <FileItem key={child.path} entry={child} depth={depth + 1} />
      ))}
    </>
  );
}

export function Sidebar() {
  const { files, projectPath, openProject, selectedFiles, clearSelection, deleteSelectedFiles } = useStore();

  const handleDelete = async () => {
    const count = selectedFiles.length;
    if (confirm(`Delete ${count} item(s)?`)) {
      await deleteSelectedFiles();
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Delete" && selectedFiles.length > 0) {
        e.preventDefault();
        handleDelete();
      } else if (e.key === "Escape") {
        clearSelection();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedFiles, deleteSelectedFiles, clearSelection]);

  if (!projectPath) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-6 text-center animate-in fade-in duration-700">
        <div className="w-12 h-12 rounded-2xl bg-zinc-950 border border-zinc-900 flex items-center justify-center mb-4 shadow-2xl">
          <FolderOpen className="w-6 h-6 text-zinc-800" />
        </div>
        <p className="text-[10px] uppercase font-black tracking-[0.2em] text-zinc-800 mb-6">No project link</p>
        <Button size="sm" variant="outline" className="h-9 px-6 rounded-full text-[9px] uppercase font-black tracking-widest border-zinc-900 bg-transparent hover:bg-white hover:text-black transition-all" onClick={openProject}>
          Establish Uplink
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a]" onClick={() => clearSelection()}>
      <ScrollArea className="flex-1">
        <div className="py-2">
          {files.map((entry) => (
            <FileItem key={entry.path} entry={entry} />
          ))}
        </div>
      </ScrollArea>
      {selectedFiles.length > 0 && (
        <div className="p-2 border-t border-zinc-900/30 bg-zinc-950/50">
          <Button
            variant="ghost"
            size="sm"
            className="w-full h-8 flex items-center justify-between px-3 text-[9px] font-black uppercase tracking-widest text-zinc-600 hover:text-red-500 hover:bg-red-500/5 transition-all"
            onClick={(e) => {
              e.stopPropagation();
              handleDelete();
            }}
          >
            <span>Delete {selectedFiles.length} item(s)</span>
            <Trash2 className="w-3 h-3" />
          </Button>
        </div>
      )}
    </div>
  );
}
