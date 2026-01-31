#!/usr/bin/env python3
"""
Build script to create NitroGen sidecar executable using PyInstaller
Run: python build-sidecar.py
"""
import subprocess
import sys
import platform
import shutil
from pathlib import Path

def get_target_triple():
    """Get Rust-style target triple for current platform"""
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "windows":
        if machine in ("amd64", "x86_64"):
            return "x86_64-pc-windows-msvc"
        return "i686-pc-windows-msvc"
    elif system == "darwin":
        if machine == "arm64":
            return "aarch64-apple-darwin"
        return "x86_64-apple-darwin"
    elif system == "linux":
        if machine in ("aarch64", "arm64"):
            return "aarch64-unknown-linux-gnu"
        return "x86_64-unknown-linux-gnu"
    return "unknown"

def main():
    script_dir = Path(__file__).parent
    script_path = script_dir / "nitrogen-sidecar.py"
    
    if not script_path.exists():
        print(f"Error: {script_path} not found")
        sys.exit(1)
    
    target = get_target_triple()
    ext = ".exe" if platform.system() == "windows" else ""
    output_name = f"nitrogen-sidecar-{target}{ext}"
    
    print(f"Building sidecar for {target}...")
    
    # Check PyInstaller
    try:
        import PyInstaller
    except ImportError:
        print("Installing PyInstaller...")
        subprocess.run([sys.executable, "-m", "pip", "install", "pyinstaller"], check=True)
    
    # Build with PyInstaller
    cmd = [
        sys.executable, "-m", "PyInstaller",
        "--onefile",
        "--name", f"nitrogen-sidecar-{target}",
        "--distpath", str(script_dir),
        "--workpath", str(script_dir / "build"),
        "--specpath", str(script_dir / "build"),
        "--clean",
        "--noconfirm",
        str(script_path)
    ]
    
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=script_dir)
    
    if result.returncode == 0:
        output_path = script_dir / output_name
        if output_path.exists():
            print(f"✓ Built: {output_path}")
        else:
            print(f"✓ Build complete. Check {script_dir} for output.")
    else:
        print("✗ Build failed")
        sys.exit(1)
    
    # Cleanup build artifacts
    build_dir = script_dir / "build"
    if build_dir.exists():
        shutil.rmtree(build_dir)

if __name__ == "__main__":
    main()
