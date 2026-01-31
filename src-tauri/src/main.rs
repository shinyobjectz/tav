#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod templates;
mod animations;
mod controls;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use templates::*;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Run a command silently (no console window on Windows)
#[cfg(windows)]
fn silent_cmd(program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
}

#[cfg(not(windows))]
fn silent_cmd(program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(program).args(args).output()
}

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub openrouter_key: Option<String>,
    pub goose_model: Option<String>,
    pub gemini_key: Option<String>,
    pub godot_path: Option<String>,
    pub godot_mcp_installed: Option<bool>,
    pub auto_connect: Option<bool>,
    pub last_project_path: Option<String>,
}

// ============================================================================
// Game Playing Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFrame {
    pub screenshot: String,
    pub state: serde_json::Value,
    pub logs: Vec<String>,
    pub frame_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAction {
    pub function: String,
    pub args: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub reasoning: String,
    pub actions: Vec<GameAction>,
}

pub struct GameSession {
    pub id: String,
    pub process: Option<std::process::Child>,
    pub project_path: String,
    pub scene_path: String,
    pub frame_count: u32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvent {
    pub event_type: String,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_args: Option<String>,
}

pub struct AppState {
    settings: Mutex<AppSettings>,
    game_sessions: Mutex<std::collections::HashMap<String, GameSession>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(AppSettings::default()),
            game_sessions: Mutex::new(std::collections::HashMap::new()),
        }
    }
}

// Templates imported from templates.rs module

// ============================================================================
// Asset Downloads from Cloudflare R2
// ============================================================================

const R2_BASE_URL: &str = "https://pub-b3ceaf5076804d56bc32fe9d83e9a3a9.r2.dev";

#[derive(Debug, Serialize, Clone)]
struct DownloadProgress {
    asset: String,
    downloaded: u64,
    total: u64,
    percent: u8,
}

#[tauri::command]
async fn download_asset(
    asset_name: String,
    destination: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    
    let url = format!("{}/{}", R2_BASE_URL, asset_name);
    let dest_path = Path::new(&destination);
    
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    println!("[download_asset] Downloading: {}", url);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client.get(&url).send().await
        .map_err(|e| format!("Download failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    println!("[download_asset] Size: {} bytes", total_size);
    
    let mut file = tokio::fs::File::create(&dest_path).await
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    let mut downloaded: u64 = 0;
    let mut last_percent: u8 = 0;
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk).await.map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let percent = ((downloaded * 100) / total_size) as u8;
            if percent > last_percent {
                last_percent = percent;
                let _ = app.emit("download-progress", DownloadProgress {
                    asset: asset_name.clone(),
                    downloaded,
                    total: total_size,
                    percent,
                });
            }
        }
    }
    
    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    println!("[download_asset] Complete: {}", destination);
    Ok(destination)
}

#[tauri::command]
async fn download_and_extract_asset(
    asset_name: String,
    destination_dir: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    
    let url = format!("{}/{}", R2_BASE_URL, asset_name);
    let dest_dir = Path::new(&destination_dir);
    
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    println!("[download_and_extract] Downloading: {}", url);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client.get(&url).send().await
        .map_err(|e| format!("Download failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    
    // Stream to temp file
    let temp_path = dest_dir.join(format!(".download_temp.{}", asset_name));
    let mut file = tokio::fs::File::create(&temp_path).await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    let mut downloaded: u64 = 0;
    let mut last_percent: u8 = 0;
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk).await.map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let percent = ((downloaded * 100) / total_size) as u8;
            if percent > last_percent {
                last_percent = percent;
                let _ = app.emit("download-progress", DownloadProgress {
                    asset: asset_name.clone(),
                    downloaded,
                    total: total_size,
                    percent,
                });
            }
        }
    }
    
    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    drop(file);
    
    // Extract zip
    println!("[download_and_extract] Extracting to: {}", dest_dir.display());
    println!("[download_and_extract] Temp file: {}", temp_path.display());
    
    // Verify temp file exists and has content
    let temp_meta = fs::metadata(&temp_path)
        .map_err(|e| format!("Temp file not accessible: {}", e))?;
    println!("[download_and_extract] Temp file size: {} bytes", temp_meta.len());
    
    let file = fs::File::open(&temp_path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip (may be corrupt or wrong format): {}", e))?;
    
    println!("[download_and_extract] Zip contains {} entries", archive.len());
    
    let mut extracted_count = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry {}: {}", i, e))?;
        
        // Use enclosed_name for safe path extraction (prevents path traversal)
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => {
                println!("[download_and_extract] Skipping unsafe entry: {}", file.name());
                continue;
            }
        };
        
        println!("[download_and_extract] Entry {}: {} -> {}", i, file.name(), outpath.display());
        
        if file.is_dir() {
            println!("[download_and_extract] Creating dir: {}", outpath.display());
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create dir {}: {}", outpath.display(), e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir {}: {}", parent.display(), e))?;
            }
            println!("[download_and_extract] Writing file: {} ({} bytes compressed)", outpath.display(), file.compressed_size());
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file {}: {}", outpath.display(), e))?;
            let bytes_written = std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to write file {}: {}", outpath.display(), e))?;
            println!("[download_and_extract] Wrote {} bytes to {}", bytes_written, outpath.display());
            extracted_count += 1;
        }
    }
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_path);
    
    println!("[download_and_extract] Complete: {} files extracted to {}", extracted_count, destination_dir);
    Ok(destination_dir)
}

#[tauri::command]
fn check_asset_exists(path: String) -> bool {
    Path::new(&path).exists()
}

/// Download and setup Quaternius character for 3D projects
#[tauri::command]
async fn setup_3d_character(
    project_path: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    
    let characters_dir = Path::new(&project_path).join("assets").join("characters");
    let character_path = characters_dir.join("character.glb");
    
    // Skip if character already exists
    if character_path.exists() {
        println!("[setup_3d_character] Character already exists: {}", character_path.display());
        return Ok(character_path.to_string_lossy().to_string());
    }
    
    fs::create_dir_all(&characters_dir)
        .map_err(|e| format!("Failed to create characters directory: {}", e))?;
    
    let url = format!("{}/quaternius-character.zip", R2_BASE_URL);
    println!("[setup_3d_character] Downloading: {}", url);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client.get(&url).send().await
        .map_err(|e| format!("Character download failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Character download failed: HTTP {} - Upload quaternius-character.zip to R2", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    
    // Stream to temp file
    let temp_path = characters_dir.join(".download_temp.zip");
    let mut file = tokio::fs::File::create(&temp_path).await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    let mut downloaded: u64 = 0;
    let mut last_percent: u8 = 0;
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk).await.map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let percent = ((downloaded * 100) / total_size) as u8;
            if percent > last_percent {
                last_percent = percent;
                let _ = app.emit("download-progress", DownloadProgress {
                    asset: "quaternius-character".to_string(),
                    downloaded,
                    total: total_size,
                    percent,
                });
            }
        }
    }
    
    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    drop(file);
    
    // Extract zip
    let file = fs::File::open(&temp_path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read character zip: {}", e))?;
    
    archive.extract(&characters_dir)
        .map_err(|e| format!("Failed to extract character: {}", e))?;
    
    // Clean up
    let _ = fs::remove_file(&temp_path);
    
    println!("[setup_3d_character] Complete: {}", character_path.display());
    Ok(character_path.to_string_lossy().to_string())
}

// ============================================================================
// Project Config
// ============================================================================

fn ensure_project_config(project_path: &str) -> Result<(), String> {
    let project_dir = Path::new(project_path);
    
    let project_godot = project_dir.join("project.godot");
    if !project_godot.exists() {
        return Ok(());
    }
    
    let rules_path = project_dir.join("RULES.md");
    if !rules_path.exists() {
        fs::write(&rules_path, GODOT_RULES)
            .map_err(|e| format!("Failed to create RULES.md: {}", e))?;
    }
    
    let claude_md_path = project_dir.join("CLAUDE.md");
    if !claude_md_path.exists() {
        fs::write(&claude_md_path, CLAUDE_MD)
            .map_err(|e| format!("Failed to create CLAUDE.md: {}", e))?;
    }
    
    let cursorrules_path = project_dir.join(".cursorrules");
    if !cursorrules_path.exists() {
        fs::write(&cursorrules_path, GODOT_RULES)
            .map_err(|e| format!("Failed to create .cursorrules: {}", e))?;
    }
    
    for dir in &[
        "scenes", "autoload", "docs",
        "assets/entities/player", "assets/entities/enemies", 
        "assets/characters",
        "assets/ui", "assets/worlds",
        "assets/audio/music", "assets/audio/sfx",
        "assets/visuals/sprites", "assets/visuals/materials",
        "src/core", "src/systems", "src/components", "src/states", "src/utilities"
    ] {
        let dir_path = project_dir.join(dir);
        if !dir_path.exists() {
            let _ = fs::create_dir_all(&dir_path);
        }
    }
    
    Ok(())
}

// ============================================================================
// File Operations
// ============================================================================

fn build_file_tree(path: &Path, depth: usize) -> Vec<FileEntry> {
    if depth > 3 {
        return vec![];
    }

    let mut entries: Vec<FileEntry> = vec![];

    if let Ok(read_dir) = fs::read_dir(path) {
        let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
        items.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in items {
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.')
                || name == "node_modules"
                || name == ".godot"
                || name == "target"
            {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let children = if is_dir {
                Some(build_file_tree(&entry_path, depth + 1))
            } else {
                None
            };

            entries.push(FileEntry {
                name,
                path: entry_path.to_string_lossy().to_string(),
                is_dir,
                children,
            });
        }
    }

    entries
}

#[tauri::command]
fn list_files(path: String) -> Result<Vec<FileEntry>, String> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }
    
    let _ = ensure_project_config(&path.to_string_lossy());
    
    Ok(build_file_tree(path, 0))
}

#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
fn write_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
fn delete_file(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if p.is_dir() {
        fs::remove_dir_all(p).map_err(|e| format!("Failed to delete directory '{}': {}", path, e))
    } else {
        fs::remove_file(p).map_err(|e| format!("Failed to delete file '{}': {}", path, e))
    }
}

// ============================================================================
// Godot Detection - Check common install locations
// ============================================================================

/// Read a key from .env.local file - checks multiple locations
fn read_env_file_key(project_path: &str, key: &str) -> Option<String> {
    let locations = [
        // Game project .env.local
        Path::new(project_path).join(".env.local"),
        // App root .env.local (dev mode)
        PathBuf::from("c:/My Apps/colorwave/.env.local"),
        // Current directory
        PathBuf::from(".env.local"),
        // Parent of project (common layout)
        Path::new(project_path).parent().map(|p| p.join(".env.local")).unwrap_or_default(),
    ];
    
    for env_path in &locations {
        if let Ok(content) = fs::read_to_string(env_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                if let Some(value) = line.strip_prefix(&format!("{}=", key)) {
                    println!("[ENV] Found {} in {:?}", key, env_path);
                    return Some(value.trim().to_string());
                }
            }
        }
    }
    println!("[ENV] Key {} not found in any .env.local", key);
    None
}

fn find_godot_path() -> Option<String> {
    // First try PATH
    let path_names = if cfg!(windows) {
        vec!["godot", "godot.exe"]
    } else if cfg!(target_os = "macos") {
        vec!["godot", "godot4"]
    } else {
        vec!["godot", "godot4", "Godot"]
    };

    for name in &path_names {
        let result = if cfg!(windows) {
            silent_cmd("cmd", &["/C", "where", name])
        } else {
            Command::new("which").arg(name).output()
        };
        
        if let Ok(output) = result {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    let first_line = path.lines().next().unwrap_or(&path);
                    if Path::new(first_line).exists() {
                        return Some(first_line.to_string());
                    }
                }
            }
        }
    }

    // Check common installation locations
    #[cfg(windows)]
    {
        let common_paths = vec![
            // Standard install locations
            r"C:\Program Files\Godot\Godot.exe",
            r"C:\Program Files\Godot\Godot_v4.3-stable_win64.exe",
            r"C:\Program Files\Godot\Godot_v4.2-stable_win64.exe",
            r"C:\Program Files (x86)\Godot\Godot.exe",
            // Steam
            r"C:\Program Files (x86)\Steam\steamapps\common\Godot Engine\godot.windows.editor.x86_64.exe",
            r"C:\Program Files\Steam\steamapps\common\Godot Engine\godot.windows.editor.x86_64.exe",
        ];
        
        for path in common_paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        
        // Check user-specific locations
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            let user_paths = vec![
                format!(r"{}\Godot\Godot.exe", localappdata),
                format!(r"{}\Programs\Godot\Godot.exe", localappdata),
            ];
            for path in user_paths {
                if Path::new(&path).exists() {
                    return Some(path);
                }
            }
        }
        
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let user_paths = vec![
                format!(r"{}\scoop\apps\godot\current\godot.exe", userprofile),
                format!(r"{}\scoop\apps\godot\current\Godot.exe", userprofile),
                format!(r"{}\.local\bin\godot.exe", userprofile),
            ];
            for path in user_paths {
                if Path::new(&path).exists() {
                    return Some(path);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mac_paths = vec![
            "/Applications/Godot.app/Contents/MacOS/Godot",
            "/Applications/Godot 4.app/Contents/MacOS/Godot",
        ];
        for path in mac_paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let linux_paths = vec![
            "/usr/bin/godot",
            "/usr/bin/godot4",
            "/usr/local/bin/godot",
            "/snap/bin/godot",
        ];
        for path in linux_paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        
        if let Ok(home) = std::env::var("HOME") {
            let home_path = format!("{}/.local/bin/godot", home);
            if Path::new(&home_path).exists() {
                return Some(home_path);
            }
        }
    }

    None
}

#[tauri::command]
fn detect_godot(state: tauri::State<AppState>) -> Result<String, String> {
    // First check saved path
    let settings = state.settings.lock().unwrap();
    if let Some(path) = &settings.godot_path {
        if !path.is_empty() && Path::new(path).exists() {
            return Ok(path.clone());
        }
    }
    drop(settings);

    // Search for Godot
    if let Some(path) = find_godot_path() {
        // Auto-save the found path
        let mut settings = state.settings.lock().unwrap();
        settings.godot_path = Some(path.clone());
        drop(settings);
        
        // Also persist to disk
        let _ = save_settings_to_disk(&AppSettings {
            godot_path: Some(path.clone()),
            ..Default::default()
        });
        
        Ok(path)
    } else {
        Err("Godot not found".to_string())
    }
}

#[tauri::command]
fn install_godot() -> Result<String, String> {
    // Try winget first on Windows
    #[cfg(windows)]
    {
        let result = Command::new("cmd")
            .args(["/C", "winget", "install", "--id", "GodotEngine.GodotEngine", "-e", "--accept-package-agreements", "--accept-source-agreements"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        
        if let Ok(output) = result {
            if output.status.success() {
                return Ok("Installing Godot via winget... Please wait and then click refresh.".to_string());
            }
        }
        
        // Fallback to opening download page
        open::that("https://godotengine.org/download/windows/")
            .map_err(|e| format!("Failed to open download page: {}", e))?;
        return Ok("Opening Godot download page...".to_string());
    }
    
    #[cfg(target_os = "macos")]
    {
        let result = Command::new("brew")
            .args(["install", "--cask", "godot"])
            .output();
        
        if let Ok(output) = result {
            if output.status.success() {
                return Ok("Installing Godot via Homebrew...".to_string());
            }
        }
        
        open::that("https://godotengine.org/download/macos/")
            .map_err(|e| format!("Failed to open download page: {}", e))?;
        return Ok("Opening Godot download page...".to_string());
    }
    
    #[cfg(target_os = "linux")]
    {
        open::that("https://godotengine.org/download/linux/")
            .map_err(|e| format!("Failed to open download page: {}", e))?;
        return Ok("Opening Godot download page...".to_string());
    }
    
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        open::that("https://godotengine.org/download/")
            .map_err(|e| format!("Failed to open download page: {}", e))?;
        Ok("Opening Godot download page...".to_string())
    }
}

// ============================================================================
// Agent Detection (internal)
// ============================================================================

fn detect_goose() -> bool {
    let result = if cfg!(windows) {
        silent_cmd("cmd", &["/C", "where", "goose"])
    } else {
        Command::new("which").arg("goose").output()
    };
    result.map(|o| o.status.success()).unwrap_or(false)
}

// ============================================================================
// Beads Task Tracking Integration
// ============================================================================

#[tauri::command]
fn detect_beads() -> bool {
    let result = if cfg!(windows) {
        silent_cmd("cmd", &["/C", "where", "bd"])
    } else {
        Command::new("which").arg("bd").output()
    };
    result.map(|o| o.status.success()).unwrap_or(false)
}

#[tauri::command]
async fn install_beads() -> Result<String, String> {
    // Use go install method (requires Go)
    let result = if cfg!(windows) {
        silent_cmd("cmd", &["/C", "go", "install", "github.com/steveyegge/beads/cmd/bd@latest"])
    } else {
        Command::new("go")
            .args(["install", "github.com/steveyegge/beads/cmd/bd@latest"])
            .output()
    };

    match result {
        Ok(output) if output.status.success() => {
            Ok("Beads installed successfully".to_string())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Try npm as fallback
            let npm_result = if cfg!(windows) {
                silent_cmd("cmd", &["/C", "npm", "install", "-g", "@beads/bd"])
            } else {
                Command::new("npm").args(["install", "-g", "@beads/bd"]).output()
            };
            
            match npm_result {
                Ok(o) if o.status.success() => Ok("Beads installed via npm".to_string()),
                _ => Err(format!("Failed to install Beads: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to run installer: {}", e))
    }
}

#[tauri::command]
fn init_beads(project_path: String) -> Result<String, String> {
    if !detect_beads() {
        return Err("Beads (bd) is not installed".to_string());
    }

    let project_dir = Path::new(&project_path);
    if !project_dir.exists() {
        return Err("Project path does not exist".to_string());
    }

    // Check if already initialized
    let beads_dir = project_dir.join(".beads");
    if beads_dir.exists() {
        return Ok("Beads already initialized".to_string());
    }

    // Initialize Beads in the project
    let result = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "bd", "init", "--quiet"])
            .current_dir(project_dir)
            .output()
    } else {
        Command::new("bd")
            .args(["init", "--quiet"])
            .current_dir(project_dir)
            .output()
    };

    match result {
        Ok(output) if output.status.success() => {
            Ok("Beads initialized for task tracking".to_string())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to initialize Beads: {}", stderr))
        }
        Err(e) => Err(format!("Failed to run bd init: {}", e))
    }
}

