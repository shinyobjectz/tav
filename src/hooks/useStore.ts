import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { exists } from "@tauri-apps/plugin-fs";

export interface FileEntry {
  name: string;
  path: string;
  isDir: boolean;
  children?: FileEntry[];
}

export interface ToolCall {
  id: string;
  name: string;
  status: "running" | "completed" | "error";
  content: string;
  timestamp: number;
}

export interface PlaytestEvent {
  event_type: string;
  message: string;
  frame?: number;
  action?: string;
  screenshot?: string;
}

export type MessageBlock = 
  | { type: "text"; content: string }
  | { type: "tools"; tools: ToolCall[] };

export interface Message {
  id: string;
  role: "user" | "assistant";
  blocks: MessageBlock[];
  timestamp: number;
  isStreaming?: boolean;
}

export interface Thread {
  id: string;
  name: string;
  messages: Message[];
  createdAt: number;
}

export type ChatMode = "agent" | "plan" | "play";

export interface NodeCaptureOptions {
  angles?: string[]; // "front", "back", "left", "right", "front_right", etc.
  top?: boolean;
  distance?: number;
  height?: number;
  custom?: { yaw: number; pitch: number; distance?: number };
}

export interface NodeCaptureResult {
  node: string;
  bounds: { center: { x: number; y: number; z: number }; size: { x: number; y: number; z: number }; found_mesh: boolean };
  captures: Record<string, string>; // angle -> base64 image
}

interface AgentEvent {
  eventType: string;
  content: string;
  toolName: string | null;
  toolArgs: string | null;
}

interface Store {
  // Project
  projectPath: string | null;
  projectName: string | null;
  files: FileEntry[];

  // Editor
  openFiles: string[];
  activeFile: string | null;
  fileContents: Record<string, string>;
  selectedFiles: string[];
  lastSelectedFile: string | null;

  // Chat
  threads: Thread[];
  activeThreadId: string | null;
  isLoading: boolean;
  currentInput: string;
  chatMode: ChatMode;

  // Console
  consoleOutput: string[];

  // Preview
  previewUrl: string | null;
  captureFrame: (() => Promise<string | null>) | null;
  testControls: ((actions: string[], duration?: number) => Promise<{ 
    before: string; 
    after: string; 
    stateBefore?: Record<string, unknown>;
    stateAfter?: Record<string, unknown>;
    bridgeUsed?: boolean;
  } | null>) | null;
  captureNode: ((nodeId: string, options?: NodeCaptureOptions) => Promise<NodeCaptureResult | null>) | null;
  inputMappings: { action: string; keys: string[]; description: string }[];

  // Build status (replaces toasts)
  buildStatus: "idle" | "building" | "success" | "error";
  buildMessage: string;

  // Playtest
  playtestRunning: boolean;
  playtestEvents: PlaytestEvent[];

  // Auth
  isSignedIn: boolean;
  isSigningIn: boolean;

  // Actions
  openProject: () => Promise<void>;
  loadLastProject: () => Promise<void>;
  loadFiles: (path: string) => Promise<void>;
  openFile: (path: string) => Promise<void>;
  closeFile: (path: string) => void;
  setActiveFile: (path: string | null) => void;
  saveFile: (path: string, content: string) => Promise<void>;
  selectFile: (path: string, multi: boolean, range: boolean) => void;
  clearSelection: () => void;
  deleteSelectedFiles: () => Promise<void>;
  setCurrentInput: (input: string) => void;
  setChatMode: (mode: ChatMode) => void;
  sendMessage: () => Promise<void>;
  stopGeneration: () => void;
  createThread: () => void;
  deleteThread: (id: string) => void;
  switchThread: (id: string) => void;
  renameThread: (id: string, name: string) => void;
  loadThreads: () => Promise<void>;
  addConsoleOutput: (line: string) => void;
  clearConsole: () => void;
  initAgentListener: () => Promise<() => void>;
  setPreviewUrl: (url: string | null) => void;
  setCaptureFrame: (fn: (() => Promise<string | null>) | null) => void;
  setTestControls: (fn: ((actions: string[], duration?: number) => Promise<{ before: string; after: string; stateBefore?: Record<string, unknown>; stateAfter?: Record<string, unknown>; bridgeUsed?: boolean } | null>) | null) => void;
  setCaptureNode: (fn: ((nodeId: string, options?: NodeCaptureOptions) => Promise<NodeCaptureResult | null>) | null) => void;
  loadInputMappings: () => Promise<void>;
  setBuildStatus: (status: "idle" | "building" | "success" | "error", message?: string) => void;
  startPlaytest: (objective: string) => Promise<void>;
  initPlaytestListener: () => Promise<() => void>;
  startAuth: () => Promise<void>;
  initAuthListener: () => Promise<() => void>;
  checkAuth: () => Promise<void>;
}

