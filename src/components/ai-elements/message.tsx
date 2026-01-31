import * as React from "react";
import { cn } from "@/lib/utils";

type MessageRole = "user" | "assistant";

interface MessageProps extends React.HTMLAttributes<HTMLDivElement> {
  from: MessageRole;
  children: React.ReactNode;
}

export function Message({ from, className, children, ...props }: MessageProps) {
  const isUser = from === "user";

  return (
    <div
      className={cn(
        "w-full flex",
        isUser ? "justify-end" : "justify-start",
        className
      )}
      {...props}
    >
      <div className={cn(
        "max-w-[85%] rounded-2xl px-4 py-2 text-sm transition-all",
        isUser 
          ? "bg-[#111] border border-[#1a1a1a] text-zinc-200 shadow-lg" 
          : "bg-transparent text-zinc-400"
      )}>
        {children}
      </div>
    </div>
  );
}

interface MessageContentProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export function MessageContent({ className, children, ...props }: MessageContentProps) {
  return (
    <div
      className={cn(className)}
      {...props}
    >
      {children}
    </div>
  );
}

interface MessageResponseProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export function MessageResponse({ className, children, ...props }: MessageResponseProps) {
  return (
    <div 
      className={cn(
        "max-w-none text-sm leading-relaxed",
        className
      )} 
      {...props}
    >
      {children}
    </div>
  );
}

export function MessageLoader() {
  return (
    <div className="flex gap-2 py-3 px-1 items-center">
      <div className="w-1 h-1 bg-zinc-700 rounded-full animate-bounce [animation-delay:-0.3s]" />
      <div className="w-1 h-1 bg-zinc-700 rounded-full animate-bounce [animation-delay:-0.15s]" />
      <div className="w-1 h-1 bg-zinc-700 rounded-full animate-bounce" />
      <span className="text-[9px] uppercase font-bold tracking-[0.2em] text-zinc-800 ml-2">Awaiting Response</span>
    </div>
  );
}
