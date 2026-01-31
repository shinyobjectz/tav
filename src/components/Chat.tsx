import { useEffect, useRef, useState } from "react";
import { useStore, ToolCall } from "@/hooks/useStore";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import {
  Conversation,
  ConversationContent,
  ConversationInput,
} from "@/components/ai-elements/conversation";
import {
  Message,
  MessageContent,
  MessageResponse,
  MessageLoader,
} from "@/components/ai-elements/message";
import { PromptInput } from "@/components/ai-elements/prompt-input";
import { CodeBlock } from "@/components/ai-elements/code-block";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Settings, Check, Loader2, AlertCircle, ChevronDown, ChevronRight, FolderOpen, Plus, X, MessageSquare } from "lucide-react";
import { cn } from "@/lib/utils";

interface ChatProps {
  onSetupClick?: () => void;
}

function ThinkingBlock({ tools }: { tools: ToolCall[] }) {
  const [expanded, setExpanded] = useState(false);
  
  const grouped = tools.reduce((acc, tool) => {
    const name = tool.name;
    if (!acc[name]) acc[name] = [];
    acc[name].push(tool);
    return acc;
  }, {} as Record<string, ToolCall[]>);
  
  const completedCount = tools.filter(t => t.status === "completed").length;
  const runningCount = tools.filter(t => t.status === "running").length;
  const errorCount = tools.filter(t => t.status === "error").length;
  const totalCount = tools.length;
  const isRunning = runningCount > 0;
  const hasError = errorCount > 0;

  return (
    <div className="mt-4 mb-6 rounded-xl bg-[#141414] border border-zinc-900/50 overflow-hidden transition-all duration-500 shadow-sm">
      <button
        className="w-full flex items-center gap-4 px-4 py-3 hover:bg-[#1a1a1a] transition-all text-left group"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex-shrink-0">
          {isRunning ? (
            <div className="w-4 h-4 border-2 border-zinc-800 border-t-zinc-400 rounded-full animate-spin" />
          ) : hasError ? (
            <AlertCircle className="w-4 h-4 text-zinc-600" />
          ) : (
            <div className="w-4 h-4 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center">
              <div className="w-1.5 h-1.5 rounded-full bg-zinc-600" />
            </div>
          )}
        </div>
        
        <div className="flex-1 min-w-0 flex items-center gap-3">
          <span className="text-[10px] text-zinc-500 font-black uppercase tracking-[0.25em]">
            {isRunning ? "System Live" : "Action Log"}
          </span>
          <div className="flex-1 h-[1px] bg-zinc-900/50" />
          <span className="text-[10px] text-zinc-700 font-mono tabular-nums">{completedCount} / {totalCount}</span>
        </div>
        
        <div className="flex-shrink-0 text-zinc-800 group-hover:text-zinc-500 transition-colors">
          {expanded ? <ChevronDown className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
        </div>
      </button>
      
      <div className="px-4 pb-3 flex flex-wrap gap-2">
        {Object.entries(grouped).map(([name, items]) => {
          const completed = items.filter(i => i.status === "completed").length;
          const running = items.filter(i => i.status === "running").length;
          
          return (
            <div
              key={name}
              className={cn(
                "flex items-center gap-2 px-2 py-1 rounded-lg text-[9px] font-black uppercase tracking-widest transition-all",
                running > 0 
                  ? "text-zinc-200 bg-zinc-900 border border-zinc-800 shadow-lg" 
                  : "text-zinc-600 bg-transparent border border-transparent opacity-40 hover:opacity-100 hover:border-zinc-900 hover:bg-zinc-950"
              )}
            >
              <span>{name}</span>
              <span className="text-zinc-800 font-normal">|</span>
              <span className="tabular-nums">{completed}</span>
            </div>
          );
        })}
      </div>
      
      {expanded && (
        <div className="border-t border-zinc-900/50 px-4 py-3 space-y-3 max-h-64 overflow-y-auto bg-[#0f0f0f]">
          {Object.entries(grouped).map(([name, items]) => (
            <div key={name} className="space-y-2">
              <div className="text-[9px] font-black text-zinc-800 uppercase tracking-[0.2em] mb-1">
                {name}
              </div>
              {items.map((tool) => (
                <div key={tool.id} className="flex items-start gap-3 pl-1 text-[10px] group/item">
                  <div className={cn(
                    "mt-1.5 w-1 h-1 rounded-full shrink-0 transition-all duration-500",
                    tool.status === "running" ? "bg-zinc-400 scale-125 shadow-[0_0_8px_rgba(255,255,255,0.2)]" : 
                    tool.status === "completed" ? "bg-zinc-800" : "bg-zinc-950"
                  )} />
                  <span className="text-zinc-600 group-hover/item:text-zinc-400 transition-colors truncate font-mono leading-relaxed">{tool.content || "internal_task"}</span>
                </div>
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export function Chat({ onSetupClick }: ChatProps) {
  const {
    threads,
    activeThreadId,
    isLoading,
    currentInput,
    setCurrentInput,
    sendMessage,
    stopGeneration,
    initAgentListener,
    projectPath,
    openProject,
    chatMode,
    setChatMode,
    createThread,
    deleteThread,
    switchThread,
    buildStatus,
    buildMessage,
    isSignedIn,
    checkAuth,
    initAuthListener,
  } = useStore();
  const scrollRef = useRef<HTMLDivElement>(null);
  const [agentName, setAgentName] = useState<string | null>(null);

  // Get current thread and its messages
  const currentThread = threads.find(t => t.id === activeThreadId) || threads[0];
  const messages = currentThread?.messages || [];

  useEffect(() => {
    scrollRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isLoading]);

  useEffect(() => {
    checkAgent();
    checkAuth();
    
    let unlistenAgent: (() => void) | null = null;
    let unlistenAuth: (() => void) | null = null;

    initAgentListener().then((fn) => {
      unlistenAgent = fn;
    });

    initAuthListener().then((fn) => {
      unlistenAuth = fn;
    });
    
    return () => {
      if (unlistenAgent) unlistenAgent();
      if (unlistenAuth) unlistenAuth();
    };
  }, []);

  const checkAgent = async () => {
    try {
      const installed = await invoke<boolean>("detect_goose");
      setAgentName(installed ? "Goose" : null);
    } catch {
      setAgentName(null);
    }
  };

  return (
    <div className="flex flex-col h-full overflow-hidden bg-[#0a0a0a] text-zinc-400">
      <div className="bg-[#0a0a0a] border-b border-zinc-900/30 shrink-0 z-10 h-10 flex items-center">
        <div className="flex items-center gap-0.5 px-3 overflow-x-auto no-scrollbar h-full">
          {threads.map((thread) => (
            <div
              key={thread.id}
              className={cn(
                "group flex items-center gap-2 px-3 h-full text-[9px] font-black uppercase tracking-widest cursor-pointer transition-all border-b-2",
                thread.id === activeThreadId
                  ? "text-zinc-100 border-zinc-400"
                  : "text-zinc-800 hover:text-zinc-500 border-transparent"
              )}
              onClick={() => switchThread(thread.id)}
            >
              <span className="truncate max-w-[80px]">{thread.name}</span>
              {threads.length > 1 && (
                <button
                  className="opacity-0 group-hover:opacity-100 hover:text-red-500 transition-all p-0.5"
                  onClick={(e) => {
                    e.stopPropagation();
                    deleteThread(thread.id);
                  }}
                >
                  <X className="w-3 h-3" />
                </button>
              )}
            </div>
          ))}
          <button
            className="flex items-center justify-center w-6 h-6 text-zinc-900 hover:text-zinc-500 transition-all ml-1"
            onClick={createThread}
            title="New Session"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>
        </div>

        <div className="flex-1 h-full border-l border-zinc-900/30 flex items-center justify-end px-4 gap-4">
          {buildStatus !== "idle" && (
            <div className="flex items-center gap-2">
              <div className={cn(
                "w-1 h-1 rounded-full transition-all duration-500",
                buildStatus === "building" ? "bg-zinc-600 animate-pulse" :
                buildStatus === "success" ? "bg-zinc-800" : "bg-red-950"
              )} />
              <span className="text-[8px] font-black uppercase tracking-widest text-zinc-800">
                {buildMessage.toLowerCase()}
              </span>
            </div>
          )}
          
          <div className="flex items-center gap-2 text-zinc-900">
            <div className="w-1 h-1 rounded-full bg-current" />
            <span className="text-[8px] font-black uppercase tracking-widest">Neural Link</span>
          </div>
        </div>
      </div>

      <Conversation className="flex-1 min-h-0 relative">
        <ScrollArea className="h-full">
          <ConversationContent className="max-w-3xl mx-auto py-12 md:py-20 px-6">
            {messages.length === 0 && (
              <div className="flex flex-col items-center justify-center min-h-[500px] text-center px-4 animate-in fade-in duration-1000 slide-in-from-bottom-8">
                <div className="relative mb-16 group">
                  <div className="absolute -inset-8 bg-zinc-500/5 rounded-full blur-3xl group-hover:bg-zinc-500/10 transition-all duration-1000"></div>
                  <div className="relative w-24 h-24 rounded-[2.5rem] bg-[#0a0a0a] border border-zinc-900 flex items-center justify-center shadow-[0_0_50px_-12px_rgba(0,0,0,0.5)] overflow-hidden">
                    <div className="absolute inset-0 bg-gradient-to-tr from-zinc-950 to-transparent"></div>
                    <img src="/tav-logo.png" alt="Tav" className="w-14 h-14 opacity-90 group-hover:scale-110 transition-transform duration-1000 ease-out" />
                  </div>
                </div>

                {!isSignedIn ? (
                  <Button onClick={onSetupClick} size="sm" variant="outline" className="h-12 px-10 rounded-full text-[10px] uppercase font-black tracking-[0.3em] border-zinc-800 bg-transparent hover:bg-white hover:text-black hover:scale-105 transition-all duration-500">
                    Boot Sequence
                  </Button>
                ) : !projectPath ? (
                  <div className="flex flex-col items-center gap-8">
                    <Button onClick={openProject} size="sm" variant="outline" className="h-12 px-10 rounded-full text-[10px] uppercase font-black tracking-[0.3em] border-zinc-800 bg-transparent hover:bg-white hover:text-black hover:scale-105 transition-all duration-500">
                      Link Workspace
                    </Button>
                    <p className="text-[9px] uppercase font-bold tracking-[0.2em] text-zinc-800 animate-pulse">Awaiting Project Uplink</p>
                  </div>
                ) : (
                  <div className="grid gap-4 w-full max-w-sm">
                    {[
                      "Initialize player controller",
                      "Analyze scene hierarchy",
                      "Optimize movement physics",
                    ].map((suggestion) => (
                      <button
                        key={suggestion}
                        onClick={() => setCurrentInput(suggestion)}
                        className="flex items-center justify-between px-6 py-5 rounded-2xl border border-zinc-900 bg-zinc-950/30 hover:border-zinc-700 hover:bg-zinc-950/80 transition-all duration-500 text-[11px] font-medium text-zinc-600 group text-left shadow-sm"
                      >
                        <span className="group-hover:text-zinc-200 transition-colors duration-500">{suggestion}</span>
                        <span className="opacity-0 group-hover:opacity-100 transition-all transform translate-x-2 group-hover:translate-x-0 text-zinc-500 duration-500">→</span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}

            {messages.map((msg) => (
              <Message key={msg.id} from={msg.role} className="mb-12">
                <MessageContent>
                  {msg.blocks.length === 0 && msg.isStreaming ? (
                    <div className="flex items-center gap-4 px-2">
                      <div className="w-4 h-4 border-2 border-zinc-900 border-t-zinc-600 rounded-full animate-spin" />
                      <span className="text-[10px] uppercase font-black tracking-[0.25em] text-zinc-800">Deciphering Logic</span>
                    </div>
                  ) : (
                    msg.blocks.map((block, idx) => (
                      block.type === "text" ? (
                        <MessageResponse key={idx} className={cn(
                          msg.role === "user" ? "text-zinc-200 font-semibold text-base" : "text-zinc-400 leading-[1.8]"
                        )}>
                          <ReactMarkdown
                            remarkPlugins={[remarkGfm]}
                            components={{
                              code({ node, className, children, ...props }) {
                                const match = /language-(\w+)/.exec(className || "");
                                const code = String(children).replace(/\n$/, "");
                                if (match) {
                                  return (
                                    <div className="relative group/code my-10">
                                      <div className="absolute -inset-4 bg-zinc-500/[0.02] rounded-3xl blur-2xl group-hover/code:bg-zinc-500/[0.05] transition-all duration-1000"></div>
                                      <CodeBlock code={code} language={match[1]} className="relative rounded-2xl border border-zinc-900/50 shadow-2xl overflow-hidden" />
                                    </div>
                                  );
                                }
                                return (
                                  <code className="bg-zinc-950 border border-zinc-900 px-2 py-0.5 rounded-md text-[11px] font-mono text-zinc-300" {...props}>
                                    {children}
                                  </code>
                                );
                              },
                              p: ({children}) => <p className="mb-6 last:mb-0">{children}</p>,
                              ul: ({children}) => <ul className="list-disc pl-6 mb-6 space-y-3 marker:text-zinc-800">{children}</ul>,
                              ol: ({children}) => <ol className="list-decimal pl-6 mb-6 space-y-3 marker:text-zinc-800 font-mono text-xs">{children}</ol>,
                              h1: ({children}) => <h1 className="text-xl font-black tracking-tight text-zinc-100 mb-6 mt-12 uppercase tracking-[0.1em]">{children}</h1>,
                              h2: ({children}) => <h2 className="text-lg font-bold tracking-tight text-zinc-200 mb-4 mt-10">{children}</h2>,
                              h3: ({children}) => <h3 className="text-base font-bold text-zinc-300 mb-3 mt-8">{children}</h3>,
                              blockquote: ({children}) => <blockquote className="border-l-2 border-zinc-800 pl-6 italic text-zinc-500 my-8 leading-relaxed">{children}</blockquote>
                            }}
                          >
                            {block.content}
                          </ReactMarkdown>
                        </MessageResponse>
                      ) : (
                        <ThinkingBlock key={idx} tools={block.tools} />
                      )
                    ))
                  )}
                </MessageContent>
              </Message>
            ))}

            {isLoading && messages[messages.length - 1]?.role !== "assistant" && (
              <Message from="assistant">
                <MessageContent>
                  <MessageLoader />
                </MessageContent>
              </Message>
            )}

            <div ref={scrollRef} className="h-12" />
          </ConversationContent>
        </ScrollArea>

        <div className="absolute bottom-full left-0 right-0 h-24 bg-gradient-to-t from-[#0a0a0a] to-transparent pointer-events-none" />

        <ConversationInput className="bg-[#0a0a0a] border-t border-zinc-900/30 backdrop-blur-md">
          <div className="max-w-3xl mx-auto w-full">
            <PromptInput
              value={currentInput}
              onChange={setCurrentInput}
              onSubmit={sendMessage}
              onStop={stopGeneration}
              isLoading={isLoading}
              placeholder={
                !isSignedIn
                  ? "Initialize core system..."
                  : !projectPath
                  ? "Establish project uplink..."
                  : chatMode === "play"
                  ? "Define validation parameters..."
                  : "Command center..."
              }
              mode={chatMode}
              onModeChange={setChatMode}
            />
          </div>
        </ConversationInput>
      </Conversation>
    </div>
  );
}
