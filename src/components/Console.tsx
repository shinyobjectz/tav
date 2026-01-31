import { useState } from "react";
import { useStore } from "@/hooks/useStore";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { ChevronUp, ChevronDown, Trash2, Terminal } from "lucide-react";

interface ConsoleProps {
  height: number;
  onHeightChange: (height: number) => void;
}

export function Console({ height, onHeightChange }: ConsoleProps) {
  const { consoleOutput, clearConsole } = useStore();
  const [isCollapsed, setIsCollapsed] = useState(false);
  const minHeight = 32;
  const defaultHeight = 150;

  const toggleCollapse = () => {
    if (isCollapsed) {
      onHeightChange(defaultHeight);
      setIsCollapsed(false);
    } else {
      onHeightChange(minHeight);
      setIsCollapsed(true);
    }
  };

  const getLineClass = (line: string): string => {
    if (line.includes("ERROR") || line.startsWith("error:")) return "text-red-900/80";
    if (line.includes("WARNING") || line.includes("WARN")) return "text-zinc-700";
    if (line.startsWith("$")) return "text-zinc-800";
    return "text-zinc-600";
  };

  return (
    <div
      className="bg-[#0a0a0a] border-t border-zinc-900/50 flex flex-col transition-all duration-300 shadow-2xl"
      style={{ height: isCollapsed ? minHeight : height }}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-1.5 bg-zinc-900/10 border-b border-zinc-900/20 backdrop-blur-sm shrink-0">
        <button
          onClick={toggleCollapse}
          className="flex items-center gap-3 text-[9px] font-black text-zinc-800 uppercase tracking-[0.3em] hover:text-zinc-500 transition-all group"
        >
          <div className="flex items-center gap-2">
            <Terminal className="w-3 h-3 text-zinc-900 group-hover:text-zinc-700 transition-colors" />
            <span>Logs</span>
          </div>
          {consoleOutput.length > 0 && (
            <span className="text-[8px] font-mono opacity-40">
              {consoleOutput.length}
            </span>
          )}
        </button>
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 text-zinc-900 hover:text-zinc-600 transition-all"
          onClick={clearConsole}
        >
          <Trash2 className="w-3 h-3" />
        </Button>
      </div>

      {/* Content */}
      {!isCollapsed && (
        <ScrollArea className="flex-1">
          <div className="p-4 font-mono text-[11px] leading-[1.6] tracking-tight">
            {consoleOutput.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-20 opacity-20">
                <Terminal className="w-8 h-8 mb-2" />
                <span className="text-[9px] uppercase font-black tracking-widest">Awaiting Logs</span>
              </div>
            ) : (
              consoleOutput.map((line, i) => (
                <div key={i} className={cn("whitespace-pre-wrap mb-1 transition-colors hover:bg-zinc-900/30 px-1 rounded", getLineClass(line))}>
                  <span className="text-zinc-900 mr-3 select-none tabular-nums">{(i + 1).toString().padStart(3, '0')}</span>
                  {line}
                </div>
              ))
            )}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