#[tauri::command]
fn get_beads_context(project_path: String) -> Result<String, String> {
    if !detect_beads() {
        return Err("Beads not installed".to_string());
    }

    let project_dir = Path::new(&project_path);
    if !project_dir.join(".beads").exists() {
        return Err("Beads not initialized in this project".to_string());
    }

    // Get ready tasks for agent context
    let result = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "bd", "prime"])
            .current_dir(project_dir)
            .output()
    } else {
        Command::new("bd")
            .args(["prime"])
            .current_dir(project_dir)
            .output()
    };

    match result {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        Ok(_) => Ok("".to_string()), // No tasks yet
        Err(e) => Err(format!("Failed to get Beads context: {}", e))
    }
}

// ============================================================================
// Godot MCP Setup
// ============================================================================

#[tauri::command]
fn install_godot_mcp() -> Result<String, String> {
    let result = if cfg!(windows) {
        silent_cmd("cmd", &["/C", "npm", "install", "-g", "godot-mcp"])
    } else {
        Command::new("npm")
            .args(["install", "-g", "godot-mcp"])
            .output()
    };

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok("Godot MCP installed successfully".to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to install Godot MCP: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to run npm: {}", e)),
    }
}

#[tauri::command]
fn detect_godot_mcp() -> bool {
    let result = if cfg!(windows) {
        silent_cmd("cmd", &["/C", "npm", "list", "-g", "godot-mcp"])
    } else {
        Command::new("npm")
            .args(["list", "-g", "godot-mcp"])
            .output()
    };
    
    result.map(|o| o.status.success()).unwrap_or(false)
}

#[tauri::command]
fn setup_godot_mcp_config() -> Result<(), String> {
    // Configure Goose's MCP settings for Godot and Beads
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let goose_config_dir = home.join(".config").join("goose");
    let goose_profiles_path = goose_config_dir.join("profiles.yaml");
    
    // Create .config/goose directory if it doesn't exist
    fs::create_dir_all(&goose_config_dir).ok();
    
    // Read existing profiles or create new
    let mut profiles: serde_yaml::Value = if goose_profiles_path.exists() {
        let content = fs::read_to_string(&goose_profiles_path).unwrap_or_default();
        serde_yaml::from_str(&content).unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()))
    } else {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    };
    
    // Add extensions to default profile
    if let serde_yaml::Value::Mapping(ref mut map) = profiles {
        let default_profile = map
            .entry(serde_yaml::Value::String("default".to_string()))
            .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
        
        if let serde_yaml::Value::Mapping(ref mut profile_map) = default_profile {
            let extensions = profile_map
                .entry(serde_yaml::Value::String("extensions".to_string()))
                .or_insert(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
            
            if let serde_yaml::Value::Mapping(ref mut ext_map) = extensions {
                // Add Godot MCP
                let mut godot_config = serde_yaml::Mapping::new();
                godot_config.insert(
                    serde_yaml::Value::String("type".to_string()),
                    serde_yaml::Value::String("stdio".to_string())
                );
                godot_config.insert(
                    serde_yaml::Value::String("cmd".to_string()),
                    serde_yaml::Value::String("npx".to_string())
                );
                let mut godot_args = serde_yaml::Sequence::new();
                godot_args.push(serde_yaml::Value::String("-y".to_string()));
                godot_args.push(serde_yaml::Value::String("godot-mcp".to_string()));
                godot_config.insert(
                    serde_yaml::Value::String("args".to_string()),
                    serde_yaml::Value::Sequence(godot_args)
                );
                ext_map.insert(
                    serde_yaml::Value::String("godot".to_string()),
                    serde_yaml::Value::Mapping(godot_config)
                );
                
                // Add Beads MCP for task tracking
                let mut beads_config = serde_yaml::Mapping::new();
                beads_config.insert(
                    serde_yaml::Value::String("type".to_string()),
                    serde_yaml::Value::String("stdio".to_string())
                );
                beads_config.insert(
                    serde_yaml::Value::String("cmd".to_string()),
                    serde_yaml::Value::String("beads-mcp".to_string())
                );
                ext_map.insert(
                    serde_yaml::Value::String("beads".to_string()),
                    serde_yaml::Value::Mapping(beads_config)
                );
            }
        }
    }
    
    // Write back
    let yaml_str = serde_yaml::to_string(&profiles)
        .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
    fs::write(&goose_profiles_path, yaml_str)
        .map_err(|e| format!("Failed to write Goose profiles: {}", e))?;
    
    Ok(())
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

// ============================================================================
// Project Templates
// ============================================================================

#[tauri::command]
fn initialize_godot_project(
    project_path: String,
    dimension: String,
    template: String,
) -> Result<(), String> {
    let path = Path::new(&project_path);
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("game");
    
    // Create professional folder structure
    let dirs = [
        "scenes",
        "autoload",
        "assets/entities/player",
        "assets/entities/enemies",
        "assets/ui",
        "assets/worlds",
        "assets/audio/music",
        "assets/audio/sfx",
        "assets/visuals/sprites",
        "assets/visuals/materials",
        "assets/characters",
        "src/core",
        "src/systems",
        "src/components",
        "src/states",
        "src/utilities",
        "docs",
    ];
    for dir in &dirs {
        fs::create_dir_all(path.join(dir)).ok();
    }
    
    // Generate project.godot with all autoloads
    let project_godot = generate_project_godot(name, &dimension);
    fs::write(path.join("project.godot"), project_godot)
        .map_err(|e| format!("Failed to write project.godot: {}", e))?;
    
    // Write core autoloads (Signal Bus Pattern)
    fs::write(path.join("autoload/event_bus.gd"), EVENT_BUS_GD)
        .map_err(|e| format!("Failed to write event_bus.gd: {}", e))?;
    fs::write(path.join("autoload/game_state.gd"), GAME_STATE_GD)
        .map_err(|e| format!("Failed to write game_state.gd: {}", e))?;
    fs::write(path.join("autoload/ai_controller.gd"), AI_CONTROLLER_GD)
        .map_err(|e| format!("Failed to write ai_controller.gd: {}", e))?;
    
    // Write reusable components
    fs::write(path.join("src/components/health_component.gd"), HEALTH_COMPONENT_GD)
        .map_err(|e| format!("Failed to write health_component.gd: {}", e))?;
    fs::write(path.join("src/components/movement_component_2d.gd"), MOVEMENT_COMPONENT_2D_GD)
        .map_err(|e| format!("Failed to write movement_component_2d.gd: {}", e))?;
    
    // Write FSM components (for 3D projects)
    fs::write(path.join("src/components/state_machine.gd"), STATE_MACHINE_GD)
        .map_err(|e| format!("Failed to write state_machine.gd: {}", e))?;
    fs::write(path.join("src/components/state.gd"), STATE_GD)
        .map_err(|e| format!("Failed to write state.gd: {}", e))?;
    // Skip custom camera/locomotion for third-person (uses AMSG addon)
    if template != "third-person" {
        fs::write(path.join("src/components/camera_rig_3d.gd"), CAMERA_RIG_3D_GD)
            .map_err(|e| format!("Failed to write camera_rig_3d.gd: {}", e))?;
        fs::write(path.join("src/components/locomotion_controller.gd"), LOCOMOTION_CONTROLLER_GD)
            .map_err(|e| format!("Failed to write locomotion_controller.gd: {}", e))?;
    }
    fs::write(path.join("src/components/mixamo_retargeter.gd"), MIXAMO_RETARGETER_GD)
        .map_err(|e| format!("Failed to write mixamo_retargeter.gd: {}", e))?;
    
    // Write locomotion states
    fs::write(path.join("src/states/idle_state.gd"), IDLE_STATE_GD)
        .map_err(|e| format!("Failed to write idle_state.gd: {}", e))?;
    fs::write(path.join("src/states/move_state.gd"), MOVE_STATE_GD)
        .map_err(|e| format!("Failed to write move_state.gd: {}", e))?;
    fs::write(path.join("src/states/air_state.gd"), AIR_STATE_GD)
        .map_err(|e| format!("Failed to write air_state.gd: {}", e))?;
    
    // Write animation setup guide
    fs::write(path.join("docs/ANIMATION_SETUP.md"), ANIMATION_SETUP_GUIDE)
        .map_err(|e| format!("Failed to write ANIMATION_SETUP.md: {}", e))?;
    
    // Generate main scene based on template
    let (main_scene, main_script) = generate_template_files(&dimension, &template);
    
    fs::write(path.join("scenes/main.tscn"), main_scene)
        .map_err(|e| format!("Failed to write main scene: {}", e))?;
    
    fs::write(path.join("assets/entities/player/player.gd"), main_script)
        .map_err(|e| format!("Failed to write player script: {}", e))?;
    
    // Create RULES.md for AI agents
    let _ = ensure_project_config(&project_path);
    
    Ok(())
}

#[tauri::command]
fn create_project_from_template(
    name: String,
    parent_path: String,
    dimension: String,
    template: String,
) -> Result<String, String> {
    let project_path = Path::new(&parent_path).join(&name);
    
    // Create project directory
    fs::create_dir_all(&project_path)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;
    
    // Create professional folder structure
    let dirs = [
        "scenes",
        "autoload",
        "assets/entities/player",
        "assets/entities/enemies",
        "assets/ui",
        "assets/worlds",
        "assets/audio/music",
        "assets/audio/sfx",
        "assets/visuals/sprites",
        "assets/visuals/materials",
        "assets/characters",
        "src/core",
        "src/systems",
        "src/components",
        "src/states",
        "src/utilities",
        "docs",
    ];
    for dir in &dirs {
        fs::create_dir_all(project_path.join(dir)).ok();
    }
    
    // Generate project.godot with all autoloads
    let project_godot = generate_project_godot(&name, &dimension);
    fs::write(project_path.join("project.godot"), project_godot)
        .map_err(|e| format!("Failed to write project.godot: {}", e))?;
    
    // Write core autoloads (Signal Bus Pattern)
    fs::write(project_path.join("autoload/event_bus.gd"), EVENT_BUS_GD)
        .map_err(|e| format!("Failed to write event_bus.gd: {}", e))?;
    fs::write(project_path.join("autoload/game_state.gd"), GAME_STATE_GD)
        .map_err(|e| format!("Failed to write game_state.gd: {}", e))?;
    fs::write(project_path.join("autoload/ai_controller.gd"), AI_CONTROLLER_GD)
        .map_err(|e| format!("Failed to write ai_controller.gd: {}", e))?;
    
    // Write reusable components
    fs::write(project_path.join("src/components/health_component.gd"), HEALTH_COMPONENT_GD)
        .map_err(|e| format!("Failed to write health_component.gd: {}", e))?;
    fs::write(project_path.join("src/components/movement_component_2d.gd"), MOVEMENT_COMPONENT_2D_GD)
        .map_err(|e| format!("Failed to write movement_component_2d.gd: {}", e))?;
    
    // Write FSM components (for 3D projects)
    fs::write(project_path.join("src/components/state_machine.gd"), STATE_MACHINE_GD)
        .map_err(|e| format!("Failed to write state_machine.gd: {}", e))?;
    fs::write(project_path.join("src/components/state.gd"), STATE_GD)
        .map_err(|e| format!("Failed to write state.gd: {}", e))?;
    // Skip custom camera/locomotion for third-person (uses AMSG addon)
    if template != "third-person" {
        fs::write(project_path.join("src/components/camera_rig_3d.gd"), CAMERA_RIG_3D_GD)
            .map_err(|e| format!("Failed to write camera_rig_3d.gd: {}", e))?;
        fs::write(project_path.join("src/components/locomotion_controller.gd"), LOCOMOTION_CONTROLLER_GD)
            .map_err(|e| format!("Failed to write locomotion_controller.gd: {}", e))?;
    }
    fs::write(project_path.join("src/components/mixamo_retargeter.gd"), MIXAMO_RETARGETER_GD)
        .map_err(|e| format!("Failed to write mixamo_retargeter.gd: {}", e))?;
    
    // Write locomotion states
    fs::write(project_path.join("src/states/idle_state.gd"), IDLE_STATE_GD)
        .map_err(|e| format!("Failed to write idle_state.gd: {}", e))?;
    fs::write(project_path.join("src/states/move_state.gd"), MOVE_STATE_GD)
        .map_err(|e| format!("Failed to write move_state.gd: {}", e))?;
    fs::write(project_path.join("src/states/air_state.gd"), AIR_STATE_GD)
        .map_err(|e| format!("Failed to write air_state.gd: {}", e))?;
    
    // Write animation setup guide
    fs::write(project_path.join("docs/ANIMATION_SETUP.md"), ANIMATION_SETUP_GUIDE)
        .map_err(|e| format!("Failed to write ANIMATION_SETUP.md: {}", e))?;
    
    // Generate main scene based on template
    let (main_scene, main_script) = generate_template_files(&dimension, &template);
    
    fs::write(project_path.join("scenes/main.tscn"), main_scene)
        .map_err(|e| format!("Failed to write main scene: {}", e))?;
    
    fs::write(project_path.join("assets/entities/player/player.gd"), main_script)
        .map_err(|e| format!("Failed to write player script: {}", e))?;
    
    // Store template info for auto-sync on future exports
    let kobold_dir = project_path.join(".tav");
    fs::create_dir_all(&kobold_dir).ok();
    let template_info = serde_json::json!({
        "template": &template,
        "dimension": &dimension,
        "version": TEMPLATE_VERSION
    });
    fs::write(
        kobold_dir.join("template_info.json"),
        serde_json::to_string_pretty(&template_info).unwrap()
    ).ok();
    
    // Create RULES.md for AI agents
    let _ = ensure_project_config(project_path.to_string_lossy().as_ref());
    
    Ok(project_path.to_string_lossy().to_string())
}

fn generate_project_godot(name: &str, dimension: &str) -> String {
    let renderer = if dimension == "3d" { "forward_plus" } else { "gl_compatibility" };
    format!(r#"; Engine configuration file.
; Generated by Kobold - Professional Godot Architecture

config_version=5

[application]

config/name="{}"
run/main_scene="res://scenes/main.tscn"
config/features=PackedStringArray("4.3", "{}")

[autoload]

EventBus="*res://autoload/event_bus.gd"
GameState="*res://autoload/game_state.gd"
AIController="*res://autoload/ai_controller.gd"

[input]

move_left={{
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":65,"key_label":0,"unicode":97,"location":0,"echo":false,"script":null), Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":4194319,"key_label":0,"unicode":0,"location":0,"echo":false,"script":null)]
}}
move_right={{
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":68,"key_label":0,"unicode":100,"location":0,"echo":false,"script":null), Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":4194321,"key_label":0,"unicode":0,"location":0,"echo":false,"script":null)]
}}
move_up={{
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":87,"key_label":0,"unicode":119,"location":0,"echo":false,"script":null), Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":4194320,"key_label":0,"unicode":0,"location":0,"echo":false,"script":null)]
}}
move_down={{
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":83,"key_label":0,"unicode":115,"location":0,"echo":false,"script":null), Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":4194322,"key_label":0,"unicode":0,"location":0,"echo":false,"script":null)]
}}
jump={{
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":32,"key_label":0,"unicode":32,"location":0,"echo":false,"script":null)]
}}
attack={{
"deadzone": 0.5,
"events": [Object(InputEventMouseButton,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"button_mask":1,"position":Vector2(0, 0),"global_position":Vector2(0, 0),"factor":1.0,"button_index":1,"canceled":false,"pressed":true,"double_click":false,"script":null)]
}}
interact={{
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":69,"key_label":0,"unicode":101,"location":0,"echo":false,"script":null)]
}}
sprint={{
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":4194325,"key_label":0,"unicode":0,"location":0,"echo":false,"script":null)]
}}

