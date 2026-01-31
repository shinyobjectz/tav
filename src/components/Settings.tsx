import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { invoke } from "@tauri-apps/api/core";
import {
  X,
  Check,
  Loader2,
  Bot,
  Download,
  ExternalLink,
  Gamepad2,
  ChevronRight,
  RefreshCw,
  Key,
  Eye,
  EyeOff,
  Plug,
  Zap,
  Cpu,
  Play,
  Square,
  LogIn
} from "lucide-react";
import { cn } from "@/lib/utils";

type Status = "checking" | "installed" | "not-installed" | "installing";

interface SetupItem {
  id: string;
  name: string;
  description: string;
  status: Status;
  installUrl: string;
  icon: React.ReactNode;
  path?: string;
}

export function Settings({ onClose }: { onClose: () => void }) {
  const {
    isSignedIn,
    isSigningIn,
    startAuth,
    initAuthListener,
    checkAuth,
    projectPath,
  } = useStore();
  
  const [step, setStep] = useState<"auth" | "godot">(isSignedIn ? "godot" : "auth");
  const [apiKey, setApiKey] = useState("");
  const [showApiKey, setShowApiKey] = useState(false);
  const [installStatus, setInstallStatus] = useState<string | null>(null);
  
  const [godot, setGodot] = useState<SetupItem>({
    id: "godot",
    name: "Godot 4.x",
    description: "Game engine",
    status: "checking",
    installUrl: "https://godotengine.org/download",
    icon: <Gamepad2 className="w-4 h-4 text-[#888]" />,
  });
  const [godotMcp, setGodotMcp] = useState<SetupItem>({
    id: "godot-mcp",
    name: "Godot MCP",
    description: "AI ↔ Godot bridge",
    status: "checking",
    installUrl: "https://github.com/Coding-Solo/godot-mcp",
    icon: <Plug className="w-4 h-4 text-[#888]" />,
  });
  const [beads, setBeads] = useState<SetupItem>({
    id: "beads",
    name: "Task Tracking",
    description: "Persistent task memory",
    status: "checking",
    installUrl: "https://github.com/steveyegge/beads",
    icon: <Check className="w-4 h-4 text-[#888]" />,
  });
  const [godotPath, setGodotPath] = useState("");
  const [geminiKey, setGeminiKey] = useState("");
  const [showGeminiKey, setShowGeminiKey] = useState(false);
  const [saving, setSaving] = useState(false);
  
  // NitroGen state
  const [nitrogenStatus, setNitrogenStatus] = useState<{
    installed: boolean;
    checkpoint_exists: boolean;
    server_running: boolean;
    sidecar_available: boolean;
    python_path: string | null;
    nitrogen_path: string | null;
  } | null>(null);
  const [nitrogenLoading, setNitrogenLoading] = useState(false);

  useEffect(() => {
    loadSettings();
    checkAuth();
    
    let unlisten: (() => void) | null = null;
    initAuthListener().then(fn => {
      unlisten = fn;
    });

    const timer = setTimeout(() => {
      detectAll();
      checkNitrogen();
    }, 100);
    
    return () => {
      clearTimeout(timer);
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    if (isSignedIn && step === "auth") {
      setStep("godot");
    }
  }, [isSignedIn]);

  const checkNitrogen = async () => {
    try {
      const status = await invoke<typeof nitrogenStatus>("check_nitrogen_installed");
      setNitrogenStatus(status);
    } catch (e) {
      console.error("Failed to check NitroGen:", e);
    }
  };

  const startNitrogenServer = async () => {
    setNitrogenLoading(true);
    try {
      await invoke("start_nitrogen_server", { checkpointPath: null });
      await checkNitrogen();
    } catch (e) {
      console.error("Failed to start NitroGen:", e);
    } finally {
      setNitrogenLoading(false);
    }
  };

  const stopNitrogenServer = async () => {
    setNitrogenLoading(true);
    try {
      await invoke("stop_nitrogen_server");
      await checkNitrogen();
    } catch (e) {
      console.error("Failed to stop NitroGen:", e);
    } finally {
      setNitrogenLoading(false);
    }
  };

  const loadSettings = async () => {
    try {
      const settings = await invoke<{
        godotPath: string;
        openrouterKey: string;
        geminiKey: string;
      }>("get_settings");
      if (settings.godotPath) setGodotPath(settings.godotPath);
      if (settings.openrouterKey) {
        setApiKey(settings.openrouterKey);
      }
      if (settings.geminiKey) setGeminiKey(settings.geminiKey);
    } catch {}
  };

  const detectAll = async () => {
    const detectGodotEngine = async () => {
      try {
        const path = await invoke<string>("detect_godot");
        setGodot((prev) => ({ ...prev, status: "installed", path }));
        setGodotPath(path);
      } catch {
        setGodot((prev) => ({ ...prev, status: "not-installed" }));
      }
    };

    const detectAndInstallMcp = async () => {
      try {
        const installed = await invoke<boolean>("detect_godot_mcp");
        if (installed) {
          setGodotMcp((prev) => ({ ...prev, status: "installed" }));
        } else {
          setGodotMcp((prev) => ({ ...prev, status: "installing" }));
          try {
            await invoke<string>("install_godot_mcp");
            await invoke("setup_godot_mcp_config");
            setGodotMcp((prev) => ({ ...prev, status: "installed" }));
          } catch {
            setGodotMcp((prev) => ({ ...prev, status: "not-installed" }));
          }
        }
      } catch {
        setGodotMcp((prev) => ({ ...prev, status: "not-installed" }));
      }
    };

    const detectAndInstallBeads = async () => {
      try {
        const installed = await invoke<boolean>("detect_beads");
        if (installed) {
          setBeads((prev) => ({ ...prev, status: "installed" }));
        } else {
          setBeads((prev) => ({ ...prev, status: "installing" }));
          try {
            await invoke<string>("install_beads");
            setBeads((prev) => ({ ...prev, status: "installed" }));
          } catch {
            setBeads((prev) => ({ ...prev, status: "not-installed" }));
          }
        }
      } catch {
        setBeads((prev) => ({ ...prev, status: "not-installed" }));
      }
    };

    await Promise.all([
      detectGodotEngine(),
      detectAndInstallMcp(),
      detectAndInstallBeads(),
    ]);
  };

  const handleSignIn = async () => {
    await startAuth();
  };

  const saveApiKeyAndContinue = async () => {
    setSaving(true);
    try {
      await invoke("save_settings", {
        settings: { godotPath, openrouterKey: apiKey, geminiKey },
      });
      await checkAuth();
    } catch {}
    setSaving(false);
    setStep("godot");
  };

  const installGodot = async () => {
    setGodot((prev) => ({ ...prev, status: "installing" }));
    setInstallStatus("Linking...");
    try {
      const result = await invoke<string>("install_godot");
      setInstallStatus(result);
      const checkInstall = setInterval(async () => {
        try {
          const path = await invoke<string>("detect_godot");
          setGodot((prev) => ({ ...prev, status: "installed", path }));
          setGodotPath(path);
          setInstallStatus(null);
          clearInterval(checkInstall);
        } catch {}
      }, 2000);
      setTimeout(() => {
        clearInterval(checkInstall);
        setInstallStatus(null);
      }, 60000);
    } catch (e) {
      setInstallStatus(`Error: ${e}`);
      setGodot((prev) => ({ ...prev, status: "not-installed" }));
    }
  };

  const installGodotMcp = async () => {
    setGodotMcp((prev) => ({ ...prev, status: "installing" }));
    try {
      await invoke<string>("install_godot_mcp");
      await invoke("setup_godot_mcp_config");
      setGodotMcp((prev) => ({ ...prev, status: "installed" }));
    } catch (e) {
      console.error("Failed to install Godot MCP:", e);
      setGodotMcp((prev) => ({ ...prev, status: "not-installed" }));
    }
  };

  const saveAndClose = async () => {
    setSaving(true);
    try {
      await invoke("save_settings", {
        settings: { godotPath, openrouterKey: apiKey, geminiKey },
      });
    } catch {}
    setSaving(false);
    onClose();
  };

  const openUrl = (url: string) => invoke("open_url", { url });

  const StatusBadge = ({ status }: { status: Status }) => {
    if (status === "checking" || status === "installing") {
      return <Loader2 className="w-3 h-3 animate-spin text-zinc-600" />;
    }
    if (status === "installed") {
      return <Check className="w-3 h-3 text-zinc-400" />;
    }
    return <Download className="w-3 h-3 text-zinc-600" />;
  };

  return (
    <div className="fixed inset-0 bg-[#0a0a0a]/95 z-50 flex items-center justify-center p-4 backdrop-blur-sm">
      <div className="bg-[#0f0f0f] border border-zinc-900 rounded-2xl shadow-2xl w-full max-w-md overflow-hidden animate-in fade-in zoom-in-95 duration-300">
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-900/50 bg-[#0a0a0a]/20">
          <div className="flex items-center gap-3">
            <div className="w-6 h-6 rounded-lg bg-[#0a0a0a] border border-zinc-900 flex items-center justify-center shadow-lg">
              <img src="/tav-logo.png" alt="Tav" className="w-4 h-4 opacity-80" />
            </div>
            <div className="flex flex-col">
              <span className="text-xs font-black uppercase tracking-[0.2em] text-zinc-200">System Configuration</span>
              <span className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest mt-0.5">
                Node Phase {step === "auth" ? "01" : "02"}
              </span>
            </div>
          </div>
          <button onClick={onClose} className="text-zinc-700 hover:text-zinc-200 transition-colors p-1">
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="max-h-[70vh] overflow-y-auto no-scrollbar">
          {step === "auth" && (
            <div className="p-8 space-y-8 text-center">
              <div className="space-y-3">
                <div className="relative inline-block group">
                  <div className="absolute -inset-4 bg-zinc-500/5 rounded-full blur-2xl group-hover:bg-zinc-500/10 transition-all duration-1000"></div>
                  <div className="relative w-20 h-20 rounded-[2rem] bg-[#0a0a0a] border border-zinc-900 flex items-center justify-center shadow-2xl mx-auto overflow-hidden">
                    <img src="/tav-logo.png" alt="Tav" className="w-12 h-12 opacity-90" />
                  </div>
                </div>
                <h2 className="text-xl font-bold tracking-tight text-zinc-100">Establish Session</h2>
                <p className="text-[11px] text-zinc-600 max-w-xs mx-auto leading-relaxed">
                  Authentication via OpenRouter is required to initialize intelligence core.
                </p>
              </div>

              {isSignedIn ? (
                <div className="space-y-6">
                  <div className="flex flex-col items-center gap-3">
                    <div className="w-10 h-10 rounded-full bg-[#0a0a0a] border border-zinc-800 flex items-center justify-center text-zinc-400">
                      <Check className="w-5 h-5" />
                    </div>
                    <span className="text-[10px] font-black uppercase tracking-[0.2em] text-zinc-500">Authentication Active</span>
                  </div>
                  <Button size="sm" className="h-12 w-full rounded-xl bg-zinc-100 hover:bg-white text-black font-black uppercase tracking-[0.2em] text-[10px] shadow-xl hover:scale-[1.02] transition-all" onClick={() => setStep("godot")}>
                    Continue to environment
                  </Button>
                </div>
              ) : (
                <div className="space-y-6">
                  <div className="space-y-4">
                    <div className="relative group">
                      <Key className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-800" />
                      <Input
                        type={showApiKey ? "text" : "password"}
                        placeholder="sk-or-..."
                        value={apiKey}
                        onChange={(e) => setApiKey(e.target.value)}
                        className="h-12 pl-11 pr-11 bg-[#0a0a0a] border-zinc-900 focus-visible:border-zinc-700 focus-visible:ring-0 rounded-xl font-mono text-xs text-zinc-300 placeholder:text-zinc-800 transition-all"
                      />
                    </div>
                    <Button
                      size="lg"
                      className="h-12 w-full rounded-xl bg-white text-black hover:bg-zinc-200 font-black uppercase tracking-[0.2em] text-[10px] transition-all disabled:opacity-20"
                      onClick={handleSignIn}
                      disabled={isSigningIn}
                    >
                      {isSigningIn ? (
                        <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      ) : (
                        <>
                          <LogIn className="w-4 h-4 mr-2" />
                          Initialize Uplink
                        </>
                      )}
                    </Button>
                  </div>
                  
                  <div className="flex flex-col gap-2">
                    <button
                      className="text-[9px] font-black uppercase tracking-widest text-zinc-700 hover:text-zinc-400 transition-colors underline decoration-zinc-900 underline-offset-8"
                      onClick={() => openUrl("https://openrouter.ai/keys")}
                    >
                      Generate access token
                    </button>
                    <button
                      className="text-[9px] font-black uppercase tracking-widest text-zinc-800 hover:text-zinc-600 transition-colors"
                      onClick={saveApiKeyAndContinue}
                      disabled={!apiKey.trim()}
                    >
                      Manual entry
                    </button>
                  </div>
                </div>
              )}
            </div>
          )}

          {step === "godot" && (
            <div className="p-6 space-y-8">
              <div className="space-y-4">
                <Label className="text-[9px] font-black text-zinc-700 uppercase tracking-[0.3em]">Execution Environment</Label>
                <div className={cn("p-4 rounded-xl border transition-all duration-500", godot.status === "installed" ? "border-zinc-800 bg-zinc-950/50 shadow-inner" : "border-zinc-900 bg-[#0a0a0a]")}>
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-xl bg-zinc-900 border border-zinc-800 flex items-center justify-center shrink-0 shadow-lg text-zinc-500">
                      <Gamepad2 className="w-5 h-5" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-bold text-zinc-300">Godot Engine</span>
                        <StatusBadge status={godot.status} />
                      </div>
                      <span className="text-[11px] text-zinc-600 truncate block font-mono opacity-60">
                        {godot.status === "installed" ? godotPath || "Binary Linked" : "Engine core missing"}
                      </span>
                    </div>
                  </div>
                  {godot.status !== "installed" && (
                    <div className="mt-6 space-y-3">
                      <Input placeholder="Absolute binary path..." value={godotPath} onChange={(e) => setGodotPath(e.target.value)} className="h-10 bg-zinc-950 border-zinc-900 focus-visible:border-zinc-800 focus-visible:ring-0 rounded-lg font-mono text-[10px] text-zinc-400 placeholder:text-zinc-800 transition-all" />
                      <Button variant="outline" size="sm" className="h-10 w-full rounded-lg border-zinc-800 bg-transparent hover:bg-zinc-900 text-zinc-400 font-bold uppercase tracking-widest text-[9px]" onClick={installGodot} disabled={godot.status === "installing"}>
                        {godot.status === "installing" ? <Loader2 className="w-3 h-3 mr-2 animate-spin" /> : <Download className="w-3 h-3 mr-2" />}
                        Autonomous Link
                      </Button>
                    </div>
                  )}
                </div>
              </div>

              <div className="space-y-4">
                <Label className="text-[9px] font-black text-zinc-700 uppercase tracking-[0.3em]">AI Integrations</Label>
                <div className={cn("p-4 rounded-xl border transition-all duration-500", beads.status === "installed" ? "border-zinc-800 bg-zinc-950/50 shadow-inner" : "border-zinc-900 bg-[#0a0a0a]")}>
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-xl bg-zinc-900 border border-zinc-800 flex items-center justify-center shrink-0 shadow-lg text-zinc-500">
                      <Check className="w-5 h-5" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-bold text-zinc-300">Task Tracking</span>
                        <StatusBadge status={beads.status} />
                      </div>
                      <span className="text-[11px] text-zinc-600 leading-none">Persistent task memory for AI</span>
                    </div>
                    {beads.status === "not-installed" && (
                      <button
                        className="text-[9px] font-black uppercase tracking-widest text-zinc-700 hover:text-zinc-400 transition-colors"
                        onClick={() => openUrl(beads.installUrl)}
                      >
                        Install
                      </button>
                    )}
                  </div>
                </div>
              </div>

              <div className="space-y-4">
                <Label className="text-[9px] font-black text-zinc-700 uppercase tracking-[0.3em]">Neural Sidecar</Label>
                <div className={cn("p-4 rounded-xl border transition-all duration-500", nitrogenStatus?.installed && nitrogenStatus?.checkpoint_exists && nitrogenStatus?.sidecar_available ? "border-zinc-800 bg-zinc-950/50 shadow-inner" : "border-zinc-900 bg-[#0a0a0a]")}>
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-xl bg-zinc-900 border border-zinc-800 flex items-center justify-center shrink-0 shadow-lg text-zinc-500">
                      <Cpu className="w-5 h-5" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-bold text-zinc-300">NitroGen</span>
                        {nitrogenStatus?.installed && nitrogenStatus?.checkpoint_exists && nitrogenStatus?.sidecar_available && <Check className="w-3.5 h-3.5 text-zinc-400" />}
                        {nitrogenStatus?.server_running && (
                          <div className="flex items-center gap-1.5 px-2 py-0.5 bg-zinc-900 border border-zinc-800 rounded-full">
                            <div className="w-1 h-1 rounded-full bg-zinc-200 animate-pulse" />
                            <span className="text-[8px] font-black uppercase tracking-widest text-zinc-400">Active</span>
                          </div>
                        )}
                      </div>
                      <span className="text-[11px] text-zinc-600 leading-none">NVIDIA vision-to-action module</span>
                    </div>
                  </div>
                  <div className="mt-6 space-y-4">
                    {nitrogenStatus === null ? (
                      <div className="flex items-center gap-3 py-2 opacity-40">
                        <Loader2 className="w-3 h-3 animate-spin" />
                        <span className="text-[10px] font-bold uppercase tracking-widest text-zinc-600">Scanning...</span>
                      </div>
                    ) : !nitrogenStatus.sidecar_available ? (
                      <div className="p-3 rounded-lg bg-zinc-950 border border-zinc-900 opacity-60 hover:opacity-100 transition-opacity">
                        <p className="text-[9px] font-black uppercase tracking-[0.2em] text-zinc-700 mb-2">Build Required</p>
                        <code className="text-[9px] text-zinc-500 font-mono block break-all leading-relaxed">pip install -r requirements.txt && python build-sidecar.py</code>
                      </div>
                    ) : (
                      <div className="flex gap-2">
                        {nitrogenStatus.server_running ? (
                          <Button variant="outline" size="sm" className="flex-1 h-10 rounded-xl border-zinc-800 bg-transparent hover:bg-zinc-900 text-zinc-400 font-black uppercase tracking-widest text-[9px]" onClick={stopNitrogenServer} disabled={nitrogenLoading}>
                            {nitrogenLoading ? <Loader2 className="w-3.5 h-3.5 mr-2 animate-spin" /> : <Square className="w-3 h-3 mr-2" />} Terminate
                          </Button>
                        ) : (
                          <Button variant="outline" size="sm" className="flex-1 h-10 rounded-xl border-zinc-800 bg-transparent hover:bg-zinc-900 text-zinc-400 font-black uppercase tracking-widest text-[9px]" onClick={startNitrogenServer} disabled={nitrogenLoading}>
                            {nitrogenLoading ? <Loader2 className="w-3.5 h-3.5 mr-2 animate-spin" /> : <Play className="w-3 h-3 mr-2" />} Initialize
                          </Button>
                        )}
                        <Button variant="ghost" size="icon" className="h-10 w-10 text-zinc-800 hover:text-zinc-400" onClick={checkNitrogen}><RefreshCw className="w-3.5 h-3.5" /></Button>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              <div className="pt-6 border-t border-zinc-900/50 flex gap-3">
                <Button variant="ghost" size="sm" className="h-12 px-6 rounded-xl text-[10px] font-black uppercase tracking-widest text-zinc-700 hover:text-zinc-400 hover:bg-zinc-900" onClick={() => setStep("auth")}>Previous</Button>
                <Button size="sm" className="h-12 flex-1 rounded-xl bg-zinc-100 hover:bg-white text-black font-black uppercase tracking-[0.2em] text-[10px] shadow-xl hover:scale-[1.02] transition-all" onClick={saveAndClose} disabled={saving}>
                  {saving && <Loader2 className="w-4 h-4 mr-3 animate-spin" />} Complete Setup
                </Button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
