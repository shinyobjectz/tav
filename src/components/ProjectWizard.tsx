import { useState } from "react";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "@/hooks/useStore";
import { Gamepad2, FolderOpen, Sparkles } from "lucide-react";
import { TemplateSelector } from "./TemplateSelector";

export function ProjectWizard() {
  const { openProject, loadFiles } = useStore();
  const [showTemplates, setShowTemplates] = useState(false);
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createProject = async (config: { template: string; dimension: string; name: string; path: string }) => {
    setCreating(true);
    setError(null);

    try {
      const fullPath = await invoke<string>("create_project_from_template", {
        name: config.name,
        parentPath: config.path,
        dimension: config.dimension,
        template: config.template,
      });

      // Load the new project
      const name = fullPath.split(/[\\/]/).pop() || config.name;
      useStore.setState({ projectPath: fullPath, projectName: name });
      await loadFiles(fullPath);

      // Save as last project
      const settings = await invoke<Record<string, unknown>>("get_settings");
      await invoke("save_settings", { settings: { ...settings, lastProjectPath: fullPath } });
    } catch (e) {
      setError(`Failed to create project: ${e}`);
      setCreating(false);
    }
  };

  // Start screen
  if (!showTemplates) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-8 text-center">
        <div className="space-y-6 max-w-md">
          <div className="space-y-2">
            <Gamepad2 className="w-12 h-12 mx-auto text-[#555]" />
            <h2 className="text-lg font-medium">Welcome to Tav</h2>
            <p className="text-sm text-[#888]">
              Open an existing Godot project or create a new one from a template.
            </p>
          </div>

          <div className="space-y-3">
            <Button variant="outline" className="w-full h-12 justify-start gap-3" onClick={openProject}>
              <FolderOpen className="w-5 h-5" />
              <div className="text-left">
                <div className="text-sm font-medium">Open Project</div>
                <div className="text-xs text-[#666]">Browse for existing Godot project</div>
              </div>
            </Button>

            <Button className="w-full h-12 justify-start gap-3" onClick={() => setShowTemplates(true)}>
              <Sparkles className="w-5 h-5" />
              <div className="text-left">
                <div className="text-sm font-medium">New Project</div>
                <div className="text-xs text-[#999]">Create from template</div>
              </div>
            </Button>
          </div>
        </div>
      </div>
    );
  }

  // Template selection
  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8">
      <TemplateSelector
        onComplete={createProject}
        onCancel={() => setShowTemplates(false)}
        loading={creating}
        error={error}
        show3DCharacterOption={false}
      />
    </div>
  );
}