[rendering]

renderer/rendering_method="{}"
"#, name, if dimension == "3d" { "3D" } else { "2D" }, renderer)
}

// Embed template files from templates folder at compile time
const THIRD_PERSON_SCENE: &str = include_str!("../../templates/third-person-3d/scene.tscn");
const THIRD_PERSON_PLAYER: &str = include_str!("../../templates/third-person-3d/player.gd");
// Note: Third-person uses AMSG addon (packages/amsg) for locomotion, camera, and states

fn generate_template_files(dimension: &str, template: &str) -> (String, String) {
    match (dimension, template) {
        ("3d", "third-person") => (
            THIRD_PERSON_SCENE.to_string(),
            THIRD_PERSON_PLAYER.to_string()
        ),
        ("2d", "platformer") => (
            r#"[gd_scene load_steps=3 format=3]

[ext_resource type="Script" path="res://assets/entities/player/player.gd" id="1"]
[ext_resource type="Script" path="res://src/components/health_component.gd" id="2"]

[node name="Main" type="Node2D"]

[node name="Player" type="CharacterBody2D" parent="."]
position = Vector2(576, 300)
script = ExtResource("1")

[node name="CollisionShape2D" type="CollisionShape2D" parent="Player"]

[node name="Sprite2D" type="Sprite2D" parent="Player"]

[node name="Camera2D" type="Camera2D" parent="Player"]

[node name="HealthComponent" type="Node" parent="Player"]
script = ExtResource("2")
"#.to_string(),
            r#"extends CharacterBody2D
class_name Player
## 2D Platformer Player - Uses Entity-Component Pattern
## HealthComponent attached as child handles damage/death

@export var speed: float = 300.0
@export var jump_force: float = -400.0

var gravity: float = ProjectSettings.get_setting("physics/2d/default_gravity")
@onready var health_comp: HealthComponent = $HealthComponent

func _ready() -> void:
	# Connect to component signals
	if health_comp:
		health_comp.died.connect(_on_died)
		health_comp.health_changed.connect(_on_health_changed)
	EventBus.player_spawned.emit(self)
	print("Player ready! Use WASD/Arrows + Space to jump")

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y += gravity * delta
	
	if Input.is_action_just_pressed("jump") and is_on_floor():
		velocity.y = jump_force
	
	var direction := Input.get_axis("move_left", "move_right")
	velocity.x = direction * speed if direction else move_toward(velocity.x, 0, speed)
	
	move_and_slide()

func take_damage(amount: int) -> void:
	if health_comp:
		health_comp.take_damage(amount)

func _on_health_changed(current: int, maximum: int) -> void:
	EventBus.health_changed.emit(current, maximum)

func _on_died() -> void:
	EventBus.player_died.emit()
	# Add death animation/respawn logic here
	print("Player died!")
"#.to_string()
        ),
        ("2d", "top-down") => (
            r#"[gd_scene load_steps=3 format=3]

[ext_resource type="Script" path="res://assets/entities/player/player.gd" id="1"]
[ext_resource type="Script" path="res://src/components/health_component.gd" id="2"]

[node name="Main" type="Node2D"]

[node name="Player" type="CharacterBody2D" parent="."]
position = Vector2(576, 324)
script = ExtResource("1")

[node name="CollisionShape2D" type="CollisionShape2D" parent="Player"]

[node name="Sprite2D" type="Sprite2D" parent="Player"]

[node name="Camera2D" type="Camera2D" parent="Player"]

[node name="HealthComponent" type="Node" parent="Player"]
script = ExtResource("2")
"#.to_string(),
            r#"extends CharacterBody2D
class_name Player
## 2D Top-Down Player - Uses Entity-Component Pattern
## HealthComponent attached as child handles damage/death

@export var speed: float = 200.0

@onready var health_comp: HealthComponent = $HealthComponent

func _ready() -> void:
	if health_comp:
		health_comp.died.connect(_on_died)
		health_comp.health_changed.connect(_on_health_changed)
	EventBus.player_spawned.emit(self)
	print("Top-down ready! Use WASD/Arrows to move, E to interact")

func _physics_process(_delta: float) -> void:
	var input_dir := Vector2(
		Input.get_axis("move_left", "move_right"),
		Input.get_axis("move_up", "move_down")
	)
	velocity = input_dir.normalized() * speed
	move_and_slide()

func take_damage(amount: int) -> void:
	if health_comp:
		health_comp.take_damage(amount)

func interact() -> void:
	# Override for interaction logic
	print("Interact pressed!")

func _on_health_changed(current: int, maximum: int) -> void:
	EventBus.health_changed.emit(current, maximum)

func _on_died() -> void:
	EventBus.player_died.emit()
	print("Player died!")
"#.to_string()
        ),
        ("3d", "first-person") => (
            r#"[gd_scene load_steps=6 format=3]

[ext_resource type="Script" path="res://assets/entities/player/player.gd" id="1"]
[ext_resource type="Script" path="res://src/components/health_component.gd" id="2"]

[sub_resource type="CapsuleShape3D" id="1"]

[sub_resource type="BoxMesh" id="2"]
size = Vector3(20, 0.1, 20)

[sub_resource type="BoxShape3D" id="3"]
size = Vector3(20, 0.1, 20)

[node name="Main" type="Node3D"]

[node name="Player" type="CharacterBody3D" parent="."]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0)
script = ExtResource("1")

[node name="CollisionShape3D" type="CollisionShape3D" parent="Player"]
shape = SubResource("1")

[node name="Camera3D" type="Camera3D" parent="Player"]
transform = Transform3D(1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0.5, 0)

[node name="HealthComponent" type="Node" parent="Player"]
script = ExtResource("2")

[node name="DirectionalLight3D" type="DirectionalLight3D" parent="."]
transform = Transform3D(1, 0, 0, 0, 0.707, 0.707, 0, -0.707, 0.707, 0, 10, 0)

[node name="Floor" type="StaticBody3D" parent="."]

[node name="FloorMesh" type="MeshInstance3D" parent="Floor"]
mesh = SubResource("2")

[node name="FloorCollision" type="CollisionShape3D" parent="Floor"]
shape = SubResource("3")
"#.to_string(),
            r#"extends CharacterBody3D
class_name Player
## First Person Player - Uses Entity-Component Pattern

@export var speed: float = 5.0
@export var mouse_sensitivity: float = 0.002

var gravity: float = ProjectSettings.get_setting("physics/3d/default_gravity")
@onready var camera: Camera3D = $Camera3D
@onready var health_comp: HealthComponent = $HealthComponent

func _ready() -> void:
	# Don't capture mouse in _ready - wait for click (required for web)
	if health_comp:
		health_comp.died.connect(_on_died)
		health_comp.health_changed.connect(_on_health_changed)
	EventBus.player_spawned.emit(self)
	print("Click to capture mouse, WASD to move, ESC to release")

func _input(event: InputEvent) -> void:
	# Capture mouse on click (web-compatible)
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		if Input.mouse_mode != Input.MOUSE_MODE_CAPTURED:
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		rotate_y(-event.relative.x * mouse_sensitivity)
		camera.rotate_x(-event.relative.y * mouse_sensitivity)
		camera.rotation.x = clamp(camera.rotation.x, -PI/2, PI/2)
	
	if event.is_action_pressed("ui_cancel"):
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y -= gravity * delta
	
	var input_dir := Input.get_vector("move_left", "move_right", "move_up", "move_down")
	var direction := (transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	
	velocity.x = direction.x * speed if direction else move_toward(velocity.x, 0, speed)
	velocity.z = direction.z * speed if direction else move_toward(velocity.z, 0, speed)
	
	move_and_slide()

func take_damage(amount: int) -> void:
	if health_comp:
		health_comp.take_damage(amount)

func _on_health_changed(current: int, maximum: int) -> void:
	EventBus.health_changed.emit(current, maximum)

func _on_died() -> void:
	EventBus.player_died.emit()
"#.to_string()
        ),
        ("2d", "puzzle") | ("3d", "puzzle") => {
            let node_type = if dimension == "3d" { "Node3D" } else { "Node2D" };
            (
                format!(r#"[gd_scene load_steps=2 format=3]

[ext_resource type="Script" path="res://assets/entities/player/player.gd" id="1"]

[node name="Main" type="{}"]
script = ExtResource("1")

[node name="UI" type="CanvasLayer" parent="."]

[node name="ScoreLabel" type="Label" parent="UI"]
offset_right = 200.0
offset_bottom = 40.0
text = "Score: 0"
"#, node_type),
                r#"extends Node
## Puzzle Game - Uses EventBus for score tracking

func _ready() -> void:
	# Listen to score changes from GameState
	EventBus.score_changed.connect(_on_score_changed)
	EventBus.coin_collected.connect(_on_coin_collected)
	_update_display()
	print("Puzzle ready! Space to add points, uses EventBus + GameState")

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_accept"):
		GameState.add_score(10)
		EventBus.coin_collected.emit(10)

func _on_score_changed(new_score: int) -> void:
	_update_display()

func _on_coin_collected(value: int) -> void:
	print("Collected: %d points!" % value)

func _update_display() -> void:
	$UI/ScoreLabel.text = "Score: %d" % GameState.score
"#.to_string()
            )
        },
        _ => {
            // Empty project with professional architecture
            let node_type = if dimension == "3d" { "Node3D" } else { "Node2D" };
            (
                format!(r#"[gd_scene load_steps=2 format=3]

[ext_resource type="Script" path="res://assets/entities/player/player.gd" id="1"]

[node name="Main" type="{}"]
script = ExtResource("1")
"#, node_type),
                format!(r#"extends {}
## Empty {} Project - Professional Architecture Ready
## EventBus, GameState, and Components are pre-configured

func _ready() -> void:
	# EventBus is ready for cross-system communication
	# GameState persists data across scenes
	# Components in src/components/ are ready to use
	
	# Example: Listen to game events
	EventBus.player_spawned.connect(func(p): print("Player spawned: ", p))
	EventBus.level_completed.connect(func(): print("Level done!"))
	
	print("Hello from Kobold! Architecture ready.")
	print("- EventBus: Signal bus for decoupled communication")
	print("- GameState: Persistent cross-scene data")
	print("- Components: HealthComponent, MovementComponent2D")
"#, node_type, if dimension == "3d" { "3D" } else { "2D" })
            )
        }
    }
}

// ============================================================================
// Godot Operations
// ============================================================================

#[tauri::command]
fn run_godot(project_path: String, state: tauri::State<AppState>) -> Result<String, String> {
    let settings = state.settings.lock().unwrap();
    let godot_cmd = settings
        .godot_path
        .clone()
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .or_else(|| find_godot_path())
        .ok_or("Godot not found. Please install Godot first.")?;

    // Run Godot with the project (opens game window)
    let output = Command::new(&godot_cmd)
        .args(["--path", &project_path])
        .current_dir(&project_path)
        .output()
        .map_err(|e| format!("Failed to run Godot ({}): {}", godot_cmd, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(format!("{}\n{}", stdout, stderr))
}

fn get_godot_version(godot_cmd: &str) -> Result<String, String> {
    let output = Command::new(godot_cmd)
        .args(["--version"])
        .output()
        .map_err(|e| format!("Failed to get Godot version: {}", e))?;
    
    let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // Version looks like "4.3.stable.official.77dcf97d8"
    // We need "4.3.stable" for the templates folder
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() >= 3 {
        Ok(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
    } else {
        Err(format!("Unexpected version format: {}", version_str))
    }
}

fn get_export_templates_path(version: &str) -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        dirs::data_dir().map(|d| d.join("Godot").join("export_templates").join(version))
    }
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().map(|d| d.join("Godot").join("export_templates").join(version))
    }
    #[cfg(target_os = "linux")]
    {
        dirs::data_dir().map(|d| d.join("godot").join("export_templates").join(version))
    }
}

#[derive(Serialize)]
struct SetupStatus {
    #[serde(rename = "godotInstalled")]
    godot_installed: bool,
    #[serde(rename = "godotPath")]
    godot_path: Option<String>,
    #[serde(rename = "godotVersion")]
    godot_version: Option<String>,
    #[serde(rename = "templatesInstalled")]
    templates_installed: bool,
}

#[tauri::command]
fn check_setup_status(state: tauri::State<AppState>) -> SetupStatus {
    let settings = state.settings.lock().unwrap().clone();
    let godot_path = settings
        .godot_path
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .or_else(|| find_godot_path());
    
    let (godot_version, templates_installed) = if let Some(ref path) = godot_path {
        match get_godot_version(path) {
            Ok(version) => {
                let templates = check_web_templates_installed(&version);
                (Some(version), templates)
            }
            Err(_) => (None, false)
        }
    } else {
        (None, false)
    };
    
    SetupStatus {
        godot_installed: godot_path.is_some(),
        godot_path,
        godot_version,
        templates_installed,
    }
}

fn check_web_templates_installed(version: &str) -> bool {
    if let Some(path) = get_export_templates_path(version) {
        // Godot 4.x uses different naming conventions for web templates
        let patterns = [
            "web_release.zip",
            "web_debug.zip",
            "web_release.wasm",
            "web_debug.wasm",
            "godot.web.template_release.wasm32.zip",
            "godot.web.template_debug.wasm32.zip",
        ];
        
        for pattern in patterns {
            if path.join(pattern).exists() {
                println!("[check_web_templates] Found: {:?}", path.join(pattern));
                return true;
            }
        }
        
        // List what files ARE there for debugging
        if path.exists() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                let files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .take(10)
                    .collect();
                println!("[check_web_templates] Path exists, files: {:?}", files);
            }
        } else {
            println!("[check_web_templates] Path does not exist: {:?}", path);
        }
        
        false
    } else {
        println!("[check_web_templates] Could not determine templates path");
        false
    }
}

#[tauri::command]
async fn ensure_export_templates(state: tauri::State<'_, AppState>) -> Result<String, String> {
    println!("[ensure_export_templates] Starting...");
    
    let settings = state.settings.lock().unwrap().clone();
    let godot_cmd = settings
        .godot_path
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .or_else(|| find_godot_path())
        .ok_or("Godot not found")?;
    
    println!("[ensure_export_templates] Godot: {}", godot_cmd);
    
    let version = get_godot_version(&godot_cmd)?;
    println!("[ensure_export_templates] Version: {}", version);
    
    if check_web_templates_installed(&version) {
        println!("[ensure_export_templates] Templates already installed");
        return Ok(format!("Export templates already installed for {}", version));
    }
    
    println!("[ensure_export_templates] Templates NOT installed, need to download...");
    
    // Need to download templates
    // URL format: https://github.com/godotengine/godot/releases/download/4.3-stable/Godot_v4.3-stable_export_templates.tpz
    let version_parts: Vec<&str> = version.split('.').collect();
    if version_parts.len() < 2 {
        return Err("Invalid version format".to_string());
    }
    
    let download_version = format!("{}.{}-{}", version_parts[0], version_parts[1], version_parts[2]);
    let url = format!(
        "https://github.com/godotengine/godot/releases/download/{}/Godot_v{}_export_templates.tpz",
        download_version, download_version
    );
    
    println!("[ensure_export_templates] Download URL: {}", url);
    
    let templates_dir = get_export_templates_path(&version)
        .ok_or("Could not determine templates directory")?;
    
    println!("[ensure_export_templates] Templates dir: {:?}", templates_dir);
    
    // Create templates directory
    fs::create_dir_all(&templates_dir)
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;
    
    println!("[ensure_export_templates] Starting download (this is ~700MB, may take a while)...");
    
    // Download the templates
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600)) // 10 minute timeout
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    println!("[ensure_export_templates] Sending request...");
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download templates: {}", e))?;
    
    println!("[ensure_export_templates] Got response: {}", response.status());
    
    if !response.status().is_success() {
        return Err(format!("Failed to download templates: HTTP {}", response.status()));
    }
    
    let total_size = response.content_length().unwrap_or(0);
    println!("[ensure_export_templates] Download size: {} MB", total_size / 1_000_000);
    
    // Stream to file instead of memory
    use tokio::io::AsyncWriteExt;
    let temp_path = templates_dir.join("templates.tpz");
    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    let mut downloaded: u64 = 0;
    let mut last_percent = 0u64;
    let mut stream = response.bytes_stream();
    
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk).await.map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let percent = (downloaded * 100) / total_size;
            if percent > last_percent && percent % 10 == 0 {
                println!("[ensure_export_templates] Downloaded {}%", percent);
                last_percent = percent;
            }
        }
    }
    
    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    drop(file);
    
    println!("[ensure_export_templates] Download complete, extracting...");
    
    // Extract the .tpz (it's a zip file)
    let file = fs::File::open(&temp_path)
        .map_err(|e| format!("Failed to open templates archive: {}", e))?;
    
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read templates archive: {}", e))?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        
        let name = file.name().to_string();
        // Files are in "templates/" folder in the archive
        if let Some(stripped) = name.strip_prefix("templates/") {
            if !stripped.is_empty() {
                let outpath = templates_dir.join(stripped);
                if file.is_dir() {
                    fs::create_dir_all(&outpath).ok();
                } else {
                    if let Some(parent) = outpath.parent() {
                        fs::create_dir_all(parent).ok();
                    }
                    let mut outfile = fs::File::create(&outpath)
                        .map_err(|e| format!("Failed to create file: {}", e))?;
                    std::io::copy(&mut file, &mut outfile)
                        .map_err(|e| format!("Failed to extract file: {}", e))?;
                }
            }
        }
    }
    
    // Clean up temp file
    fs::remove_file(&temp_path).ok();
    
    Ok(format!("Export templates installed for {}", version))
}

