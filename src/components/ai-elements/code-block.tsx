import * as React from "react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Check, Copy } from "lucide-react";

interface CodeBlockProps {
  code: string;
  language?: string;
  filename?: string;
  className?: string;
}

export function CodeBlock({ code, language, filename, className }: CodeBlockProps) {
  const [copied, setCopied] = React.useState(false);

  const copyToClipboard = async () => {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={cn("rounded-lg border border-[#222] bg-[#0a0a0a] overflow-hidden", className)}>
      {(filename || language) && (
        <div className="flex items-center justify-between px-3 py-1.5 border-b border-[#222] bg-[#111]">
          <span className="text-[11px] text-[#666]">{filename || language}</span>
          <Button
            size="icon"
            variant="ghost"
            className="h-5 w-5"
            onClick={copyToClipboard}
          >
            {copied ? (
              <Check className="w-3 h-3 text-green-500" />
            ) : (
              <Copy className="w-3 h-3 text-[#666]" />
            )}
          </Button>
        </div>
      )}
      <pre className="p-3 overflow-x-auto text-xs leading-relaxed">
        <code className="font-mono">{code}</code>
      </pre>
    </div>
  );
}
