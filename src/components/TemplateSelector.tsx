import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Box,
  Square,
  User,
  Eye,
  Move,
  Grid3X3,
  Puzzle,
  Sparkles,
  ChevronLeft,
  Loader2,
} from "lucide-react";
import { cn } from "@/lib/utils";

type GameDimension = "2d" | "3d";
type GameTemplate = "platformer" | "top-down" | "first-person" | "third-person" | "puzzle" | "empty";

interface TemplateOption {
  id: GameTemplate;
  name: string;
  description: string;
  icon: React.ReactNode;
  dimensions: GameDimension[];
}

const TEMPLATES: TemplateOption[] = [
  { id: "platformer", name: "Platformer", description: "Side-scrolling jump & run", icon: <Move className="w-5 h-5" />, dimensions: ["2d"] },
  { id: "top-down", name: "Top-Down", description: "Bird's eye view action/RPG", icon: <Grid3X3 className="w-5 h-5" />, dimensions: ["2d"] },
  { id: "first-person", name: "First Person", description: "FPS / exploration", icon: <Eye className="w-5 h-5" />, dimensions: ["3d"] },
  { id: "third-person", name: "Third Person", description: "Over-shoulder camera", icon: <User className="w-5 h-5" />, dimensions: ["3d"] },
  { id: "puzzle", name: "Puzzle", description: "Logic & matching games", icon: <Puzzle className="w-5 h-5" />, dimensions: ["2d", "3d"] },
  { id: "empty", name: "Empty Project", description: "Start from scratch", icon: <Sparkles className="w-5 h-5" />, dimensions: ["2d", "3d"] },
];

interface TemplateSelectorProps {
  /** If provided, skip name/location step and use this path */
  existingPath?: string;
  existingName?: string;
  /** Called when template selection is complete */
  onComplete: (config: { template: GameTemplate; dimension: GameDimension; name: string; path: string }) => void;
  /** Called when user wants to go back/cancel */
  onCancel?: () => void;
  /** Show loading state */
  loading?: boolean;
  /** Progress percentage for downloads */
  progress?: number | null;
  /** Error message to display */
  error?: string | null;
  /** Show character download option for 3D */
  show3DCharacterOption?: boolean;
  onCharacterOptionChange?: (include: boolean) => void;
  includeCharacter?: boolean;
}