#[tauri::command]
fn clear_export_cache(project_path: String) -> Result<(), String> {
    let kobold_dir = Path::new(&project_path).join(".tav");
    if kobold_dir.exists() {
        fs::remove_dir_all(&kobold_dir)
            .map_err(|e| format!("Failed to clear cache: {}", e))?;
        println!("[Cache] Cleared .tav directory");
    }
    Ok(())
}

fn inject_kobold_bridge(project: &Path) -> Result<(), String> {
    // Write Kobold Bridge script to .tav folder
    let kobold_dir = project.join(".tav");
    fs::create_dir_all(&kobold_dir).ok();
    
    let bridge_path = kobold_dir.join("kobold_bridge.gd");
    fs::write(&bridge_path, KOBOLD_BRIDGE_GD)
        .map_err(|e| format!("Failed to write Kobold Bridge: {}", e))?;
    
    // Add to project.godot autoloads if not already present
    let project_file = project.join("project.godot");
    if project_file.exists() {
        let content = fs::read_to_string(&project_file)
            .map_err(|e| format!("Failed to read project.godot: {}", e))?;
        
        if !content.contains("KoboldBridge") {
            // Find [autoload] section and add our bridge
            let new_content = if content.contains("[autoload]") {
                content.replace(
                    "[autoload]",
                    "[autoload]\n\nKoboldBridge=\"*res://.tav/kobold_bridge.gd\""
                )
            } else {
                // Add autoload section before [input] or at end
                if content.contains("[input]") {
                    content.replace(
                        "[input]",
                        "[autoload]\n\nKoboldBridge=\"*res://.tav/kobold_bridge.gd\"\n\n[input]"
                    )
                } else {
                    format!("{}\n\n[autoload]\n\nKoboldBridge=\"*res://.tav/kobold_bridge.gd\"\n", content)
                }
            };
            
            fs::write(&project_file, new_content)
                .map_err(|e| format!("Failed to update project.godot: {}", e))?;
            
            println!("[Export] Injected KoboldBridge autoload");
        }
    }
    
    Ok(())
}

// Version bump this when bridge code changes to invalidate caches
const KOBOLD_BRIDGE_VERSION: u32 = 4;

// Template version - bump when template files change to trigger auto-sync
const TEMPLATE_VERSION: &str = "1.0.0";

/// Check if project's template needs updating and sync if so
fn sync_template_if_needed(project: &Path) -> Result<(), String> {
    let template_info_path = project.join(".tav/template_info.json");
    
    // Read existing template info
    let template_info: Option<serde_json::Value> = fs::read_to_string(&template_info_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    
    let (template_id, dimension, stored_version) = match &template_info {
        Some(info) => (
            info["template"].as_str().unwrap_or("").to_string(),
            info["dimension"].as_str().unwrap_or("3d").to_string(),
            info["version"].as_str().unwrap_or("0.0.0"),
        ),
        None => return Ok(()), // No template info, skip sync (legacy project)
    };
    
    // Check if update needed
    if stored_version == TEMPLATE_VERSION {
        return Ok(());
    }
    
    println!("[Template] Auto-syncing {} from v{} to v{}", template_id, stored_version, TEMPLATE_VERSION);
    
    // Get template files
    let (scene_content, player_content) = generate_template_files(&dimension, &template_id);
    
    // Sync scene and player files
    fs::write(project.join("scenes/main.tscn"), &scene_content)
        .map_err(|e| format!("Failed to sync main.tscn: {}", e))?;
    fs::write(project.join("assets/entities/player/player.gd"), &player_content)
        .map_err(|e| format!("Failed to sync player.gd: {}", e))?;
    
    // Ensure critical inputs exist in project.godot
    let project_godot_path = project.join("project.godot");
    if project_godot_path.exists() {
        let content = fs::read_to_string(&project_godot_path).unwrap_or_default();
        if !content.contains("sprint=") {
            // Add sprint input before [rendering] section
            let sprint_input = r#"sprint={
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":0,"physical_keycode":4194325,"key_label":0,"unicode":0,"location":0,"echo":false,"script":null)]
}
"#;
            let new_content = if content.contains("[rendering]") {
                content.replace("[rendering]", &format!("{}\n[rendering]", sprint_input))
            } else {
                format!("{}\n{}", content, sprint_input)
            };
            fs::write(&project_godot_path, new_content).ok();
            println!("[Template] Added missing 'sprint' input");
        }
    }
    
    // Update version in template info
    let new_info = serde_json::json!({
        "template": template_id,
        "dimension": dimension,
        "version": TEMPLATE_VERSION
    });
    fs::write(&template_info_path, serde_json::to_string_pretty(&new_info).unwrap())
        .map_err(|e| format!("Failed to update template_info.json: {}", e))?;
    
    println!("[Template] Sync complete - scene and player updated");
    Ok(())
}

fn get_project_hash(project_path: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    
    // Include bridge version so template changes invalidate cache
    KOBOLD_BRIDGE_VERSION.hash(&mut hasher);
    
    // Hash modification times of key files
    fn hash_dir(path: &Path, hasher: &mut DefaultHasher) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                
                // Skip hidden and generated directories
                if name.starts_with('.') || name == "export_presets.cfg" {
                    continue;
                }
                
                if path.is_dir() {
                    hash_dir(&path, hasher);
                } else if let Ok(meta) = path.metadata() {
                    if let Ok(modified) = meta.modified() {
                        modified.hash(hasher);
                        path.to_string_lossy().hash(hasher);
                    }
                }
            }
        }
    }
    
    hash_dir(project_path, &mut hasher);
    hasher.finish()
}

#[tauri::command]
fn export_project_web(project_path: String, force: Option<bool>, state: tauri::State<AppState>) -> Result<String, String> {
    let settings = state.settings.lock().unwrap();
    let godot_cmd = settings
        .godot_path
        .clone()
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .or_else(|| find_godot_path())
        .ok_or("Godot not found")?;

    let project = Path::new(&project_path);
    let export_dir = project.join(".tav/web");
    let hash_file = export_dir.join(".export_hash");
    
    // Calculate current project hash
    let current_hash = get_project_hash(project);
    
    // Auto-sync template files if version mismatch
    sync_template_if_needed(project)?;
    
    // Always inject/update Kobold Bridge first (even for cached exports)
    inject_kobold_bridge(project)?;
    
    // Check if we can use cached export
    if !force.unwrap_or(false) && export_dir.join("index.html").exists() {
        if let Ok(cached_hash) = fs::read_to_string(&hash_file) {
            if let Ok(cached) = cached_hash.trim().parse::<u64>() {
                if cached == current_hash {
                    // Still need to re-inject JS into cached HTML
                    inject_js_helper(&export_dir)?;
                    return Ok(format!("CACHED:{}", export_dir.to_string_lossy()));
                }
            }
        }
    }
    
    // Create export directory
    fs::create_dir_all(&export_dir)
        .map_err(|e| format!("Failed to create export directory: {}", e))?;
    
    // Create export_presets.cfg if it doesn't exist
    let presets_path = project.join("export_presets.cfg");
    if !presets_path.exists() {
        fs::write(&presets_path, WEB_EXPORT_PRESET)
            .map_err(|e| format!("Failed to write export presets: {}", e))?;
    }
    
    // Run Godot export (debug mode is faster)
    println!("[Export] Running: {} --headless --path {} --export-debug Web", godot_cmd, project_path);
    
    let output = Command::new(&godot_cmd)
        .args([
            "--headless",
            "--path", &project_path,
            "--export-debug", "Web",
            &export_dir.join("index.html").to_string_lossy(),
        ])
        .current_dir(&project_path)
        .output()
        .map_err(|e| format!("Export failed: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !stdout.is_empty() {
        println!("[Export] stdout: {}", stdout);
    }
    if !stderr.is_empty() {
        println!("[Export] stderr: {}", stderr);
    }
    
    if !output.status.success() {
        return Err(format!("Export failed (exit {}): {}", output.status, stderr));
    }
    
    // Verify export succeeded
    if !export_dir.join("index.html").exists() {
        // List what files were created
        if export_dir.exists() {
            let files: Vec<_> = fs::read_dir(&export_dir)
                .map(|entries| entries.filter_map(|e| e.ok().map(|f| f.file_name().to_string_lossy().to_string())).collect())
                .unwrap_or_default();
            return Err(format!("Export completed but index.html not found. Files in export dir: {:?}", files));
        }
        return Err("Export completed but index.html not found. Make sure Godot Web export templates are installed.".to_string());
    }
    
    // Inject JS helper into exported HTML
    inject_js_helper(&export_dir)?;
    
    // Save hash for caching
    fs::write(&hash_file, current_hash.to_string()).ok();
    
    Ok(export_dir.to_string_lossy().to_string())
}

fn inject_js_helper(export_dir: &Path) -> Result<(), String> {
    let index_path = export_dir.join("index.html");
    let html = fs::read_to_string(&index_path)
        .map_err(|e| format!("Failed to read index.html: {}", e))?;
    
    // Skip if already injected
    if html.contains("KoboldBridge") {
        return Ok(());
    }
    
    let capture_script = r#"
<script>
// Kobold Bridge Helper - Uses native Godot API when available, falls back to canvas
(function() {
    const canvas = () => document.querySelector('canvas');
    let bridgeReady = false;
    
    // Wait for KoboldBridge to be ready
    window.addEventListener('kobold-bridge-ready', () => {
        bridgeReady = true;
        console.log('[Kobold] Bridge ready - using native Godot API');
    });
    
    // Capture current frame (always use canvas for images)
    function captureFrame() {
        const c = canvas();
        if (!c) return null;
        return { data: c.toDataURL('image/png'), width: c.width, height: c.height };
    }
    
    // Get game state from bridge
    function getGameState() {
        if (bridgeReady && window.KoboldBridge) {
            try {
                return JSON.parse(window.KoboldBridge.getState());
            } catch (e) {
                console.warn('[Kobold] Failed to get state:', e);
            }
        }
        return null;
    }
    
    // Send input via bridge (native) or simulate keys (fallback)
    function sendInput(action, pressed = true) {
        if (bridgeReady && window.KoboldBridge) {
            try {
                const result = JSON.parse(window.KoboldBridge.sendInput(action, pressed));
                return result.success;
            } catch (e) {
                console.warn('[Kobold] Bridge input failed, using fallback:', e);
            }
        }
        // Fallback to key simulation
        simulateKeyForAction(action, pressed ? 'keydown' : 'keyup');
        return true;
    }
    
    // Map actions to keys for fallback
    function simulateKeyForAction(action, type) {
        const c = canvas();
        if (!c) return;
        
        // Common action to key mappings
        const actionKeyMap = {
            'move_left': 'a', 'move_right': 'd', 'move_up': 'w', 'move_down': 's',
            'move_forward': 'w', 'move_back': 's',
            'jump': ' ', 'attack': 'mouse1', 'interact': 'e'
        };
        
        const key = actionKeyMap[action] || action;
        if (key === 'mouse1') return; // Can't simulate mouse
        
        const keyMap = {
            'w': 'KeyW', 's': 'KeyS', 'a': 'KeyA', 'd': 'KeyD', 'e': 'KeyE',
            ' ': 'Space', 'space': 'Space', 'shift': 'ShiftLeft'
        };
        
        const code = keyMap[key.toLowerCase()] || `Key${key.toUpperCase()}`;
        const event = new KeyboardEvent(type, { key, code, bubbles: true, cancelable: true });
        c.dispatchEvent(event);
        document.dispatchEvent(event);
    }
    
    window.addEventListener('message', async function(event) {
        if (!event.data || !event.data.type) return;
        
        // Focus the game canvas for input
        if (event.data.type === 'kobold-focus') {
            const canvas = document.querySelector('canvas');
            if (canvas) {
                canvas.focus();
                // Also click it to ensure Godot captures input
                canvas.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, clientX: canvas.width / 2, clientY: canvas.height / 2 }));
                canvas.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: canvas.width / 2, clientY: canvas.height / 2 }));
            }
            return;
        }
        
        // Simple capture
        if (event.data.type === 'kobold-capture') {
            const frame = captureFrame();
            const state = getGameState();
            if (frame) {
                window.parent.postMessage({ type: 'kobold-capture-result', ...frame, state }, '*');
            } else {
                window.parent.postMessage({ type: 'kobold-capture-error', error: 'No canvas' }, '*');
            }
        }
        
        // Get game state
        if (event.data.type === 'kobold-get-state') {
            const state = getGameState();
            window.parent.postMessage({ type: 'kobold-state-result', state }, '*');
        }
        
        // Test controls: capture before, send input via bridge, wait, capture after
        if (event.data.type === 'kobold-test-controls') {
            try {
                const { actions, duration = 1000 } = event.data;
                
                // Capture before with state
                const before = captureFrame();
                const stateBefore = getGameState();
                
                // Send inputs via bridge (or fallback)
                for (const action of actions) {
                    sendInput(action, true);
                }
                
                // Wait
                await new Promise(r => setTimeout(r, duration));
                
                // Capture after with state
                const after = captureFrame();
                const stateAfter = getGameState();
                
                // Release inputs
                for (const action of actions) {
                    sendInput(action, false);
                }
                
                window.parent.postMessage({
                    type: 'kobold-test-result',
                    before: before?.data,
                    after: after?.data,
                    stateBefore,
                    stateAfter,
                    actions,
                    duration,
                    bridgeUsed: bridgeReady
                }, '*');
            } catch (e) {
                window.parent.postMessage({ type: 'kobold-test-error', error: e.toString() }, '*');
            }
        }
        
        // Capture node from multiple angles (uses native Godot bridge)
        if (event.data.type === 'kobold-capture-node') {
            if (!bridgeReady) {
                window.parent.postMessage({ type: 'kobold-capture-node-error', error: 'Bridge not ready' }, '*');
                return;
            }
            
            const { nodeId, options = {} } = event.data;
            const requestId = event.data.requestId || Date.now().toString();
            
            // Call bridge to start capture
            try {
                const result = window.KoboldBridge.captureNode(nodeId, options);
                const parsed = JSON.parse(result);
                
                if (parsed.promise_id) {
                    // Wait for async result
                    const handler = (e) => {
                        if (e.detail && e.detail.id === parsed.promise_id) {
                            window.removeEventListener('kobold-capture-complete', handler);
                            window.parent.postMessage({
                                type: 'kobold-capture-node-result',
                                requestId,
                                result: e.detail.result
                            }, '*');
                        }
                    };
                    window.addEventListener('kobold-capture-complete', handler);
                    // Timeout after 10s
                    setTimeout(() => {
                        window.removeEventListener('kobold-capture-complete', handler);
                        window.parent.postMessage({ type: 'kobold-capture-node-error', requestId, error: 'Timeout' }, '*');
                    }, 10000);
                } else {
                    // Immediate result (error)
                    window.parent.postMessage({
                        type: 'kobold-capture-node-error',
                        requestId,
                        error: parsed.error || 'Unknown error'
                    }, '*');
                }
            } catch (e) {
                window.parent.postMessage({ type: 'kobold-capture-node-error', requestId, error: e.toString() }, '*');
            }
        }
        
        // Find a node by name
        if (event.data.type === 'kobold-find-node') {
            if (!bridgeReady) {
                window.parent.postMessage({ type: 'kobold-find-node-result', found: false, error: 'Bridge not ready' }, '*');
                return;
            }
            
            try {
                const result = window.KoboldBridge.findNode(event.data.name);
                const parsed = JSON.parse(result);
                window.parent.postMessage({ type: 'kobold-find-node-result', ...parsed }, '*');
            } catch (e) {
                window.parent.postMessage({ type: 'kobold-find-node-result', found: false, error: e.toString() }, '*');
            }
        }
    });
    
    console.log('[Kobold] Helper loaded, waiting for bridge...');
})();
</script>
</head>"#;
    
    let modified_html = html.replace("</head>", capture_script);
    fs::write(&index_path, modified_html)
        .map_err(|e| format!("Failed to write index.html: {}", e))?;
    
    println!("[Export] Injected Kobold JS helper");
    Ok(())
}

