import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Loader2, Download, CheckCircle, AlertCircle, ExternalLink } from "lucide-react";

interface SetupStatus {
  godotInstalled: boolean;
  godotPath: string | null;
  godotVersion: string | null;
  templatesInstalled: boolean;
}

export function SetupWizard({ onComplete }: { onComplete: () => void }) {
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [checking, setChecking] = useState(true);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkSetup();
  }, []);

  const checkSetup = async () => {
    setChecking(true);
    setError(null);
    try {
      const result = await invoke<SetupStatus>("check_setup_status");
      setStatus(result);
      if (result.godotInstalled && result.templatesInstalled) {
        onComplete();
      }
    } catch (e) {
      setError(String(e));
    }
    setChecking(false);
  };

  const downloadTemplates = async () => {
    setDownloading(true);
    setError(null);
    setDownloadProgress(0);
    
    try {
      // Start download with progress updates
      const result = await invoke<string>("ensure_export_templates");
      console.log("Download result:", result);
      await checkSetup();
    } catch (e) {
      setError(String(e));
    }
    setDownloading(false);
  };

  const openGodotDownload = () => {
    invoke("open_url", { url: "https://godotengine.org/download" });
  };

  if (checking) {
    return (
      <div className="fixed inset-0 bg-[#0a0a0a]/95 flex items-center justify-center z-50 backdrop-blur-xl">
        <div className="flex flex-col items-center gap-6 animate-in fade-in duration-1000">
          <div className="relative">
            <div className="absolute -inset-8 bg-zinc-500/5 rounded-full blur-3xl animate-pulse" />
            <Loader2 className="w-12 h-12 animate-spin text-zinc-800 relative z-10" />
          </div>
          <div className="space-y-1 text-center">
            <p className="text-[10px] uppercase font-black tracking-[0.3em] text-zinc-600">Initializing Core</p>
            <p className="text-[9px] font-mono text-zinc-800 uppercase">Scanning environment setup...</p>
          </div>
        </div>
      </div>
    );
  }

  const godotOk = status?.godotInstalled;
  const templatesOk = status?.templatesInstalled;

  return (
    <div className="fixed inset-0 bg-[#0a0a0a]/95 flex items-center justify-center z-50 backdrop-blur-xl">
      <div className="bg-[#0f0f0f] border border-zinc-900 rounded-[2.5rem] p-10 max-w-lg w-full mx-4 shadow-2xl animate-in zoom-in-95 duration-500">
        <div className="flex flex-col items-center text-center mb-12">
          <div className="w-16 h-16 rounded-3xl bg-[#0a0a0a] border border-zinc-900 flex items-center justify-center mb-6 shadow-2xl">
            <img src="/tav-logo.png" alt="Tav" className="w-10 h-10 opacity-80" />
          </div>
          <h2 className="text-2xl font-black tracking-tighter text-zinc-100 uppercase tracking-[0.1em] mb-2">System Dependency</h2>
          <p className="text-[10px] uppercase font-black tracking-[0.2em] text-zinc-700">Environment validation required</p>
        </div>

        <div className="space-y-4 mb-10">
          {/* Godot Status */}
          <div className={cn("flex items-center gap-4 p-5 rounded-2xl border transition-all duration-500", godotOk ? "bg-zinc-950/50 border-zinc-800" : "bg-[#0a0a0a] border-zinc-900")}>
            <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center shrink-0 border", godotOk ? "bg-zinc-900 border-zinc-800 text-zinc-400" : "bg-[#0a0a0a] border-red-900/30 text-red-900/50")}>
              <Monitor className="w-5 h-5" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-[11px] font-black uppercase tracking-widest text-zinc-300 mb-1">Godot Engine</div>
              {godotOk ? (
                <div className="text-[9px] font-mono text-zinc-600 truncate">{status?.godotVersion} Established</div>
              ) : (
                <div className="text-[9px] font-mono text-red-900 uppercase">Binary not found</div>
              )}
            </div>
            {!godotOk && (
              <Button size="sm" variant="outline" className="h-9 px-4 rounded-full text-[9px] font-black uppercase tracking-widest border-zinc-800 bg-transparent hover:bg-white hover:text-black transition-all" onClick={openGodotDownload}>
                Retrieve
              </Button>
            )}
          </div>

          {/* Templates Status */}
          <div className={cn("flex items-center gap-4 p-5 rounded-2xl border transition-all duration-500", templatesOk ? "bg-zinc-950/50 border-zinc-800" : "bg-[#0a0a0a] border-zinc-900")}>
            <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center shrink-0 border", templatesOk ? "bg-zinc-900 border-zinc-800 text-zinc-400" : downloading ? "bg-zinc-900 border-zinc-700 text-zinc-500" : "bg-[#0a0a0a] border-zinc-900 text-zinc-800")}>
              {downloading ? <Loader2 className="w-5 h-5 animate-spin" /> : <Download className="w-5 h-5" />}
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-[11px] font-black uppercase tracking-widest text-zinc-300 mb-1">Web Runtime</div>
              {templatesOk ? (
                <div className="text-[9px] font-mono text-zinc-600">Assets Verified</div>
              ) : downloading ? (
                <div className="text-[9px] font-mono text-zinc-500 animate-pulse">Syncing... (~1GB)</div>
              ) : (
                <div className="text-[9px] font-mono text-zinc-800 uppercase">Awaiting Installation</div>
              )}
            </div>
            {!templatesOk && godotOk && !downloading && (
              <Button size="sm" className="h-9 px-4 rounded-full bg-zinc-100 hover:bg-white text-black text-[9px] font-black uppercase tracking-widest transition-all" onClick={downloadTemplates}>
                Initialize
              </Button>
            )}
          </div>
        </div>

        {error && (
          <div className="p-4 rounded-xl bg-red-950/10 border border-red-900/20 mb-8">
            <p className="text-[10px] font-mono text-red-900 uppercase tracking-widest leading-relaxed">{error}</p>
          </div>
        )}

        <div className="flex flex-col gap-4">
          <Button
            className={cn("h-14 w-full rounded-2xl font-black uppercase tracking-[0.3em] text-[11px] shadow-2xl transition-all", godotOk && templatesOk ? "bg-zinc-100 hover:bg-white text-black" : "bg-zinc-900 text-zinc-700 border border-zinc-800 hover:text-zinc-400")}
            onClick={godotOk && templatesOk ? onComplete : checkSetup}
            disabled={downloading}
          >
            {godotOk && templatesOk ? "Establish Connection" : "Run Diagnostics"}
          </Button>
          
          <p className="text-[8px] uppercase font-bold tracking-[0.2em] text-zinc-800 text-center leading-relaxed">
            Note: Manual template installation possible via<br/>Godot Editor → Manage Export Templates
          </p>
        </div>
      </div>
    </div>
  );
}