const initialThreadId = crypto.randomUUID();

// Helper to persist threads to disk
const saveThreadsToDisk = async (threads: Thread[]) => {
  try {
    await invoke("save_threads", { threads });
  } catch (e) {
    console.error("[Store] Failed to save threads:", e);
  }
};

export const useStore = create<Store>((set, get) => ({
  projectPath: null,
  projectName: null,
  files: [],
  openFiles: [],
  activeFile: null,
  fileContents: {},
  selectedFiles: [],
  lastSelectedFile: null,
  threads: [{ id: initialThreadId, name: "Thread 1", messages: [], createdAt: Date.now() }],
  activeThreadId: initialThreadId,
  isLoading: false,
  currentInput: "",
  chatMode: "agent",
  consoleOutput: [],
  previewUrl: null,
  captureFrame: null,
  testControls: null,
  captureNode: null,
  inputMappings: [],
  buildStatus: "idle",
  buildMessage: "",
  playtestRunning: false,
  playtestEvents: [],
  isSignedIn: false,
  isSigningIn: false,

  openProject: async () => {
    try {
      const selected = await open({
        directory: true,
        title: "Select Godot Project Folder",
      });

      if (selected && typeof selected === "string") {
        const name = selected.split(/[\\/]/).pop() || "Project";
        set({ projectPath: selected, projectName: name });
        await get().loadFiles(selected);
        await get().loadInputMappings(); // Load Godot input mappings
        
        // Save as last project
        try {
          const settings = await invoke<Record<string, unknown>>("get_settings");
          await invoke("save_settings", {
            settings: { ...settings, lastProjectPath: selected },
          });
        } catch {}
      }
    } catch (e) {
      console.error("Failed to open project:", e);
    }
  },

  loadLastProject: async () => {
    try {
      const settings = await invoke<{ lastProjectPath?: string }>("get_settings");
      if (settings.lastProjectPath) {
        // Check if the folder still exists
        const folderExists = await exists(settings.lastProjectPath);
        if (!folderExists) {
          console.log("Last project folder no longer exists, clearing setting");
          // Clear the invalid path from settings
          await invoke("save_settings", {
            settings: { ...settings, lastProjectPath: null },
          });
          return;
        }
        
        const name = settings.lastProjectPath.split(/[\\/]/).pop() || "Project";
        set({ projectPath: settings.lastProjectPath, projectName: name });
        await get().loadFiles(settings.lastProjectPath);
      }
    } catch (e) {
      console.error("Failed to load last project:", e);
    }
  },

  loadFiles: async (path: string) => {
    try {
      const files = await invoke<FileEntry[]>("list_files", { path });
      set({ files });
    } catch (e) {
      console.error("Failed to load files:", e);
    }
  },

  openFile: async (path: string) => {
    const { openFiles, fileContents } = get();

    if (!openFiles.includes(path)) {
      try {
        const content = await invoke<string>("read_file", { path });
        set({
          openFiles: [...openFiles, path],
          fileContents: { ...fileContents, [path]: content },
          activeFile: path,
        });
      } catch (e) {
        console.error("Failed to open file:", e);
      }
    } else {
      set({ activeFile: path });
    }
  },

  closeFile: (path: string) => {
    const { openFiles, activeFile, fileContents } = get();
    const newOpenFiles = openFiles.filter((f) => f !== path);
    const newContents = { ...fileContents };
    delete newContents[path];

    set({
      openFiles: newOpenFiles,
      fileContents: newContents,
      activeFile: activeFile === path ? newOpenFiles[0] || null : activeFile,
    });
  },

  setActiveFile: (path: string | null) => set({ activeFile: path }),

  selectFile: (path: string, multi: boolean, range: boolean) => {
    const { selectedFiles, lastSelectedFile, files } = get();
    
    // Helper to flatten file tree
    const flattenFiles = (entries: FileEntry[]): string[] => {
      const result: string[] = [];
      for (const entry of entries) {
        result.push(entry.path);
        if (entry.children) {
          result.push(...flattenFiles(entry.children));
        }
      }
      return result;
    };

    if (range && lastSelectedFile) {
      // Shift+click: select range
      const allPaths = flattenFiles(files);
      const lastIdx = allPaths.indexOf(lastSelectedFile);
      const currIdx = allPaths.indexOf(path);
      if (lastIdx !== -1 && currIdx !== -1) {
        const start = Math.min(lastIdx, currIdx);
        const end = Math.max(lastIdx, currIdx);
        const rangePaths = allPaths.slice(start, end + 1);
        set({ selectedFiles: [...new Set([...selectedFiles, ...rangePaths])] });
      }
    } else if (multi) {
      // Ctrl+click: toggle selection
      if (selectedFiles.includes(path)) {
        set({ selectedFiles: selectedFiles.filter((p) => p !== path), lastSelectedFile: path });
      } else {
        set({ selectedFiles: [...selectedFiles, path], lastSelectedFile: path });
      }
    } else {
      // Normal click: single select
      set({ selectedFiles: [path], lastSelectedFile: path });
    }
  },

  clearSelection: () => set({ selectedFiles: [], lastSelectedFile: null }),

  deleteSelectedFiles: async () => {
    const { selectedFiles, projectPath, openFiles, closeFile, loadFiles } = get();
    if (selectedFiles.length === 0) return;

    // Filter out paths that are children of other selected paths
    // (if a folder is selected, we don't need to delete its children separately)
    const pathsToDelete = selectedFiles.filter((path) => {
      return !selectedFiles.some((other) => {
        if (other === path) return false;
        // Check if path is inside other (other is a parent folder)
        const normalizedPath = path.replace(/\\/g, "/");
        const normalizedOther = other.replace(/\\/g, "/");
        return normalizedPath.startsWith(normalizedOther + "/");
      });
    });

    // Close any open files that will be deleted
    for (const path of selectedFiles) {
      if (openFiles.includes(path)) {
        closeFile(path);
      }
    }

    // Delete files
    let hasError = false;
    for (const path of pathsToDelete) {
      try {
        await invoke("delete_file", { path });
      } catch (e) {
        console.error(`Failed to delete ${path}:`, e);
        hasError = true;
      }
    }

    set({ selectedFiles: [], lastSelectedFile: null });
    if (projectPath) {
      await loadFiles(projectPath);
    }
    
    if (hasError) {
      console.warn("Some files could not be deleted");
    }
  },

  saveFile: async (path: string, content: string) => {
    try {
      await invoke("write_file", { path, content });
      set((s) => ({
        fileContents: { ...s.fileContents, [path]: content },
      }));
    } catch (e) {
      console.error("Failed to save file:", e);
    }
  },

  setCurrentInput: (input: string) => set({ currentInput: input }),

  setChatMode: (mode: ChatMode) => set({ chatMode: mode }),

  sendMessage: async () => {
    const { currentInput, threads, activeThreadId, projectPath, chatMode } = get();
    if (!currentInput.trim()) return;

    // Ensure we have an active thread
    let threadId = activeThreadId;
    if (!threadId && threads.length > 0) {
      threadId = threads[0].id;
      set({ activeThreadId: threadId });
    }
    if (!threadId) return;

    const thread = threads.find(t => t.id === threadId);
    if (!thread) return;

    // Build message based on mode
    let messageToSend = currentInput.trim();
    if (chatMode === "plan") {
      messageToSend = `[PLAN MODE] Create a detailed plan before making any changes. Explain what you will do step by step, then ask for confirmation before executing.\n\n${messageToSend}`;
    } else if (chatMode === "play") {
      // Play mode: capture frame and analyze with Gemini
      const { captureFrame } = get();
      
      if (!captureFrame) {
        messageToSend = `[PLAY MODE ERROR] Game preview is not running. Please click Preview in the Game tab first, then try again.`;
      } else {
        // Capture will happen after we create the message
        messageToSend = currentInput.trim();
      }
    }

    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: "user",
      blocks: [{ type: "text", content: currentInput.trim() }],
      timestamp: Date.now(),
    };

    const assistantMessage: Message = {
      id: crypto.randomUUID(),
      role: "assistant",
      blocks: [],
      timestamp: Date.now(),
      isStreaming: true,
    };

    set((s) => ({
      threads: s.threads.map(t => 
        t.id === threadId 
          ? { ...t, messages: [...t.messages, userMessage, assistantMessage] }
          : t
      ),
      currentInput: "",
      isLoading: true,
    }));

    // Helper to update streaming message with progress
    const updateProgress = (text: string) => {
      set((s) => ({
        threads: s.threads.map(t => 
          t.id === threadId
            ? { ...t, messages: t.messages.map(m =>
                m.id === assistantMessage.id
                  ? { ...m, blocks: [{ type: "text" as const, content: text }] }
                  : m
              )}
            : t
        ),
      }));
    };

    try {
      let response: string;
      const input = currentInput.trim().toLowerCase();
      const { captureFrame, testControls } = get();
      
      const { inputMappings } = get();
      
      if (chatMode === "play" && (captureFrame || testControls)) {
        // Detect if user wants to test controls
        const wantsControlTest = /\b(test|try|check|use|press|move|walk|run|jump|controls?|controller|movement|input)\b/i.test(input);
        
        if (wantsControlTest && testControls) {
          // Use Godot input mappings to determine actions to test
          let actionsToTest: string[] = [];
          let duration = 1500;
          
          // Check for broad requests first
          const wantsAll = /all|every|full/i.test(input);
          const wantsMovement = /movement|wasd|walk|move around|locomotion/i.test(input);
          const wantsCamera = /camera|look|view|orbit|rotate/i.test(input);
          const wantsControls = /control|controller/i.test(input);
          
          if (wantsAll || (wantsControls && !input.match(/\b(jump|attack|interact)\b/i))) {
            // Test all available actions
            actionsToTest = inputMappings.map(m => m.action);
          } else if (wantsMovement || wantsCamera) {
            // Test all movement-related actions
            actionsToTest = inputMappings
              .filter(m => m.action.includes('move') || m.action.includes('walk') || m.action.includes('look'))
              .map(m => m.action);
          } else {
            // Check each input mapping against user's specific request
            for (const mapping of inputMappings) {
              const actionWords = mapping.action.toLowerCase().split('_');
              const descLower = mapping.description.toLowerCase();
              
              // Check if user mentioned this action or its description
              if (actionWords.some(word => word.length > 2 && input.includes(word)) || 
                  input.includes(mapping.action.toLowerCase()) ||
                  descLower.split(' ').some(word => word.length > 3 && input.includes(word)) ||
                  (mapping.action.includes('jump') && /jump/i.test(input)) ||
                  (mapping.action.includes('attack') && /attack|click/i.test(input)) ||
                  (mapping.action.includes('interact') && /interact|use/i.test(input))) {
                actionsToTest.push(mapping.action);
              }
            }
          }
          
          // Default: test all movement if nothing specific matched
          if (actionsToTest.length === 0) {
            const movementActions = inputMappings.filter(m => 
              m.action.includes('move') || m.action.includes('walk')
            );
            if (movementActions.length > 0) {
              actionsToTest = movementActions.map(m => m.action);
            } else {
              // Ultimate fallback
              actionsToTest = ['move_up', 'move_left', 'move_right', 'move_down'];
            }
          }
          
          // Remove duplicates
          actionsToTest = [...new Set(actionsToTest)];
          
          console.log('[PlayMode] Testing actions:', actionsToTest, 'for', duration, 'ms');
          updateProgress(`🎮 Testing: ${actionsToTest.join(', ')}\n⏱️ Duration: ${duration/1000}s\n\n⏳ Capturing & sending input via Tav Bridge...`);
          
          // Test controls via Tav Bridge (native Godot API)
          const result = await testControls(actionsToTest, duration);
          console.log('[PlayMode] Test result:', result ? 'success' : 'failed', 'bridgeUsed:', result?.bridgeUsed);
          
          if (!result) {
            throw new Error("Failed to test controls. Make sure the game is running and click on the preview first to give it focus.");
          }
          
          const bridgeStatus = result.bridgeUsed ? '🔗 Native Bridge' : '⌨️ Key Simulation';
          console.log('[PlayMode] State before:', result.stateBefore);
          console.log('[PlayMode] State after:', result.stateAfter);
          updateProgress(`🎮 Testing: ${actionsToTest.join(', ')}\n${bridgeStatus}\n\n✅ Captured before frame\n✅ Executed actions\n✅ Captured after frame\n\n🤖 Analyzing with Gemini Robotics...`);
          
          const beforeB64 = result.before.replace(/^data:image\/\w+;base64,/, '');
          const afterB64 = result.after.replace(/^data:image\/\w+;base64,/, '');
          
          // Build context about controls and state for the AI
          const controlsContext = inputMappings.length > 0
            ? `Available game controls:\n${inputMappings.map(m => `- ${m.action}: ${m.keys.join('/')} (${m.description})`).join('\n')}`
            : '';
          
          const stateContext = result.stateBefore && result.stateAfter
            ? `\n\nGame state BEFORE:\n${JSON.stringify(result.stateBefore, null, 2)}\n\nGame state AFTER:\n${JSON.stringify(result.stateAfter, null, 2)}`
            : '';
          
          response = await invoke<string>("test_game_controls", {
            beforeB64,
            afterB64,
            keys: actionsToTest, // Backend still expects 'keys' param name
            durationMs: duration,
            prompt: `${currentInput.trim()}\n\n${controlsContext}\n\nActions tested: ${actionsToTest.join(', ')}${stateContext}`,
          });
        } else if (captureFrame) {
          console.log('[PlayMode] Capture-only mode');
          updateProgress(`📸 Capturing game frame...\n\n🤖 Analyzing with Gemini Robotics...`);
          
          // Just capture and analyze (no control test)
          const frameData = await captureFrame();
          console.log('[PlayMode] Capture result:', frameData ? `${frameData.length} bytes` : 'failed');
          
          if (!frameData) {
            throw new Error("Failed to capture game frame. Make sure the game is running.");
          }
          
          // Include available controls in context
          const controlsContext = inputMappings.length > 0
            ? `\n\nAvailable game controls:\n${inputMappings.map(m => `- ${m.action}: ${m.keys.join('/')} (${m.description})`).join('\n')}`
            : '';
          
          const base64 = frameData.replace(/^data:image\/\w+;base64,/, '');
          console.log('[PlayMode] Sending to Gemini, base64 length:', base64.length);
          response = await invoke<string>("analyze_game_frame", {
            screenshotB64: base64,
            prompt: currentInput.trim() + controlsContext,
          });
          console.log('[PlayMode] Got response, length:', response.length);
        } else {
          throw new Error("Game preview not running. Start the preview first.");
        }
      } else if (/\b(playtest|play\s*test|test.*game|ai.*play|robot.*play|auto.*play)\b/i.test(input) && projectPath) {
        // Trigger AI playtest
        const objectiveMatch = input.match(/(?:objective|goal|try to|should)\s*[:=]?\s*["']?([^"'\n]+)["']?/i);
        const objective = objectiveMatch?.[1] || "Explore the game and test that controls work correctly";
        
        // Check if NitroGen is available, otherwise use Gemini
        const useNitrogen = /nitrogen|nitro|local/i.test(input);
        
        if (useNitrogen) {
          updateProgress(`🤖 Starting NitroGen Playtest\n📍 Objective: ${objective}\n\n⏳ Connecting to local AI...`);
          response = await invoke<string>("run_playtest_nitrogen", {
            projectPath,
            config: { objective, max_duration_secs: 60 },
          });
        } else {
          updateProgress(`🤖 Starting AI Playtest (Gemini)\n📍 Objective: ${objective}\n\n⏳ Analyzing game frames...`);
          response = await invoke<string>("run_playtest", {
            projectPath,
            config: { objective, max_duration_secs: 60 },
          });
        }
      } else {
        // Normal agent mode
        // Check if this thread has previous assistant messages (to continue session)
        const thread = threads.find(t => t.id === threadId);
        const hasHistory = thread?.messages.some((m: Message) => m.role === "assistant" && m.blocks.length > 0) ?? false;
        
        response = await invoke<string>("send_agent_message", {
          message: messageToSend,
          projectPath,
          continueSession: hasHistory,
        });
        
        // Refresh files in case agent modified them
        if (projectPath) {
          await get().loadFiles(projectPath);
        }
      }

      // Update the assistant message with final response
      const newThreads = get().threads.map(t => 
          t.id === threadId
            ? { ...t, messages: t.messages.map(m =>
                m.id === assistantMessage.id
                  ? { ...m, blocks: [{ type: "text" as const, content: response }], isStreaming: false }
                  : m
              )}
            : t
        );
      set({ threads: newThreads, isLoading: false });
      saveThreadsToDisk(newThreads);
    } catch (e) {
      console.error("Agent communication failed:", e);
      const newThreads = get().threads.map(t => 
          t.id === threadId
            ? { ...t, messages: t.messages.map(m =>
                m.id === assistantMessage.id
                  ? { ...m, blocks: [{ type: "text" as const, content: `Error: ${e}` }], isStreaming: false }
                  : m
              )}
            : t
        );
      set({ threads: newThreads, isLoading: false });
      saveThreadsToDisk(newThreads);
    }
  },

  stopGeneration: () => {
    set({ isLoading: false });
  },

  createThread: () => {
    const { threads } = get();
    const newThread: Thread = {
      id: crypto.randomUUID(),
      name: `Thread ${threads.length + 1}`,
      messages: [],
      createdAt: Date.now(),
    };
    const newThreads = [...threads, newThread];
    set({ threads: newThreads, activeThreadId: newThread.id });
    saveThreadsToDisk(newThreads);
  },

  deleteThread: (id: string) => {
    const { threads, activeThreadId } = get();
    if (threads.length <= 1) return; // Keep at least one thread
    
    const newThreads = threads.filter(t => t.id !== id);
    const newActiveId = activeThreadId === id 
      ? newThreads[0]?.id || null 
      : activeThreadId;
    
    set({ threads: newThreads, activeThreadId: newActiveId });
    saveThreadsToDisk(newThreads);
  },

  switchThread: (id: string) => {
    set({ activeThreadId: id });
  },

  renameThread: (id: string, name: string) => {
    const newThreads = get().threads.map(t => 
      t.id === id ? { ...t, name } : t
    );
    set({ threads: newThreads });
    saveThreadsToDisk(newThreads);
  },

  loadThreads: async () => {
    try {
      const savedThreads = await invoke<Thread[]>("load_threads");
      if (savedThreads && savedThreads.length > 0) {
        set({ 
          threads: savedThreads,
          activeThreadId: savedThreads[0].id,
        });
      }
    } catch (e) {
      console.error("[Store] Failed to load threads:", e);
    }
  },

  addConsoleOutput: (line: string) => {
    set((s) => ({ consoleOutput: [...s.consoleOutput, line] }));
  },

  clearConsole: () => set({ consoleOutput: [] }),

  initAgentListener: async () => {
    const unlisten = await listen<AgentEvent>("agent-event", (event) => {
      const { eventType, content, toolName } = event.payload;
      console.log('[AgentEvent]', eventType, toolName || '', content?.substring(0, 100));

      set((s) => {
        const { threads, activeThreadId } = s;
        if (!activeThreadId) return s;

        const threadIndex = threads.findIndex(t => t.id === activeThreadId);
        if (threadIndex === -1) return s;

        const thread = threads[threadIndex];
        const messages = [...thread.messages];
        const lastMsg = messages[messages.length - 1];

        if (!lastMsg || lastMsg.role !== "assistant" || !lastMsg.isStreaming) {
          return s;
        }

        const blocks = [...lastMsg.blocks];
        const lastBlock = blocks[blocks.length - 1];
        
        if (eventType === "start") {
          // Reset blocks
          lastMsg.blocks = [];
        } else if (eventType === "tool_start") {
          // Add tool - group with existing tools block or create new one
          const newTool: ToolCall = {
            id: crypto.randomUUID(),
            name: toolName || "tool",
            status: "running",
            content: content,
            timestamp: Date.now(),
          };
          if (lastBlock?.type === "tools") {
            lastBlock.tools = [...lastBlock.tools, newTool];
          } else {
            blocks.push({ type: "tools", tools: [newTool] });
          }
          lastMsg.blocks = blocks;
        } else if (eventType === "tool_end") {
          // Find last running tool and complete it
          for (let i = blocks.length - 1; i >= 0; i--) {
            const block = blocks[i];
            if (block.type === "tools") {
              const runningTool = block.tools.find(t => t.status === "running");
              if (runningTool) {
                runningTool.status = "completed";
                break;
              }
            }
          }
          lastMsg.blocks = blocks;
        } else if (eventType === "output") {
          // Append text - group with existing text block or create new one
          if (content && content.trim()) {
            if (lastBlock?.type === "text") {
              if (!lastBlock.content.endsWith(content)) {
                lastBlock.content += content;
              }
            } else {
              blocks.push({ type: "text", content });
            }
            lastMsg.blocks = blocks;
          }
        } else if (eventType === "error") {
          // Find running tool or add error text
          let handled = false;
          for (let i = blocks.length - 1; i >= 0; i--) {
            const block = blocks[i];
            if (block.type === "tools") {
              const runningTool = block.tools.find(t => t.status === "running");
              if (runningTool) {
                runningTool.status = "error";
                runningTool.content = (runningTool.content || "") + "\n" + content;
                handled = true;
                break;
              }
            }
          }
          if (!handled) {
            blocks.push({ type: "text", content: `\n\n**Error:** ${content}\n` });
          }
          lastMsg.blocks = blocks;
        } else if (eventType === "done") {
          // Mark all remaining running tools as completed
          for (const block of blocks) {
            if (block.type === "tools") {
              for (const tool of block.tools) {
                if (tool.status === "running") {
                  tool.status = "completed";
                }
              }
            }
          }
          // Clean up - trim last text block if exists
          if (lastBlock?.type === "text") {
            lastBlock.content = lastBlock.content.trim();
          }
          lastMsg.blocks = blocks;
        }

        const newThreads = [...threads];
        newThreads[threadIndex] = { ...thread, messages };
        return { threads: newThreads };
      });
    });

    return unlisten;
  },

  setPreviewUrl: (url: string | null) => set({ previewUrl: url }),
  setCaptureFrame: (fn: (() => Promise<string | null>) | null) => set({ captureFrame: fn }),
  setTestControls: (fn: ((keys: string[], duration?: number) => Promise<{ before: string; after: string } | null>) | null) => set({ testControls: fn }),
  setCaptureNode: (fn) => set({ captureNode: fn }),
  
  setBuildStatus: (status, message = "") => {
    set({ buildStatus: status, buildMessage: message });
    // Auto-clear success/error after delay
    if (status === "success" || status === "error") {
      setTimeout(() => {
        const current = useStore.getState().buildStatus;
        if (current === status) set({ buildStatus: "idle", buildMessage: "" });
      }, 3000);
    }
  },

  loadInputMappings: async () => {
    const { projectPath } = get();
    if (!projectPath) return;
    
    try {
      const mappings = await invoke<{ action: string; keys: string[]; description: string }[]>(
        "get_input_mappings",
        { projectPath }
      );
      console.log('[Store] Loaded input mappings:', mappings);
      set({ inputMappings: mappings });
    } catch (e) {
      console.error('[Store] Failed to load input mappings:', e);
    }
  },

  startPlaytest: async (objective: string) => {
    const { projectPath } = get();
    if (!projectPath) {
      console.error('[Playtest] No project path');
      return;
    }
    
    set({ playtestRunning: true, playtestEvents: [] });
    
    try {
      const result = await invoke<string>("run_playtest", {
        projectPath,
        config: {
          objective,
          max_steps: 30,
          step_delay_ms: 800,
        },
      });
      console.log('[Playtest] Complete:', result);
    } catch (e) {
      console.error('[Playtest] Error:', e);
      set((s) => ({
        playtestEvents: [...s.playtestEvents, {
          event_type: "error",
          message: String(e),
        }],
      }));
    } finally {
      set({ playtestRunning: false });
    }
  },

  initPlaytestListener: async () => {
    const unlisten = await listen<PlaytestEvent>("playtest-event", (event) => {
      const data = event.payload;
      console.log('[Playtest Event]', data.event_type, data.message);
      
      set((s) => {
        const newEvents = [...s.playtestEvents.slice(-50), data];
        
        // Also update the streaming assistant message with progress
        const { threads, activeThreadId } = s;
        if (!activeThreadId) return { ...s, playtestEvents: newEvents };
        
        const threadIndex = threads.findIndex(t => t.id === activeThreadId);
        if (threadIndex === -1) return { ...s, playtestEvents: newEvents };
        
        const thread = threads[threadIndex];
        const messages = [...thread.messages];
        const lastMsg = messages[messages.length - 1];
        
        if (lastMsg?.role === "assistant" && lastMsg.isStreaming) {
          // Build progress text from recent events
          const recentEvents = newEvents.slice(-8);
          const icon = {
            start: "🚀",
            connected: "🔗",
            observation: "👁️",
            action: "🎮",
            complete: "✅",
            error: "❌",
          }[data.event_type] || "•";
          
          let progressText = `🤖 **AI Playtest Running**\n\n`;
          for (const e of recentEvents) {
            const eIcon = { start: "🚀", connected: "🔗", observation: "👁️", action: "🎮", complete: "✅", error: "❌" }[e.event_type] || "•";
            progressText += `${eIcon} ${e.message}\n`;
          }
          
          lastMsg.blocks = [{ type: "text", content: progressText }];
          
          const newThreads = [...threads];
          newThreads[threadIndex] = { ...thread, messages };
          
          return { ...s, playtestEvents: newEvents, threads: newThreads };
        }
        
        return { ...s, playtestEvents: newEvents };
      });
      
      if (data.event_type === "complete" || data.event_type === "error") {
        set({ playtestRunning: false });
      }
    });
    
    return unlisten;
  },

  startAuth: async () => {
    set({ isSigningIn: true });
    try {
      await invoke("start_openrouter_auth");
    } catch (e) {
      console.error("[Auth] Failed to start auth:", e);
      set({ isSigningIn: false });
    }
  },

  initAuthListener: async () => {
    const successUnlisten = await listen("oauth-success", () => {
      console.log("[Auth] OAuth Success");
      set({ isSignedIn: true, isSigningIn: false });
    });

    const errorUnlisten = await listen("oauth-error", (event) => {
      console.error("[Auth] OAuth Error:", event.payload);
      set({ isSigningIn: false });
    });

    return () => {
      successUnlisten();
      errorUnlisten();
    };
  },

  checkAuth: async () => {
    try {
      const settings = await invoke<{ openrouterKey?: string }>("get_settings");
      if (settings.openrouterKey) {
        set({ isSignedIn: true });
      }
    } catch {}
  },
}));