#[tauri::command]
fn start_preview_server(export_path: String) -> Result<u16, String> {
    use std::thread;
    
    // Verify export path exists
    let export_dir = Path::new(&export_path);
    if !export_dir.exists() {
        return Err(format!("Export directory does not exist: {}", export_path));
    }
    if !export_dir.join("index.html").exists() {
        return Err(format!("index.html not found in: {}", export_path));
    }
    
    println!("[PreviewServer] Starting server for: {}", export_path);
    
    // Find an available port
    let port = (8080..9000)
        .find(|p| std::net::TcpListener::bind(("127.0.0.1", *p)).is_ok())
        .ok_or("No available port found")?;
    
    println!("[PreviewServer] Using port: {}", port);
    
    let export_path_clone = export_path.clone();
    thread::spawn(move || {
        let server = match tiny_http::Server::http(format!("127.0.0.1:{}", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[PreviewServer] Failed to start: {}", e);
                return;
            }
        };
        
        println!("[PreviewServer] Server running on http://127.0.0.1:{}", port);
        
        for request in server.incoming_requests() {
            let url = request.url().to_string();
            let file_path = if url == "/" || url.is_empty() {
                Path::new(&export_path_clone).join("index.html")
            } else {
                Path::new(&export_path_clone).join(url.trim_start_matches('/'))
            };
            
            println!("[PreviewServer] Request: {} -> {:?}", url, file_path);
            
            let response = if file_path.exists() {
                let content = match fs::read(&file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[PreviewServer] Failed to read file: {}", e);
                        let r = tiny_http::Response::from_string("Read error")
                            .with_status_code(500);
                        let _ = request.respond(r);
                        continue;
                    }
                };
                
                let mime = match file_path.extension().and_then(|e| e.to_str()) {
                    Some("html") => "text/html; charset=utf-8",
                    Some("js") => "application/javascript",
                    Some("wasm") => "application/wasm",
                    Some("png") => "image/png",
                    Some("ico") => "image/x-icon",
                    Some("pck") => "application/octet-stream",
                    Some("css") => "text/css",
                    _ => "application/octet-stream",
                };
                
                tiny_http::Response::from_data(content)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap())
                    .with_header(tiny_http::Header::from_bytes(&b"Cross-Origin-Opener-Policy"[..], &b"same-origin"[..]).unwrap())
                    .with_header(tiny_http::Header::from_bytes(&b"Cross-Origin-Embedder-Policy"[..], &b"require-corp"[..]).unwrap())
                    .with_header(tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
            } else {
                println!("[PreviewServer] 404: {:?}", file_path);
                tiny_http::Response::from_string("Not found").with_status_code(404)
            };
            
            let _ = request.respond(response);
        }
    });
    
    // Give the server a moment to start
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    Ok(port)
}

// ============================================================================
// Game Playing Commands
// ============================================================================

#[tauri::command]
fn start_game_session(
    project_path: String,
    scene_path: String,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap();
    let godot_cmd = settings
        .godot_path
        .clone()
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .or_else(|| find_godot_path())
        .ok_or("Godot not found")?;
    drop(settings);

    // Create screenshots directory
    let screenshots_dir = Path::new(&project_path).join("user_screenshots");
    fs::create_dir_all(&screenshots_dir).ok();

    // Create agent input file
    let input_file = Path::new(&project_path).join("agent_input.json");
    fs::write(&input_file, "{}").ok();

    // Launch Godot windowed (not headless - we need rendering for screenshots)
    let child = Command::new(&godot_cmd)
        .args([
            "--path", &project_path,
            "--resolution", "768x768",
            "--position", "0,0",
            "--fixed-fps", "10",
            &scene_path
        ])
        .env("AGENT_ENABLED", "true")
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Godot: {}", e))?;

    let session_id = uuid::Uuid::new_v4().to_string();
    
    let session = GameSession {
        id: session_id.clone(),
        process: Some(child),
        project_path,
        scene_path,
        frame_count: 0,
    };

    state.game_sessions.lock().unwrap().insert(session_id.clone(), session);
    
    // Give Godot time to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    Ok(session_id)
}

#[tauri::command]
fn get_game_frame(session_id: String, state: tauri::State<AppState>) -> Result<GameFrame, String> {
    let mut sessions = state.game_sessions.lock().unwrap();
    let session = sessions.get_mut(&session_id).ok_or("Session not found")?;

    let screenshots_dir = Path::new(&session.project_path).join("user_screenshots");
    
    // Find latest screenshot
    let mut latest_screenshot = String::new();
    let mut latest_num = 0;

    if let Ok(entries) = fs::read_dir(&screenshots_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("frame_") && name.ends_with(".png") {
                    if let Ok(num) = name
                        .strip_prefix("frame_")
                        .and_then(|s| s.strip_suffix(".png"))
                        .unwrap_or("0")
                        .parse::<u32>()
                    {
                        if num > latest_num {
                            latest_num = num;
                            latest_screenshot = entry.path().to_string_lossy().to_string();
                        }
                    }
                }
            }
        }
    }

    // Read screenshot as base64
    let screenshot_b64 = if !latest_screenshot.is_empty() && Path::new(&latest_screenshot).exists() {
        let data = fs::read(&latest_screenshot).unwrap_or_default();
        base64::engine::general_purpose::STANDARD.encode(&data)
    } else {
        String::new()
    };

    // Read game state
    let state_path = Path::new(&session.project_path).join("game_state.json");
    let game_state = if state_path.exists() {
        let content = fs::read_to_string(&state_path).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Read logs
    let logs_path = Path::new(&session.project_path).join("game.log");
    let logs: Vec<String> = if logs_path.exists() {
        fs::read_to_string(&logs_path)
            .unwrap_or_default()
            .lines()
            .rev()
            .take(20)
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![]
    };

    session.frame_count = latest_num;

    Ok(GameFrame {
        screenshot: screenshot_b64,
        state: game_state,
        logs,
        frame_count: latest_num,
    })
}

#[tauri::command]
fn send_game_action(
    session_id: String,
    action: GameAction,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let sessions = state.game_sessions.lock().unwrap();
    let session = sessions.get(&session_id).ok_or("Session not found")?;

    let input_path = Path::new(&session.project_path).join("agent_input.json");
    let action_json = serde_json::to_string(&action).map_err(|e| e.to_string())?;
    fs::write(&input_path, action_json).map_err(|e| format!("Failed to write action: {}", e))?;

    // Wait for game to process
    std::thread::sleep(std::time::Duration::from_millis(50));

    Ok(())
}

#[tauri::command]
fn execute_actions(
    session_id: String,
    actions: Vec<GameAction>,
    state: tauri::State<AppState>,
) -> Result<u32, String> {
    let sessions = state.game_sessions.lock().unwrap();
    let session = sessions.get(&session_id).ok_or("Session not found")?;
    let project_path = session.project_path.clone();
    drop(sessions);

    let input_path = Path::new(&project_path).join("agent_input.json");
    let mut executed = 0;

    for action in actions {
        let action_json = serde_json::to_string(&action).map_err(|e| e.to_string())?;
        fs::write(&input_path, action_json).map_err(|e| format!("Failed to write action: {}", e))?;
        executed += 1;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(executed)
}

#[tauri::command]
fn stop_game_session(session_id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut sessions = state.game_sessions.lock().unwrap();
    
    if let Some(mut session) = sessions.remove(&session_id) {
        if let Some(mut process) = session.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }

    Ok(())
}

// ============================================================================
// Playtest Agent - Real-time game testing with Gemini Live API (WebSocket)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct PlaytestEvent {
    pub event_type: String,  // "start", "connected", "action", "observation", "complete", "error"
    pub message: String,
    pub frame: Option<u32>,
    pub action: Option<String>,
    pub screenshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaytestConfig {
    pub objective: String,
    pub max_duration_secs: Option<u64>,
}

/// Game action tools for Gemini to call
fn get_game_tools() -> serde_json::Value {
    serde_json::json!([{
        "functionDeclarations": [
            {
                "name": "move",
                "description": "Move the player in a direction",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "direction": {
                            "type": "string",
                            "enum": ["left", "right", "up", "down", "stop"],
                            "description": "Direction to move"
                        }
                    },
                    "required": ["direction"]
                }
            },
            {
                "name": "jump",
                "description": "Make the player jump"
            },
            {
                "name": "sprint",
                "description": "Toggle sprinting while moving",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "enabled": {"type": "boolean", "description": "true to sprint, false to stop"}
                    },
                    "required": ["enabled"]
                }
            },
            {
                "name": "look",
                "description": "Rotate the camera/view direction",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "x": {"type": "number", "description": "Horizontal rotation in degrees"},
                        "y": {"type": "number", "description": "Vertical rotation in degrees"}
                    },
                    "required": ["x", "y"]
                }
            },
            {
                "name": "interact",
                "description": "Interact with nearby object or NPC"
            },
            {
                "name": "attack",
                "description": "Perform an attack action"
            },
            {
                "name": "stop",
                "description": "Stop all movement"
            },
            {
                "name": "report_observation",
                "description": "Report what you observe in the game",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "observation": {"type": "string", "description": "What you see"},
                        "progress": {"type": "string", "description": "Progress toward objective"}
                    },
                    "required": ["observation"]
                }
            }
        ]
    }])
}

#[tauri::command]
async fn run_playtest(
    app: tauri::AppHandle,
    project_path: String,
    config: PlaytestConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let godot_cmd = settings.godot_path.clone()
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .or_else(|| find_godot_path())
        .ok_or("Godot not found")?;
    let api_key = settings.gemini_key.clone()
        .or_else(|| read_env_file_key(&project_path, "GEMINI_API_KEY"))
        .ok_or("Gemini API key required. Add it in Settings or .env.local")?;
    
    println!("[Playtest] Starting with API key: {}...", &api_key[..12.min(api_key.len())]);
    
    let max_steps = config.max_duration_secs.unwrap_or(30) as u32;
    
    let _ = app.emit("playtest-event", PlaytestEvent {
        event_type: "start".to_string(),
        message: format!("Starting playtest: {}", config.objective),
        frame: None, action: None, screenshot: None,
    });

    // Setup project directories
    let project = Path::new(&project_path);
    let screenshots_dir = project.join("user_screenshots");
    fs::create_dir_all(&screenshots_dir).ok();
    fs::write(project.join("agent_input.json"), "{}").ok();
    
    // Clear old screenshots
    if let Ok(entries) = fs::read_dir(&screenshots_dir) {
        for entry in entries.flatten() {
            fs::remove_file(entry.path()).ok();
        }
    }

    // Start Godot windowed (768x768 optimal for Gemini)
    let mut godot = Command::new(&godot_cmd)
        .args([
            "--path", &project_path,
            "--resolution", "768x768",
            "--position", "0,0",
            "res://scenes/main.tscn"
        ])
        .env("AGENT_ENABLED", "true")
        .current_dir(&project_path)
        .spawn()
        .map_err(|e| format!("Failed to start Godot: {}", e))?;

    // Wait for Godot to initialize
    tokio::time::sleep(tokio::time::Duration::from_millis(2500)).await;

    let _ = app.emit("playtest-event", PlaytestEvent {
        event_type: "connected".to_string(),
        message: "Godot started, AI analyzing frames...".to_string(),
        frame: None, action: None, screenshot: None,
    });

    let client = reqwest::Client::new();
    let api_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    let system_prompt = format!(
        r#"You are a game-playing AI agent. Your objective: {}

You see a screenshot from a video game. Analyze it and decide what action to take.

AVAILABLE ACTIONS (respond with exactly one):
- move_left - Move character left
- move_right - Move character right  
- move_up - Move character forward/up
- move_down - Move character backward/down
- jump - Make character jump
- stop - Stop moving
- look_left - Turn camera left
- look_right - Turn camera right

RESPOND WITH JSON ONLY:
{{"observation": "what you see", "action": "action_name", "reasoning": "why"}}"#,
        config.objective
    );

    let mut observations: Vec<String> = Vec::new();
    let mut last_action = String::new();
    let mut last_frame_num = 0u32;

    // Main control loop - analyze frames and take actions
    println!("[Playtest] Starting main loop, max_steps={}", max_steps);
    
    for step in 0..max_steps {
        // Check if Godot still running
        if let Ok(Some(_)) = godot.try_wait() {
            println!("[Playtest] Godot exited at step {}", step);
            let _ = app.emit("playtest-event", PlaytestEvent {
                event_type: "error".to_string(),
                message: "Godot exited".to_string(),
                frame: Some(step), action: None, screenshot: None,
            });
            break;
        }

        // Wait for new frame
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        // Find latest screenshot
        let mut latest_path: Option<PathBuf> = None;
        let mut latest_num = 0u32;
        if let Ok(entries) = fs::read_dir(&screenshots_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("frame_") && name.ends_with(".png") {
                        if let Ok(num) = name.replace("frame_", "").replace(".png", "").parse::<u32>() {
                            if num > latest_num {
                                latest_num = num;
                                latest_path = Some(entry.path());
                            }
                        }
                    }
                }
            }
        }

        // Skip if no new frame
        if latest_num <= last_frame_num {
            if step % 5 == 0 {
                println!("[Playtest] Step {}: waiting for new frame (last={})", step, last_frame_num);
            }
            continue;
        }
        last_frame_num = latest_num;
        println!("[Playtest] Step {}: Processing frame {}", step, latest_num);

        let screenshot_b64 = match &latest_path {
            Some(p) if p.exists() => {
                base64::engine::general_purpose::STANDARD.encode(&fs::read(p).unwrap_or_default())
            }
            _ => continue,
        };

        // Build prompt with history
        let history = if observations.len() > 3 {
            observations[observations.len()-3..].join("\n")
        } else {
            observations.join("\n")
        };

        let prompt = format!(
            "{}\n\nLast action: {}\nRecent observations:\n{}\n\nAnalyze this frame and choose your next action:",
            system_prompt, last_action, history
        );

        // Call Gemini API
        println!("[Playtest] Calling Gemini API (image size: {} bytes)...", screenshot_b64.len());
        
        let response = client
            .post(&api_url)
            .json(&serde_json::json!({
                "contents": [{
                    "parts": [
                        {"text": prompt},
                        {"inlineData": {"mimeType": "image/png", "data": screenshot_b64}}
                    ]
                }],
                "generationConfig": {
                    "temperature": 0.3,
                    "maxOutputTokens": 300
                }
            }))
            .send()
            .await;

        let ai_text = match response {
            Ok(resp) => {
                let status = resp.status();
                let json: serde_json::Value = resp.json().await.unwrap_or_default();
                if !status.is_success() {
                    println!("[Playtest] API error {}: {:?}", status, json);
                    let _ = app.emit("playtest-event", PlaytestEvent {
                        event_type: "error".to_string(),
                        message: format!("API error: {}", json.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).unwrap_or("Unknown")),
                        frame: Some(step), action: None, screenshot: None,
                    });
                    continue;
                }
                let text = json["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                println!("[Playtest] Got response: {}...", &text[..50.min(text.len())]);
                text
            }
            Err(e) => {
                println!("[Playtest] API request failed: {}", e);
                continue;
            }
        };

        // Parse response
        let clean = ai_text.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        if let Ok(data) = serde_json::from_str::<serde_json::Value>(clean) {
            let observation = data["observation"].as_str().unwrap_or("").to_string();
            let action = data["action"].as_str().unwrap_or("").to_string();
            let reasoning = data["reasoning"].as_str().unwrap_or("").to_string();

            observations.push(format!("[{}] {}", step, observation));
            
            let _ = app.emit("playtest-event", PlaytestEvent {
                event_type: "observation".to_string(),
                message: format!("{} → {}", observation, reasoning),
                frame: Some(step), action: None, screenshot: None,
            });

            if !action.is_empty() {
                // Map action to game control
                let (func, args): (&str, Vec<serde_json::Value>) = match action.as_str() {
                    "move_left" => ("move", vec![serde_json::json!("left")]),
                    "move_right" => ("move", vec![serde_json::json!("right")]),
                    "move_up" => ("move", vec![serde_json::json!("up")]),
                    "move_down" => ("move", vec![serde_json::json!("down")]),
                    "jump" => ("jump", vec![]),
                    "stop" => ("stop", vec![]),
                    "look_left" => ("look", vec![serde_json::json!(-30), serde_json::json!(0)]),
                    "look_right" => ("look", vec![serde_json::json!(30), serde_json::json!(0)]),
                    _ => ("stop", vec![]),
                };

                let action_json = serde_json::json!({"function": func, "args": args});
                fs::write(project.join("agent_input.json"), action_json.to_string()).ok();
                
                last_action = action.clone();
                
                let _ = app.emit("playtest-event", PlaytestEvent {
                    event_type: "action".to_string(),
                    message: action.clone(),
                    frame: Some(step),
                    action: Some(action),
                    screenshot: None,
                });
            }
        } else {
            println!("[Playtest] Failed to parse: {}", clean);
        }
    }

    // Cleanup
    let _ = godot.kill();
    let _ = godot.wait();

    let summary = format!(
        "Playtest complete. {} steps, {} observations.",
        max_steps, observations.len()
    );
    
    let _ = app.emit("playtest-event", PlaytestEvent {
        event_type: "complete".to_string(),
        message: summary.clone(),
        frame: None, action: None, screenshot: None,
    });

    Ok(summary)
}

