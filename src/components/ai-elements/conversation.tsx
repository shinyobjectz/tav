import * as React from "react";
import { cn } from "@/lib/utils";

interface ConversationProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export function Conversation({ className, children, ...props }: ConversationProps) {
  return (
    <div className={cn("flex flex-col h-full overflow-hidden", className)} {...props}>
      {children}
    </div>
  );
}

export function ConversationContent({ className, children, ...props }: ConversationProps) {
  return (
    <div className={cn("flex-1 overflow-y-auto px-6 py-6 space-y-10", className)} {...props}>
      {children}
    </div>
  );
}

interface ConversationInputProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export function ConversationInput({ className, children, ...props }: ConversationInputProps) {
  return (
    <div className={cn("px-4 pb-6 pt-2 shrink-0", className)} {...props}>
      {children}
    </div>
  );
}