export function TemplateSelector({
  existingPath,
  existingName,
  onComplete,
  onCancel,
  loading = false,
  progress = null,
  error = null,
  show3DCharacterOption = true,
  onCharacterOptionChange,
  includeCharacter = true,
}: TemplateSelectorProps) {
  const needsLocation = !existingPath;
  const [step, setStep] = useState<"dimension" | "template" | "location">("dimension");
  const [dimension, setDimension] = useState<GameDimension | null>(null);
  const [template, setTemplate] = useState<GameTemplate | null>(null);
  const [projectName, setProjectName] = useState(existingName || "");
  const [projectPath, setProjectPath] = useState("");

  const filteredTemplates = TEMPLATES.filter(t => !dimension || t.dimensions.includes(dimension));
  const totalSteps = needsLocation ? 3 : 2;
  const currentStep = step === "dimension" ? 1 : step === "template" ? 2 : 3;

  const selectFolder = async () => {
    try {
      const selected = await open({ directory: true, title: "Select folder for new project" });
      if (selected && typeof selected === "string") setProjectPath(selected);
    } catch {}
  };

  const handleDimensionSelect = (dim: GameDimension) => {
    setDimension(dim);
    setStep("template");
  };

  const handleTemplateSelect = (t: GameTemplate) => {
    setTemplate(t);
    if (needsLocation) {
      setStep("location");
    } else {
      // Use existing path - complete immediately
      onComplete({ template: t, dimension: dimension!, name: existingName || "", path: existingPath! });
    }
  };

  const handleCreate = () => {
    if (!template || !dimension || !projectName || !projectPath) return;
    onComplete({ template, dimension, name: projectName, path: projectPath });
  };

  const goBack = () => {
    if (step === "location") setStep("template");
    else if (step === "template") setStep("dimension");
    else onCancel?.();
  };

  return (
    <div className="space-y-10 max-w-md w-full animate-in fade-in slide-in-from-bottom-4 duration-700">
      {/* Step indicator */}
      <div className="space-y-3 text-center">
        <div className="flex items-center justify-center gap-3">
          <div className="h-[1px] w-8 bg-zinc-900" />
          <span className="text-[9px] font-black uppercase tracking-[0.3em] text-zinc-800">
            Phase {currentStep} of {totalSteps}
          </span>
          <div className="h-[1px] w-8 bg-zinc-900" />
        </div>
        <h2 className="text-2xl font-black tracking-tighter text-zinc-100">
          {step === "dimension" && "Select Logic Domain"}
          {step === "template" && "Load Blueprint"}
          {step === "location" && "Target Parameters"}
        </h2>
      </div>

      {/* Dimension selection */}
      {step === "dimension" && (
        <div className="grid grid-cols-2 gap-4">
          <button
            onClick={() => handleDimensionSelect("2d")}
            className={cn(
              "p-8 rounded-2xl border transition-all duration-500 group",
              dimension === "2d" ? "border-zinc-200 bg-white text-black scale-105" : "border-zinc-900 bg-[#0a0a0a] text-zinc-500 hover:border-zinc-700 hover:bg-zinc-950"
            )}
          >
            <Square className={cn("w-12 h-12 mx-auto mb-4 transition-colors", dimension === "2d" ? "text-black" : "text-zinc-800 group-hover:text-zinc-600")} />
            <div className="text-xs font-black uppercase tracking-widest">2D Domain</div>
            <div className={cn("text-[10px] mt-2 font-medium opacity-60", dimension === "2d" ? "text-black" : "text-zinc-700")}>Sprites & Vectors</div>
          </button>
          <button
            onClick={() => handleDimensionSelect("3d")}
            className={cn(
              "p-8 rounded-2xl border transition-all duration-500 group",
              dimension === "3d" ? "border-zinc-200 bg-white text-black scale-105" : "border-zinc-900 bg-[#0a0a0a] text-zinc-500 hover:border-zinc-700 hover:bg-zinc-950"
            )}
          >
            <Box className={cn("w-12 h-12 mx-auto mb-4 transition-colors", dimension === "3d" ? "text-black" : "text-zinc-800 group-hover:text-zinc-600")} />
            <div className="text-xs font-black uppercase tracking-widest">3D Domain</div>
            <div className={cn("text-[10px] mt-2 font-medium opacity-60", dimension === "3d" ? "text-black" : "text-zinc-700")}>Meshes & Depth</div>
          </button>
        </div>
      )}

      {/* Template selection */}
      {step === "template" && (
        <div className="space-y-6">
          <div className="grid grid-cols-2 gap-3">
            {filteredTemplates.map((t) => (
              <button
                key={t.id}
                onClick={() => handleTemplateSelect(t.id)}
                disabled={loading}
                className={cn(
                  "p-5 rounded-xl border transition-all duration-500 text-left group",
                  template === t.id ? "border-zinc-200 bg-white text-black shadow-2xl" : "border-zinc-900 bg-[#0f0f0f] text-zinc-500 hover:border-zinc-700"
                )}
              >
                <div className="flex items-center gap-4">
                  <div className={cn("w-10 h-10 rounded-lg flex items-center justify-center transition-colors", template === t.id ? "bg-zinc-100 text-black" : "bg-[#0a0a0a] border border-zinc-900 text-zinc-700 group-hover:text-zinc-400")}>
                    {t.icon}
                  </div>
                  <div>
                    <div className="text-[11px] font-black uppercase tracking-widest leading-none mb-1">{t.name}</div>
                    <div className={cn("text-[9px] font-medium leading-tight", template === t.id ? "text-zinc-600" : "text-zinc-800")}>{t.description}</div>
                  </div>
                </div>
              </button>
            ))}
          </div>

          {/* 3D Character option */}
          {show3DCharacterOption && dimension === "3d" && (
            <label className="flex items-center gap-3 text-[10px] font-bold uppercase tracking-widest text-zinc-700 cursor-pointer hover:text-zinc-400 justify-center transition-colors">
              <div className="relative">
                <input
                  type="checkbox"
                  checked={includeCharacter}
                  onChange={(e) => onCharacterOptionChange?.(e.target.checked)}
                  className="peer sr-only"
                />
                <div className="w-4 h-4 border border-zinc-800 rounded bg-zinc-950 peer-checked:bg-zinc-200 peer-checked:border-zinc-200 transition-all" />
                <Check className="absolute inset-0 w-4 h-4 text-black opacity-0 peer-checked:opacity-100 transition-opacity" />
              </div>
              Neural Avatar Uplink
            </label>
          )}

          {/* Loading state for direct creation (no location step) */}
          {!needsLocation && loading && (
            <div className="flex flex-col items-center gap-4 animate-in fade-in duration-500">
              <div className="flex items-center gap-3">
                <Loader2 className="w-4 h-4 animate-spin text-zinc-600" />
                <span className="text-[10px] font-black uppercase tracking-[0.2em] text-zinc-600">
                  {progress !== null ? `Synchronizing Data (${progress}%)` : "Generating Blueprint"}
                </span>
              </div>
              {progress !== null && (
                <div className="w-full bg-zinc-950 border border-zinc-900 rounded-full h-1.5 overflow-hidden">
                  <div className="bg-zinc-400 h-full transition-all duration-500 shadow-[0_0_10px_rgba(255,255,255,0.1)]" style={{ width: `${progress}%` }} />
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Location step (only for new projects) */}
      {step === "location" && (
        <div className="space-y-8 text-left animate-in slide-in-from-right-4 duration-500">
          <div className="space-y-3">
            <Label className="text-[9px] font-black text-zinc-700 uppercase tracking-[0.3em]">Designation</Label>
            <Input
              placeholder="System-Identifier"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value.replace(/[^a-zA-Z0-9-_]/g, "-"))}
              className="h-12 bg-[#0a0a0a] border-zinc-900 focus-visible:border-zinc-700 focus-visible:ring-0 rounded-xl font-mono text-xs text-zinc-300 placeholder:text-zinc-800 transition-all"
            />
          </div>
          <div className="space-y-3">
            <Label className="text-[9px] font-black text-zinc-700 uppercase tracking-[0.3em]">Storage Vector</Label>
            <div className="flex gap-2">
              <Input placeholder="Select target directory..." value={projectPath} readOnly className="h-12 bg-[#0a0a0a] border-zinc-900 focus-visible:border-zinc-700 focus-visible:ring-0 rounded-xl font-mono text-[10px] text-zinc-500 flex-1" />
              <Button variant="outline" className="h-12 px-6 rounded-xl border-zinc-800 bg-transparent hover:bg-zinc-900 text-zinc-400 font-bold uppercase tracking-widest text-[9px]" onClick={selectFolder}>Browse</Button>
            </div>
          </div>

          {error && <p className="text-[10px] font-mono text-red-900 uppercase tracking-widest">{error}</p>}

          <div className="p-5 rounded-2xl bg-zinc-950/50 border border-zinc-900 text-[10px] font-bold uppercase tracking-[0.15em] text-zinc-600">
            <div className="text-zinc-400 mb-3 border-b border-zinc-900 pb-2">Deployment Specs</div>
            <div className="flex justify-between mb-1">
              <span>Domain</span>
              <span className="text-zinc-200">{dimension?.toUpperCase()}</span>
            </div>
            <div className="flex justify-between mb-1">
              <span>Blueprint</span>
              <span className="text-zinc-200">{TEMPLATES.find((t) => t.id === template)?.name}</span>
            </div>
            {projectPath && projectName && (
              <div className="mt-3 text-[9px] font-mono text-zinc-800 lowercase truncate tracking-tight">{projectPath}/{projectName}</div>
            )}
          </div>

          <Button
            className="h-14 w-full rounded-2xl bg-zinc-100 hover:bg-white text-black font-black uppercase tracking-[0.3em] text-[11px] shadow-2xl transition-all hover:scale-[1.02] active:scale-95 disabled:opacity-20"
            onClick={handleCreate}
            disabled={!projectName || !projectPath || loading}
          >
            {loading ? <Loader2 className="w-5 h-5 mr-3 animate-spin" /> : <Sparkles className="w-5 h-5 mr-3" />}
            Initialize Environment
          </Button>
        </div>
      )}

      {/* Back button */}
      {(step !== "dimension" || onCancel) && (
        <button 
          onClick={goBack} 
          disabled={loading} 
          className="mx-auto flex items-center gap-2 text-[9px] font-black uppercase tracking-[0.3em] text-zinc-800 hover:text-zinc-400 transition-colors py-4"
        >
          <ChevronLeft className="w-3 h-3" />
          Revert Phase
        </button>
      )}
    </div>
  );
}