// ============================================================================
// NitroGen Playtest - Local vision-to-action model via Tauri Sidecar
// ============================================================================

use controls::{ControlMapper, ControlMappings};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

/// NitroGen server state (managed globally)
static NITROGEN_SERVER: std::sync::OnceLock<std::sync::Mutex<Option<std::process::Child>>> = std::sync::OnceLock::new();

/// Sidecar process handle
static NITROGEN_SIDECAR: std::sync::OnceLock<std::sync::Mutex<Option<u32>>> = std::sync::OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NitrogenStatus {
    pub installed: bool,
    pub checkpoint_exists: bool,
    pub server_running: bool,
    pub sidecar_available: bool,
    pub python_path: Option<String>,
    pub nitrogen_path: Option<String>,
}

#[tauri::command]
fn check_nitrogen_installed(app: tauri::AppHandle) -> NitrogenStatus {
    let python_path = which_python();
    let nitrogen_path = find_nitrogen_path();
    
    // Check for checkpoint in multiple locations
    let (checkpoint_exists, checkpoint_path) = find_checkpoint_path(&app);
    
    let server_running = NITROGEN_SERVER.get()
        .and_then(|m| m.lock().ok())
        .map(|guard| guard.is_some())
        .unwrap_or(false);
    
    // Check if sidecar binary exists
    let sidecar_available = app.shell()
        .sidecar("binaries/nitrogen-sidecar")
        .is_ok();
    
    // Consider "installed" if we have the checkpoint (model is what matters)
    let installed = checkpoint_exists || nitrogen_path.is_some();
    
    NitrogenStatus {
        installed,
        checkpoint_exists,
        server_running,
        sidecar_available,
        python_path,
        nitrogen_path: checkpoint_path.or(nitrogen_path),
    }
}

/// Find the ng.pt checkpoint file in various locations
fn find_checkpoint_path(app: &tauri::AppHandle) -> (bool, Option<String>) {
    // Priority 1: Bundled in binaries folder (src-tauri/binaries/ng.pt)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled = resource_dir.join("binaries/ng.pt");
        if bundled.exists() {
            return (true, Some(bundled.to_string_lossy().to_string()));
        }
    }
    
    // Priority 2: Next to executable (for dev mode)
    if let Ok(exe_dir) = std::env::current_exe().and_then(|p| Ok(p.parent().unwrap().to_path_buf())) {
        let dev_path = exe_dir.join("../binaries/ng.pt");
        if dev_path.exists() {
            return (true, Some(dev_path.canonicalize().unwrap_or(dev_path).to_string_lossy().to_string()));
        }
    }
    
    // Priority 3: In src-tauri/binaries (dev mode from workspace root)
    let workspace_path = PathBuf::from("src-tauri/binaries/ng.pt");
    if workspace_path.exists() {
        return (true, Some(workspace_path.canonicalize().unwrap_or(workspace_path).to_string_lossy().to_string()));
    }
    
    // Priority 4: Common user locations
    if let Some(home) = dirs::home_dir() {
        for subpath in &["NitroGen/ng.pt", "Documents/NitroGen/ng.pt", "projects/NitroGen/ng.pt"] {
            let path = home.join(subpath);
            if path.exists() {
                return (true, Some(path.to_string_lossy().to_string()));
            }
        }
    }
    
    // Priority 5: Absolute Windows paths
    for path_str in &["C:/NitroGen/ng.pt", "D:/NitroGen/ng.pt"] {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return (true, Some(path.to_string_lossy().to_string()));
        }
    }
    
    (false, None)
}

fn which_python() -> Option<String> {
    for cmd in &["python", "python3", "py"] {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                return Some(cmd.to_string());
            }
        }
    }
    None
}

fn find_nitrogen_path() -> Option<String> {
    let home = dirs::home_dir()?;
    let candidates = vec![
        home.join("NitroGen"),
        home.join("Documents/NitroGen"),
        home.join("projects/NitroGen"),
        PathBuf::from("C:/NitroGen"),
        PathBuf::from("D:/NitroGen"),
    ];
    
    for path in candidates {
        if path.join("scripts/serve.py").exists() || path.join("ng.pt").exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    
    if let Some(python) = which_python() {
        if let Ok(output) = Command::new(&python)
            .args(["-c", "import nitrogen; print(nitrogen.__path__[0])"])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[tauri::command]
fn start_nitrogen_server(app: tauri::AppHandle, checkpoint_path: Option<String>) -> Result<String, String> {
    let python = which_python().ok_or("Python not found. Install Python 3.10+ first.")?;
    
    // Find checkpoint
    let ckpt = checkpoint_path.unwrap_or_else(|| {
        let (_, path) = find_checkpoint_path(&app);
        path.unwrap_or_else(|| "ng.pt".to_string())
    });
    
    if !Path::new(&ckpt).exists() {
        return Err(format!("Checkpoint not found at {}. Download ng.pt from HuggingFace.", ckpt));
    }
    
    // Find serve.py script - check NitroGen install or use bundled
    let serve_script = find_serve_script().ok_or(
        "NitroGen serve.py not found. Install: pip install nitrogen"
    )?;
    
    let working_dir = Path::new(&serve_script).parent().unwrap_or(Path::new("."));
    
    let child = Command::new(&python)
        .args([&serve_script, &ckpt, "--port", "5555"])
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start NitroGen server: {}", e))?;
    
    let server_mutex = NITROGEN_SERVER.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut guard) = server_mutex.lock() {
        *guard = Some(child);
    }
    
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok("NitroGen server started on port 5555".to_string())
}

/// Find the NitroGen serve.py script
fn find_serve_script() -> Option<String> {
    // Check common locations
    if let Some(home) = dirs::home_dir() {
        for subpath in &["NitroGen/scripts/serve.py", "Documents/NitroGen/scripts/serve.py", "projects/NitroGen/scripts/serve.py"] {
            let path = home.join(subpath);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    
    for path_str in &["C:/NitroGen/scripts/serve.py", "D:/NitroGen/scripts/serve.py"] {
        if Path::new(path_str).exists() {
            return Some(path_str.to_string());
        }
    }
    
    // Check if nitrogen module is installed and has serve script
    if let Some(python) = which_python() {
        if let Ok(output) = Command::new(&python)
            .args(["-c", "import nitrogen; import os; print(os.path.join(os.path.dirname(nitrogen.__file__), 'scripts', 'serve.py'))"])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if Path::new(&path).exists() {
                    return Some(path);
                }
            }
        }
    }
    
    None
}

#[tauri::command]
fn stop_nitrogen_server() -> Result<(), String> {
    let server_mutex = NITROGEN_SERVER.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut guard) = server_mutex.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    Ok(())
}

#[tauri::command]
fn get_control_mappings(project_path: String) -> ControlMappings {
    ControlMapper::load_from_project(Path::new(&project_path)).mappings.clone()
}

#[tauri::command]
fn save_control_mappings(project_path: String, mappings: ControlMappings) -> Result<(), String> {
    ControlMapper::new(mappings).save_to_project(Path::new(&project_path))
}

#[tauri::command]
async fn run_playtest_nitrogen(
    app: tauri::AppHandle,
    project_path: String,
    config: PlaytestConfig,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let godot_cmd = settings.godot_path.clone()
        .filter(|p| !p.is_empty() && Path::new(p).exists())
        .or_else(|| find_godot_path())
        .ok_or("Godot not found")?;
    
    let max_steps = config.max_duration_secs.unwrap_or(60) as u32;
    
    let _ = app.emit("playtest-event", PlaytestEvent {
        event_type: "start".to_string(),
        message: format!("Starting NitroGen playtest: {}", config.objective),
        frame: None, action: None, screenshot: None,
    });

    // Check NitroGen
    let status = check_nitrogen_installed(app.clone());
    if !status.checkpoint_exists {
        return Err("NitroGen checkpoint (ng.pt) not found. Download from HuggingFace.".to_string());
    }
    
    // Start server if not running
    if !status.server_running {
        let _ = app.emit("playtest-event", PlaytestEvent {
            event_type: "connected".to_string(),
            message: "Starting NitroGen server...".to_string(),
            frame: None, action: None, screenshot: None,
        });
        start_nitrogen_server(app.clone(), status.nitrogen_path.clone())?;
    }

    // Setup directories
    let project = Path::new(&project_path);
    let screenshots_dir = project.join("user_screenshots");
    fs::create_dir_all(&screenshots_dir).ok();
    fs::write(project.join("agent_input.json"), "{}").ok();
    
    // Clear old screenshots
    if let Ok(entries) = fs::read_dir(&screenshots_dir) {
        for entry in entries.flatten() {
            fs::remove_file(entry.path()).ok();
        }
    }

    // Load control mappings
    let mut mapper = ControlMapper::load_from_project(project);

    // Start Godot
    let mut godot = Command::new(&godot_cmd)
        .args(["--path", &project_path, "--resolution", "768x768", "--position", "0,0", "res://scenes/main.tscn"])
        .env("AGENT_ENABLED", "true")
        .current_dir(&project_path)
        .spawn()
        .map_err(|e| format!("Failed to start Godot: {}", e))?;

    tokio::time::sleep(tokio::time::Duration::from_millis(2500)).await;

    let _ = app.emit("playtest-event", PlaytestEvent {
        event_type: "connected".to_string(),
        message: "Starting NitroGen sidecar...".to_string(),
        frame: None, action: None, screenshot: None,
    });

    // Spawn sidecar using Tauri shell plugin
    let sidecar_result = app.shell()
        .sidecar("binaries/nitrogen-sidecar")
        .map_err(|e| format!("Sidecar not found: {}. Run build-sidecar.py first.", e))?
        .spawn();

    let (mut rx, mut child) = match sidecar_result {
        Ok(result) => result,
        Err(e) => {
            let _ = godot.kill();
            return Err(format!("Failed to spawn sidecar: {}", e));
        }
    };

    // Store sidecar PID
    let sidecar_mutex = NITROGEN_SIDECAR.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut guard) = sidecar_mutex.lock() {
        *guard = Some(child.pid());
    }

    // Wait for ready signal
    let timeout = tokio::time::timeout(tokio::time::Duration::from_secs(10), async {
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) = event {
                let text = String::from_utf8_lossy(&line);
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                    if msg.get("type").and_then(|t| t.as_str()) == Some("ready") {
                        return true;
                    }
                }
            }
        }
        false
    }).await;

    let ready = timeout.unwrap_or(false);
    if !ready {
        let _ = child.kill();
        let _ = godot.kill();
        return Err("Sidecar did not become ready in time".to_string());
    }

    // Send connect command
    let connect_cmd = serde_json::json!({"type": "connect", "addr": "tcp://127.0.0.1:5555"});
    child.write(format!("{}\n", connect_cmd).as_bytes()).map_err(|e| e.to_string())?;

    // Wait for connected response
    let connect_timeout = tokio::time::timeout(tokio::time::Duration::from_secs(10), async {
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) = event {
                let text = String::from_utf8_lossy(&line);
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                    let msg_type = msg.get("type").and_then(|t| t.as_str());
                    if msg_type == Some("connected") {
                        return Ok(());
                    } else if msg_type == Some("error") {
                        return Err(msg.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error").to_string());
                    }
                }
            }
        }
        Err("Connection timeout".to_string())
    }).await;

    match connect_timeout {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            let _ = child.kill();
            let _ = godot.kill();
            return Err(format!("Failed to connect to NitroGen: {}", e));
        }
        Err(_) => {
            let _ = child.kill();
            let _ = godot.kill();
            return Err("Connection to NitroGen timed out".to_string());
        }
    }

    let _ = app.emit("playtest-event", PlaytestEvent {
        event_type: "connected".to_string(),
        message: "NitroGen connected! Playing game...".to_string(),
        frame: None, action: None, screenshot: None,
    });

    let mut last_frame_num = 0u32;
    let mut frame_count = 0u32;
    let mut actions_taken: Vec<String> = Vec::new();

    // Main control loop
    for step in 0..max_steps {
        if let Ok(Some(_)) = godot.try_wait() {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Find latest screenshot
        let mut latest_path: Option<PathBuf> = None;
        let mut latest_num = 0u32;
        if let Ok(entries) = fs::read_dir(&screenshots_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("frame_") && name.ends_with(".png") {
                        if let Ok(num) = name.replace("frame_", "").replace(".png", "").parse::<u32>() {
                            if num > latest_num {
                                latest_num = num;
                                latest_path = Some(entry.path());
                            }
                        }
                    }
                }
            }
        }

        if latest_num <= last_frame_num {
            continue;
        }
        last_frame_num = latest_num;
        frame_count += 1;

        let screenshot_b64 = match &latest_path {
            Some(p) if p.exists() => {
                base64::engine::general_purpose::STANDARD.encode(&fs::read(p).unwrap_or_default())
            }
            _ => continue,
        };

        // Send predict request to sidecar
        let predict_cmd = serde_json::json!({"type": "predict", "image": screenshot_b64});
        if child.write(format!("{}\n", predict_cmd).as_bytes()).is_err() {
            break;
        }

        // Read prediction response (with timeout)
        let prediction = tokio::time::timeout(tokio::time::Duration::from_secs(2), async {
            while let Some(event) = rx.recv().await {
                if let CommandEvent::Stdout(line) = event {
                    let text = String::from_utf8_lossy(&line);
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        if msg.get("type").and_then(|t| t.as_str()) == Some("prediction") {
                            return Some(msg);
                        }
                    }
                }
            }
            None
        }).await;

        if let Ok(Some(response)) = prediction {
            if response.get("error").is_some() {
                println!("[NitroGen] Error: {}", response["error"]);
                continue;
            }

            let j_left: Vec<f32> = response["j_left"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
                .unwrap_or_default();
            let j_right: Vec<f32> = response["j_right"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
                .unwrap_or_default();
            let buttons: Vec<f32> = response["buttons"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect())
                .unwrap_or_default();

            let gamepad_state = ControlMapper::parse_nitrogen_output(&j_left, &j_right, &buttons);
            let actions = mapper.map_to_actions(&gamepad_state);
            
            if let Some(action) = actions.first() {
                let action_json = serde_json::json!({"function": action.function, "args": action.args});
                fs::write(project.join("agent_input.json"), action_json.to_string()).ok();
                actions_taken.push(action.function.clone());
                
                let _ = app.emit("playtest-event", PlaytestEvent {
                    event_type: "action".to_string(),
                    message: format!("{} (L:{:.1},{:.1} R:{:.1},{:.1})", 
                        action.function, j_left.get(0).unwrap_or(&0.0), j_left.get(1).unwrap_or(&0.0),
                        j_right.get(0).unwrap_or(&0.0), j_right.get(1).unwrap_or(&0.0)),
                    frame: Some(step),
                    action: Some(action.function.clone()),
                    screenshot: None,
                });
            }
        }
    }

    // Cleanup
    let quit_cmd = serde_json::json!({"type": "quit"});
    let _ = child.write(format!("{}\n", quit_cmd).as_bytes());
    let _ = child.kill();
    let _ = godot.kill();
    let _ = godot.wait();

    // Clear sidecar PID
    if let Ok(mut guard) = sidecar_mutex.lock() {
        *guard = None;
    }

    let summary = format!("NitroGen playtest complete. {} frames, {} actions.", frame_count, actions_taken.len());
    
    let _ = app.emit("playtest-event", PlaytestEvent {
        event_type: "complete".to_string(),
        message: summary.clone(),
        frame: None, action: None, screenshot: None,
    });

    Ok(summary)
}

