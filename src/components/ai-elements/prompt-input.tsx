"use client";

import * as React from "react";
import {
  AnimatePresence,
  motion,
  MotionConfig,
} from "framer-motion";
import {
  Plus,
  ArrowUp,
  Square,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Textarea } from "@/components/ui/textarea";
import type { ChatMode } from "@/hooks/useStore";

interface PromptInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  onStop?: () => void;
  isLoading?: boolean;
  placeholder?: string;
  className?: string;
  mode?: ChatMode;
  onModeChange?: (mode: ChatMode) => void;
}

const BUTTON_BASE_STYLES =
  "bg-zinc-900/50 hover:bg-zinc-900 border border-zinc-800/50 cursor-pointer rounded-xl h-10 w-10 flex items-center justify-center transition-all focus-visible:outline-[1px] -outline-offset-1 outline-zinc-500";

const SPRING_CONFIG = {
  type: "spring" as const,
  stiffness: 300,
  damping: 30,
};

export function PromptInput({
  value,
  onChange,
  onSubmit,
  onStop,
  isLoading,
  placeholder = "Ask anything...",
  className,
  mode = "agent",
}: PromptInputProps) {
  const textareaRef = React.useRef<HTMLTextAreaElement>(null);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      if (!isLoading && value.trim()) {
        onSubmit();
      }
    }
  };

  React.useEffect(() => {
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = "auto";
      textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`;
    }
  }, [value]);

  return (
    <MotionConfig transition={SPRING_CONFIG}>
      <div className={cn("w-full max-w-3xl mx-auto", className)}>
        <div className="bg-[#0f0f0f]/60 backdrop-blur-xl border border-zinc-900/50 rounded-3xl p-1.5 shadow-2xl overflow-hidden">
          {/* Text Input Area */}
          <div className="relative">
            <Textarea
              ref={textareaRef}
              value={value}
              autoFocus
              placeholder=""
              className="max-h-52 w-full resize-none rounded-none border-none !bg-transparent p-5 !text-[13px] leading-relaxed shadow-none focus-visible:outline-0 focus-visible:ring-0 text-zinc-200 placeholder:text-zinc-800"
              onKeyDown={handleKeyDown}
              onChange={(e) => onChange(e.target.value)}
              disabled={isLoading}
            />
            {!value && (
              <div className="absolute left-5 top-5 pointer-events-none">
                <AnimatePresence mode="wait">
                  <motion.p
                    key={mode}
                    initial={{ opacity: 0, y: 5 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -5 }}
                    className="text-zinc-700 text-[13px]"
                  >
                    {mode === "play" ? "Define validation parameters..." : 
                     mode === "plan" ? "Initialize tactical roadmap..." : 
                     placeholder}
                  </motion.p>
                </AnimatePresence>
              </div>
            )}
          </div>

          {/* Control Buttons Row */}
          <div className="bg-[#0a0a0a] border border-zinc-900/50 flex justify-between rounded-[1.25rem] p-1.5 mt-1">
            <div className="flex items-center">
              <button
                type="button"
                className={cn(BUTTON_BASE_STYLES, "text-zinc-500 hover:text-zinc-200")}
                title="Attach Files"
              >
                <Plus className="size-4" />
              </button>
            </div>

            <div className="flex items-center">
              {isLoading ? (
                <button
                  type="button"
                  onClick={onStop}
                  className={cn(
                    BUTTON_BASE_STYLES,
                    "bg-zinc-900 border-zinc-800 text-zinc-400 hover:bg-zinc-800",
                  )}
                >
                  <Square className="h-3 w-3 fill-current" />
                </button>
              ) : (
                <button
                  type="button"
                  onClick={onSubmit}
                  disabled={!value.trim()}
                  className={cn(
                    BUTTON_BASE_STYLES,
                    "transition-all ease-in-out active:scale-90 shadow-lg",
                    value.trim() ? "bg-zinc-100 border-zinc-200 text-black hover:bg-white" : "opacity-20 cursor-not-allowed",
                  )}
                >
                  <ArrowUp className="h-4 w-4 stroke-[3]" />
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </MotionConfig>
  );
}