#[tauri::command]
async fn plan_trajectory(
    screenshot_b64: String,
    objective: String,
    game_functions: String,
    state: tauri::State<'_, AppState>,
) -> Result<Trajectory, String> {
    let settings = state.settings.lock().unwrap().clone();
    let api_key = settings.gemini_key.ok_or("Gemini API key not set")?;

    let prompt = format!(
        r#"You control a game character. Available functions:
{}

Objective: {}

Analyze the screenshot and return a sequence of 10-20 function calls to progress toward the objective.

Respond ONLY with valid JSON in this exact format:
{{"reasoning": "brief explanation of your plan", "actions": [{{"function": "function_name", "args": [arg1, arg2]}}]}}"#,
        game_functions, objective
    );

    let request_body = serde_json::json!({
        "contents": [{
            "parts": [
                {"text": prompt},
                {
                    "inlineData": {
                        "mimeType": "image/png",
                        "data": screenshot_b64
                    }
                }
            ]
        }],
        "generationConfig": {
            "temperature": 0.5,
            "thinkingConfig": {"thinkingBudget": 0}
        }
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-robotics-er-1.5-preview:generateContent?key={}",
            api_key
        ))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Extract text from Gemini response
    let text = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or("No text in response")?;

    // Parse the JSON response
    let trajectory: Trajectory = serde_json::from_str(text)
        .map_err(|e| format!("Failed to parse trajectory: {} - Response: {}", e, text))?;

    Ok(trajectory)
}

#[tauri::command]
async fn analyze_game_frame(
    screenshot_b64: String,
    prompt: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let api_key = settings.gemini_key.ok_or("Gemini API key not set. Please add your Gemini API key in Settings.")?;

    let full_prompt = format!(
        r#"You are analyzing a video game screenshot to validate and test gameplay.

User request: {}

Analyze the game scene and provide:
1. **Objects detected**: List key objects (player, NPCs, items, UI elements) with their approximate positions
2. **Scene understanding**: Describe the environment, spatial relationships, and game state
3. **Issues found**: Any visual bugs, clipping, missing elements, or unexpected behavior
4. **Validation result**: Does the scene match what was requested? What works, what doesn't?

Be specific about locations (left/right/center, foreground/background) and reference what you actually see."#,
        prompt
    );

    let request_body = serde_json::json!({
        "contents": [{
            "parts": [
                {
                    "inlineData": {
                        "mimeType": "image/png",
                        "data": screenshot_b64
                    }
                },
                {"text": full_prompt}
            ]
        }],
        "generationConfig": {
            "temperature": 0.5,
            "thinkingConfig": {"thinkingBudget": 1024}
        }
    });

    // Use robotics model for superior spatial reasoning in games
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-robotics-er-1.5-preview:generateContent?key={}",
            api_key
        ))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Check for API errors
    if let Some(error) = response_json.get("error") {
        return Err(format!("Gemini API error: {}", error));
    }

    // Extract text from Gemini response
    let text = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| format!("No text in response: {:?}", response_json))?;

    Ok(text.to_string())
}

#[tauri::command]
async fn test_game_controls(
    before_b64: String,
    after_b64: String,
    keys: Vec<String>,
    duration_ms: u32,
    prompt: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let api_key = settings.gemini_key.ok_or("Gemini API key not set")?;

    let keys_desc = keys.join(", ");
    let full_prompt = format!(
        r#"You are testing game controls. The user pressed [{keys}] for {duration}ms.

Compare these two game frames:
- BEFORE: First image (before input)
- AFTER: Second image (after input)

User request: {prompt}

Analyze:
1. **Movement detected**: Did the character/camera move? Describe the change.
2. **Animation change**: Did the character's pose or animation change?
3. **Controller working**: Based on the before/after, are the controls functioning?
4. **Issues found**: Any problems (no response, wrong direction, stuck, etc.)?

Be specific about what changed between the frames."#,
        keys = keys_desc,
        duration = duration_ms,
        prompt = prompt
    );

    let request_body = serde_json::json!({
        "contents": [{
            "parts": [
                {
                    "inlineData": {
                        "mimeType": "image/png",
                        "data": before_b64
                    }
                },
                {
                    "inlineData": {
                        "mimeType": "image/png",
                        "data": after_b64
                    }
                },
                {"text": full_prompt}
            ]
        }],
        "generationConfig": {
            "temperature": 0.5,
            "thinkingConfig": {"thinkingBudget": 2048}
        }
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-robotics-er-1.5-preview:generateContent?key={}",
            api_key
        ))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(error) = response_json.get("error") {
        return Err(format!("Gemini API error: {}", error));
    }

    let text = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| format!("No text in response: {:?}", response_json))?;

    Ok(text.to_string())
}

#[tauri::command]
async fn analyze_node_captures(
    captures: std::collections::HashMap<String, String>,
    node_name: String,
    prompt: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    let api_key = settings.gemini_key.ok_or("Gemini API key not set")?;

    // Build image parts for each angle
    let mut image_parts: Vec<serde_json::Value> = vec![];
    let mut angle_desc = String::new();
    
    for (angle, data) in &captures {
        image_parts.push(serde_json::json!({
            "inlineData": {
                "mimeType": "image/png",
                "data": data
            }
        }));
        angle_desc.push_str(&format!("- Image {}: {} view\n", image_parts.len(), angle));
    }

    let full_prompt = format!(
        r#"You are analyzing multi-angle captures of a game object called "{}".

{}

User request: {}

Analyze these views to provide:
1. **Object description**: What is this object? Describe its appearance, shape, materials/textures
2. **Texture assessment**: Are textures properly applied? Any UV mapping issues, stretching, or missing textures?
3. **Model quality**: Check for mesh issues like holes, z-fighting, normals, or LOD problems
4. **Visual consistency**: Does it look consistent from all angles? Any angle-specific issues?
5. **Recommendations**: What improvements would help this object look better?

Be specific about which angle shows each issue."#,
        node_name, angle_desc, prompt
    );

    image_parts.push(serde_json::json!({"text": full_prompt}));

    let request_body = serde_json::json!({
        "contents": [{"parts": image_parts}],
        "generationConfig": {
            "temperature": 0.5,
            "thinkingConfig": {"thinkingBudget": 2048}
        }
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            api_key
        ))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(error) = response_json.get("error") {
        return Err(format!("Gemini API error: {}", error));
    }

    let text = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| format!("No text in response: {:?}", response_json))?;

    Ok(text.to_string())
}

// ============================================================================
// Animation Library Management
// ============================================================================

#[tauri::command]
fn get_animation_catalog() -> Vec<animations::AnimationPack> {
    animations::get_animation_catalog()
}

#[tauri::command]
async fn download_animation_pack(
    pack_id: String,
    project_path: String,
) -> Result<String, String> {
    let catalog = animations::get_animation_catalog();
    let pack = catalog.iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| format!("Animation pack not found: {}", pack_id))?;
    
    let animations_dir = Path::new(&project_path).join("assets").join("animations");
    fs::create_dir_all(&animations_dir)
        .map_err(|e| format!("Failed to create animations directory: {}", e))?;
    
    match &pack.source {
        animations::AnimationSource::Url { url } => {
            // Download from URL
            let client = reqwest::Client::new();
            let response = client.get(url)
                .send()
                .await
                .map_err(|e| format!("Download failed: {}", e))?;
            
            if !response.status().is_success() {
                return Err(format!("Download failed with status: {}", response.status()));
            }
            
            let bytes = response.bytes().await
                .map_err(|e| format!("Failed to read response: {}", e))?;
            
            // Save to temp file
            let zip_path = animations_dir.join(format!("{}.zip", pack_id));
            fs::write(&zip_path, &bytes)
                .map_err(|e| format!("Failed to save zip: {}", e))?;
            
            // Extract
            let file = fs::File::open(&zip_path)
                .map_err(|e| format!("Failed to open zip: {}", e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("Failed to read zip: {}", e))?;
            
            let pack_dir = animations_dir.join(&pack_id);
            fs::create_dir_all(&pack_dir)
                .map_err(|e| format!("Failed to create pack directory: {}", e))?;
            
            archive.extract(&pack_dir)
                .map_err(|e| format!("Failed to extract: {}", e))?;
            
            // Clean up zip
            fs::remove_file(&zip_path).ok();
            
            Ok(pack_dir.to_string_lossy().to_string())
        }
        animations::AnimationSource::GitHub { repo, path } => {
            // Download from GitHub releases
            let url = format!(
                "https://github.com/{}/releases/latest/download/{}",
                repo, path
            );
            
            let client = reqwest::Client::new();
            let response = client.get(&url)
                .send()
                .await
                .map_err(|e| format!("GitHub download failed: {}", e))?;
            
            if !response.status().is_success() {
                return Err(format!("GitHub download failed: {} - URL: {}", response.status(), url));
            }
            
            let bytes = response.bytes().await
                .map_err(|e| format!("Failed to read response: {}", e))?;
            
            let zip_path = animations_dir.join(format!("{}.zip", pack_id));
            fs::write(&zip_path, &bytes)
                .map_err(|e| format!("Failed to save: {}", e))?;
            
            // Extract
            let file = fs::File::open(&zip_path)
                .map_err(|e| format!("Failed to open zip: {}", e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("Failed to read zip: {}", e))?;
            
            let pack_dir = animations_dir.join(&pack_id);
            fs::create_dir_all(&pack_dir).ok();
            archive.extract(&pack_dir).ok();
            fs::remove_file(&zip_path).ok();
            
            Ok(pack_dir.to_string_lossy().to_string())
        }
        animations::AnimationSource::Itch { page, file: _ } => {
            // Itch.io packs require manual download (user needs to visit page)
            // Return instructions for the user
            let download_url = pack.download_url.clone().unwrap_or_else(|| 
                format!("https://{}.itch.io", page.replace("/", "."))
            );
            Err(format!(
                "This animation pack is hosted on itch.io and requires manual download.\n\n\
                1. Visit: {}\n\
                2. Download the pack (it's free/CC0)\n\
                3. Extract to: {}\n\n\
                The pack will then be automatically detected.",
                download_url,
                animations_dir.join(&pack_id).to_string_lossy()
            ))
        }
        animations::AnimationSource::Bundled { asset_name } => {
            // Use bundled assets (for offline/included assets)
            Err(format!("Bundled asset '{}' not yet implemented", asset_name))
        }
    }
}

#[tauri::command]
fn setup_animation_library(
    project_path: String,
    pack_id: String,
    _target_node: Option<String>,
) -> Result<String, String> {
    let catalog = animations::get_animation_catalog();
    let pack = catalog.iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| format!("Pack not found: {}", pack_id))?;
    
    let scripts_dir = Path::new(&project_path).join("scripts");
    fs::create_dir_all(&scripts_dir)
        .map_err(|e| format!("Failed to create scripts dir: {}", e))?;
    
    // Write animation library setup script
    let lib_script_path = scripts_dir.join("animation_library_setup.gd");
    fs::write(&lib_script_path, animations::ANIMATION_LIBRARY_SETUP_GD)
        .map_err(|e| format!("Failed to write script: {}", e))?;
    
    // Write locomotion blend tree script
    let blend_script_path = scripts_dir.join("locomotion_blend_tree.gd");
    fs::write(&blend_script_path, animations::LOCOMOTION_BLEND_TREE_GD)
        .map_err(|e| format!("Failed to write blend script: {}", e))?;
    
    // Generate AnimationTree scene
    let anim_names: Vec<String> = pack.animations.iter().map(|a| a.name.clone()).collect();
    let tree_tscn = animations::generate_animation_tree_tscn(&anim_names);
    
    let scenes_dir = Path::new(&project_path).join("scenes");
    fs::create_dir_all(&scenes_dir).ok();
    let tree_path = scenes_dir.join("locomotion_tree.tscn");
    fs::write(&tree_path, tree_tscn)
        .map_err(|e| format!("Failed to write AnimationTree: {}", e))?;
    
    Ok(format!(
        "Animation library setup complete!\n\
        - Library script: scripts/animation_library_setup.gd\n\
        - Blend tree script: scripts/locomotion_blend_tree.gd\n\
        - AnimationTree scene: scenes/locomotion_tree.tscn\n\
        \n\
        Animations available: {}",
        anim_names.join(", ")
    ))
}

#[tauri::command]
fn list_project_animations(project_path: String) -> Result<Vec<String>, String> {
    let animations_dir = Path::new(&project_path).join("assets").join("animations");
    
    if !animations_dir.exists() {
        return Ok(vec![]);
    }
    
    let mut animations = Vec::new();
    
    fn scan_dir(dir: &Path, animations: &mut Vec<String>, base: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_dir(&path, animations, base);
                } else {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ["glb", "gltf", "fbx", "tres", "res"].contains(&ext) {
                        if let Ok(rel) = path.strip_prefix(base) {
                            animations.push(rel.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    scan_dir(&animations_dir, &mut animations, &animations_dir);
    Ok(animations)
}

// ============================================================================
// Input Mapping Parser
// ============================================================================

#[derive(Debug, Clone, serde::Serialize)]
struct InputMapping {
    action: String,
    keys: Vec<String>,
    description: String,
}

#[tauri::command]
fn get_input_mappings(project_path: String) -> Result<Vec<InputMapping>, String> {
    let project_file = Path::new(&project_path).join("project.godot");
    if !project_file.exists() {
        return Err("project.godot not found".to_string());
    }
    
    let content = fs::read_to_string(&project_file)
        .map_err(|e| format!("Failed to read project.godot: {}", e))?;
    
    let mut mappings = Vec::new();
    let mut in_input_section = false;
    let mut current_action: Option<String> = None;
    let mut current_block = String::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed == "[input]" {
            in_input_section = true;
            continue;
        }
        
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_input_section = false;
            continue;
        }
        
        if !in_input_section {
            continue;
        }
        
        // Check for new action definition (action_name={)
        if let Some(eq_pos) = trimmed.find("={") {
            // Save previous action if any
            if let Some(action) = current_action.take() {
                let keys = parse_keys_from_block(&current_block);
                if !keys.is_empty() {
                    mappings.push(InputMapping {
                        description: action_to_description(&action),
                        action,
                        keys,
                    });
                }
            }
            current_action = Some(trimmed[..eq_pos].to_string());
            current_block = trimmed[eq_pos..].to_string();
        } else if current_action.is_some() {
            current_block.push_str(trimmed);
        }
        
        // Check if block ends
        if current_action.is_some() && trimmed.ends_with('}') {
            if let Some(action) = current_action.take() {
                let keys = parse_keys_from_block(&current_block);
                if !keys.is_empty() {
                    mappings.push(InputMapping {
                        description: action_to_description(&action),
                        action,
                        keys,
                    });
                }
            }
            current_block.clear();
        }
    }
    
    Ok(mappings)
}

fn parse_keys_from_block(block: &str) -> Vec<String> {
    let mut keys = Vec::new();
    
    // Parse physical_keycode values using simple string search
    let search = "physical_keycode\":";
    let mut pos = 0;
    while let Some(start) = block[pos..].find(search) {
        let code_start = pos + start + search.len();
        if let Some(end) = block[code_start..].find(|c: char| !c.is_ascii_digit()) {
            if let Ok(code) = block[code_start..code_start + end].parse::<u32>() {
                if let Some(key) = keycode_to_name(code) {
                    if !keys.contains(&key) {
                        keys.push(key);
                    }
                }
            }
        }
        pos = code_start;
    }
    
    // Check for mouse buttons
    if block.contains("InputEventMouseButton") {
        if block.contains("button_index\":1") {
            keys.push("LeftClick".to_string());
        } else if block.contains("button_index\":2") {
            keys.push("RightClick".to_string());
        }
    }
    
    keys
}

fn keycode_to_name(code: u32) -> Option<String> {
    match code {
        65..=90 => Some(((code as u8) as char).to_string()), // A-Z
        32 => Some("Space".to_string()),
        16777217 => Some("Escape".to_string()),
        16777218 => Some("Tab".to_string()),
        16777220 => Some("Enter".to_string()),
        16777221 => Some("Shift".to_string()),
        16777238 => Some("Ctrl".to_string()),
        16777240 => Some("Alt".to_string()),
        4194319 => Some("Left".to_string()),
        4194320 => Some("Up".to_string()),
        4194321 => Some("Right".to_string()),
        4194322 => Some("Down".to_string()),
        _ => None,
    }
}

fn action_to_description(action: &str) -> String {
    match action {
        "move_left" => "Move character left".to_string(),
        "move_right" => "Move character right".to_string(),
        "move_up" | "move_forward" => "Move character forward".to_string(),
        "move_down" | "move_back" => "Move character backward".to_string(),
        "jump" => "Make character jump".to_string(),
        "attack" => "Attack action".to_string(),
        "interact" => "Interact with objects".to_string(),
        "sprint" | "run" => "Sprint/run faster".to_string(),
        "crouch" => "Crouch down".to_string(),
        _ => action.replace('_', " ").to_string(),
    }
}

// ============================================================================
// Settings Management
// ============================================================================

fn get_settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kobold")
        .join("settings.json")
}

fn save_settings_to_disk(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    // Merge with existing settings
    let mut existing = load_settings_from_disk();
    if settings.openrouter_key.is_some() {
        existing.openrouter_key = settings.openrouter_key.clone();
    }
    if settings.goose_model.is_some() {
        existing.goose_model = settings.goose_model.clone();
    }
    if settings.godot_path.is_some() {
        existing.godot_path = settings.godot_path.clone();
    }
    if settings.gemini_key.is_some() {
        existing.gemini_key = settings.gemini_key.clone();
    }
    
    let json = serde_json::to_string_pretty(&existing).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Failed to save settings: {}", e))?;
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_settings(settings: AppSettings, state: tauri::State<AppState>) -> Result<(), String> {
    *state.settings.lock().unwrap() = settings.clone();

    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}

fn load_settings_from_disk() -> AppSettings {
    let path = get_settings_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
    }
    AppSettings::default()
}

// ============================================================================
// OpenRouter OAuth PKCE Flow
// ============================================================================

use std::sync::atomic::{AtomicU16, Ordering as AtomicOrdering};
use sha2::{Sha256, Digest};

static OAUTH_PORT: AtomicU16 = AtomicU16::new(0);
static OAUTH_VERIFIER: Mutex<Option<String>> = Mutex::new(None);

fn generate_code_verifier() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&result)
}

#[tauri::command]
async fn start_openrouter_auth(app: tauri::AppHandle) -> Result<(), String> {
    // Generate PKCE codes
    let verifier = generate_code_verifier();
    let challenge = generate_code_challenge(&verifier);
    
    // Store verifier for later exchange
    *OAUTH_VERIFIER.lock().unwrap() = Some(verifier);
    
    // Find available port
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind: {}", e))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    OAUTH_PORT.store(port, AtomicOrdering::SeqCst);
    drop(listener);
    
    // Start callback server in background
    let app_clone = app.clone();
    std::thread::spawn(move || {
        if let Err(e) = run_oauth_callback_server(port, app_clone) {
            eprintln!("OAuth callback server error: {}", e);
        }
    });
    
    // Open browser
    let callback_url = format!("http://127.0.0.1:{}", port);
    let auth_url = format!(
        "https://openrouter.ai/auth?callback_url={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(&callback_url),
        urlencoding::encode(&challenge)
    );
    
    open::that(&auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;
    
    Ok(())
}

fn run_oauth_callback_server(port: u16, app: tauri::AppHandle) -> Result<(), String> {
    let server = tiny_http::Server::http(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("Failed to start server: {}", e))?;
    
    // Wait for callback (with timeout)
    let timeout = std::time::Duration::from_secs(300); // 5 minute timeout
    
    if let Ok(Some(request)) = server.recv_timeout(timeout) {
        let url = request.url().to_string();
        
        // Parse the code from URL
        if let Some(code) = url.split("code=").nth(1).map(|s| s.split('&').next().unwrap_or(s)) {
            // Exchange code for API key
            let verifier = OAUTH_VERIFIER.lock().unwrap().take();
            
            if let Some(verifier) = verifier {
                // Spawn async task to exchange code
                let code = code.to_string();
                let app_clone = app.clone();
                
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match exchange_code_for_key(&code, &verifier).await {
                            Ok(api_key) => {
                                // Save the key
                                let settings = AppSettings {
                                    openrouter_key: Some(api_key),
                                    ..Default::default()
                                };
                                let _ = save_settings_to_disk(&settings);
                                let _ = app_clone.emit("oauth-success", ());
                            }
                            Err(e) => {
                                let _ = app_clone.emit("oauth-error", e);
                            }
                        }
                    });
                });
            }
            
            // Respond with success page
            let response = tiny_http::Response::from_string(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Sign In Complete</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; background: #0a0a0a; color: #fff; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
        .container { text-align: center; }
        h1 { font-size: 24px; margin-bottom: 8px; }
        p { color: #888; font-size: 14px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>✓ Signed In</h1>
        <p>You can close this window and return to the app.</p>
    </div>
</body>
</html>"#
            ).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap()
            );
            let _ = request.respond(response);
        } else {
            // Error response
            let response = tiny_http::Response::from_string("Authorization failed")
                .with_status_code(400);
            let _ = request.respond(response);
        }
    }
    
    Ok(())
}

async fn exchange_code_for_key(code: &str, verifier: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post("https://openrouter.ai/api/v1/auth/keys")
        .json(&serde_json::json!({
            "code": code,
            "code_verifier": verifier,
            "code_challenge_method": "S256"
        }))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }
    
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    data.get("key")
        .and_then(|k| k.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No key in response".to_string())
}

// ============================================================================
// Thread Persistence
// ============================================================================

fn get_threads_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kobold")
        .join("threads.json")
}

#[tauri::command]
fn save_threads(threads: serde_json::Value) -> Result<(), String> {
    let path = get_threads_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&threads).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Failed to save threads: {}", e))?;
    Ok(())
}

#[tauri::command]
fn load_threads() -> serde_json::Value {
    let path = get_threads_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(threads) = serde_json::from_str(&content) {
                return threads;
            }
        }
    }
    serde_json::json!([])
}

// ============================================================================
// Agent Communication with Streaming
// ============================================================================

#[tauri::command]
async fn send_agent_message(
    app: tauri::AppHandle,
    message: String,
    project_path: Option<String>,
    _continue_session: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.lock().unwrap().clone();
    
    let working_dir = match &project_path {
        Some(path) if !path.is_empty() && Path::new(path).exists() => path.clone(),
        _ => {
            return Ok("**No Project Open**\n\nPlease open a Godot project folder first using the folder icon in the app bar.".to_string());
        }
    };

    let _ = ensure_project_config(&working_dir);

    // Check for OpenRouter API key
    let api_key = settings.openrouter_key.clone().unwrap_or_default();
    if api_key.is_empty() {
        return Ok("**Sign In Required**\n\nPlease sign in with OpenRouter in Settings to continue.".to_string());
    }

    // Check if Goose is installed (internal check, no branding shown)
    if !detect_goose() {
        return Ok("**Agent Setup Required**\n\nThe AI agent is not installed. Please install it and restart the app.\n\nVisit: https://github.com/block/goose".to_string());
    }

    // Auto-initialize Beads for task tracking if available
    if detect_beads() {
        let beads_dir = Path::new(&working_dir).join(".beads");
        if !beads_dir.exists() {
            let _ = init_beads(working_dir.clone());
        }
    }

    let _ = app.emit("agent-event", AgentEvent {
        event_type: "start".to_string(),
        content: format!("Working in: {}", working_dir),
        tool_name: None,
        tool_args: None,
    });

    // Get Beads context to inject into the message
    let beads_context = if detect_beads() {
        get_beads_context(working_dir.clone()).ok()
    } else {
        None
    };

    // Prepare message with Beads context
    let enhanced_message = if let Some(ctx) = beads_context {
        if !ctx.trim().is_empty() {
            format!("{}\n\n---\nTask Context:\n{}", message, ctx)
        } else {
            message.clone()
        }
    } else {
        message.clone()
    };

    let result = run_goose(&app, &enhanced_message, &working_dir, &settings).await;

    let _ = app.emit("agent-event", AgentEvent {
        event_type: "done".to_string(),
        content: "".to_string(),
        tool_name: None,
        tool_args: None,
    });

    result
}

async fn run_goose(
    app: &tauri::AppHandle,
    message: &str,
    working_dir: &str,
    settings: &AppSettings,
) -> Result<String, String> {
    let working_path = Path::new(working_dir);
    if !working_path.exists() {
        return Err(format!("Working directory does not exist: {}", working_dir));
    }
    
    let abs_working_dir = working_path.canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?
        .to_string_lossy()
        .to_string();

    // Build goose command with OpenRouter configuration
    let mut cmd = Command::new("goose");
    cmd.args(["run", "--text", message])
        .current_dir(&abs_working_dir)
        .env("GOOSE_PROVIDER", "openrouter");

    // Set model (default to claude-sonnet-4-20250514 if not specified)
    let model = settings.goose_model.clone()
        .unwrap_or_else(|| "anthropic/claude-sonnet-4-20250514".to_string());
    cmd.env("GOOSE_MODEL", &model);

    // Set OpenRouter API key
    if let Some(key) = &settings.openrouter_key {
        if !key.is_empty() {
            cmd.env("OPENROUTER_API_KEY", key);
        }
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start Goose in {}: {}", abs_working_dir, e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let mut full_output = String::new();

    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        if let Ok(line) = line {
            // Check for tool-related output patterns
            if line.contains("tool_use") || line.contains("Reading") || line.contains("Writing") || line.contains("executing") {
                let _ = app.emit("agent-event", AgentEvent {
                    event_type: "tool_start".to_string(),
                    content: line.clone(),
                    tool_name: Some(extract_tool_name(&line)),
                    tool_args: None,
                });
            } else if line.contains("tool_result") || line.contains("Created") || line.contains("Updated") || line.contains("completed") {
                let _ = app.emit("agent-event", AgentEvent {
                    event_type: "tool_end".to_string(),
                    content: line.clone(),
                    tool_name: None,
                    tool_args: None,
                });
            } else if !line.trim().is_empty() {
                let _ = app.emit("agent-event", AgentEvent {
                    event_type: "output".to_string(),
                    content: format!("{}\n", line),
                    tool_name: None,
                    tool_args: None,
                });
            }
            full_output.push_str(&line);
            full_output.push('\n');
        }
    }

    // Read stderr
    let stderr_reader = BufReader::new(stderr);
    for line in stderr_reader.lines() {
        if let Ok(line) = line {
            // Filter out noise, only emit actual errors
            if !line.trim().is_empty() && !line.contains("Loading") {
                let _ = app.emit("agent-event", AgentEvent {
                    event_type: "error".to_string(),
                    content: line.clone(),
                    tool_name: None,
                    tool_args: None,
                });
            }
        }
    }

    let _ = child.wait();

    if full_output.trim().is_empty() {
        Ok("Goose completed the task.".to_string())
    } else {
        Ok(full_output)
    }
}

fn extract_tool_name(line: &str) -> String {
    if let Some(start) = line.find("Tool:") {
        let rest = &line[start + 5..];
        rest.split_whitespace().next().unwrap_or("unknown").to_string()
    } else if let Some(start) = line.find("Running:") {
        let rest = &line[start + 8..];
        rest.split_whitespace().next().unwrap_or("command").to_string()
    } else if line.contains("Reading") {
        "read_file".to_string()
    } else if line.contains("Writing") {
        "write_file".to_string()
    } else {
        "tool".to_string()
    }
}

// ============================================================================
// File Watcher for Live Preview
// ============================================================================

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static WATCHER_ACTIVE: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn start_file_watcher(project_path: String, app: tauri::AppHandle) -> Result<(), String> {
    use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
    use std::thread;
    
    // Don't start if already watching
    if WATCHER_ACTIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    
    WATCHER_ACTIVE.store(true, Ordering::SeqCst);
    
    let path = PathBuf::from(&project_path);
    
    thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        
        let mut debouncer = match new_debouncer(Duration::from_millis(500), tx) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to create file watcher: {}", e);
                WATCHER_ACTIVE.store(false, Ordering::SeqCst);
                return;
            }
        };
        
        if let Err(e) = debouncer.watcher().watch(&path, RecursiveMode::Recursive) {
            eprintln!("Failed to watch path: {}", e);
            WATCHER_ACTIVE.store(false, Ordering::SeqCst);
            return;
        }
        
        println!("[FileWatcher] Watching: {}", path.display());
        
        while WATCHER_ACTIVE.load(Ordering::SeqCst) {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(Ok(events)) => {
                    // Filter to only relevant file changes
                    let relevant: Vec<_> = events
                        .iter()
                        .filter(|e| {
                            let p = e.path.to_string_lossy();
                            // Ignore hidden files, .tav folder, and export_presets
                            !p.contains("/.") && 
                            !p.contains("\\.") && 
                            !p.contains(".tav") &&
                            !p.ends_with("export_presets.cfg") &&
                            // Only watch relevant file types
                            (p.ends_with(".gd") || 
                             p.ends_with(".tscn") || 
                             p.ends_with(".tres") ||
                             p.ends_with(".png") ||
                             p.ends_with(".jpg") ||
                             p.ends_with(".wav") ||
                             p.ends_with(".ogg") ||
                             p.ends_with(".godot"))
                        })
                        .collect();
                    
                    if !relevant.is_empty() {
                        let changed_files: Vec<String> = relevant
                            .iter()
                            .map(|e| e.path.to_string_lossy().to_string())
                            .collect();
                        
                        println!("[FileWatcher] Changes detected: {:?}", changed_files);
                        
                        // Emit event to frontend
                        let _ = app.emit("project-files-changed", changed_files);
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("[FileWatcher] Error: {:?}", e);
                }
                Err(_) => {
                    // Timeout, continue loop
                }
            }
        }
        
        println!("[FileWatcher] Stopped");
    });
    
    Ok(())
}

#[tauri::command]
fn stop_file_watcher() {
    WATCHER_ACTIVE.store(false, Ordering::SeqCst);
}

// ============================================================================
// Application Entry
// ============================================================================

fn main() {
    let initial_settings = load_settings_from_disk();

    tauri::Builder::default()
        .manage(AppState {
            settings: Mutex::new(initial_settings),
            game_sessions: Mutex::new(std::collections::HashMap::new()),
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_files,
            read_file,
            write_file,
            delete_file,
            run_godot,
            export_project_web,
            ensure_export_templates,
            check_setup_status,
            open_url,
            download_asset,
            download_and_extract_asset,
            check_asset_exists,
            setup_3d_character,
            start_preview_server,
            start_file_watcher,
            stop_file_watcher,
            get_settings,
            save_settings,
            save_threads,
            load_threads,
            start_openrouter_auth,
            detect_beads,
            install_beads,
            init_beads,
            get_beads_context,
            detect_godot,
            install_godot,
            detect_godot_mcp,
            install_godot_mcp,
            setup_godot_mcp_config,
            create_project_from_template,
            initialize_godot_project,
            open_url,
            send_agent_message,
            start_game_session,
            get_game_frame,
            send_game_action,
            execute_actions,
            stop_game_session,
            plan_trajectory,
            analyze_game_frame,
            test_game_controls,
            analyze_node_captures,
            get_input_mappings,
            clear_export_cache,
            get_animation_catalog,
            download_animation_pack,
            setup_animation_library,
            list_project_animations,
            run_playtest,
            check_nitrogen_installed,
            start_nitrogen_server,
            stop_nitrogen_server,
            get_control_mappings,
            save_control_mappings,
            run_playtest_nitrogen
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
