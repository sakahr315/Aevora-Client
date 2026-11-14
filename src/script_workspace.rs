//! Local Luau script discovery and desktop integration helpers.
//!
//! This module deliberately launches the standalone offline Luau runner. It
//! does not attach to, inject into, or automate a live Roblox game session.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn contains_project_files(path: &Path) -> bool {
    if !path.join("scripts").is_dir() {
        return false;
    }
    let has_manifest = path.join("Cargo.toml").is_file();
    let runner_name = if cfg!(windows) { "aevora-luau.exe" } else { "aevora-luau" };
    let has_bundled_runner = [
        path.join("tools/aevora-luau/target/release").join(runner_name),
        path.join("tools/aevora-luau/target/debug").join(runner_name),
    ]
    .iter()
    .any(|candidate| candidate.is_file());
    has_manifest || has_bundled_runner
}

/// Resolve the repository root when running from a Cargo build tree or from
/// the repository working directory.
pub fn project_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        for ancestor in exe.ancestors() {
            if contains_project_files(ancestor) {
                return ancestor.to_path_buf();
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            if contains_project_files(ancestor) {
                return ancestor.to_path_buf();
            }
        }
        return cwd;
    }

    PathBuf::from(".")
}

pub fn scripts_dir() -> PathBuf {
    project_root().join("scripts")
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(&path, files)?;
        } else if kind.is_file() {
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if extension == "lua" || extension == "luau" {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// Recursively list local Luau scripts. Missing scripts/ is a normal empty
/// state, not a startup error.
pub fn discover_scripts() -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect(&scripts_dir(), &mut files)?;
    files.sort();
    Ok(files)
}

pub fn display_path(path: &Path) -> String {
    path.strip_prefix(scripts_dir())
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Find the standalone runner built by BUILD_AEVORA.bat or Cargo.
pub fn runner_path() -> Option<PathBuf> {
    let root = project_root();
    let executable = if cfg!(windows) {
        "aevora-luau.exe"
    } else {
        "aevora-luau"
    };
    [
        root.join("tools/aevora-luau/target/release").join(executable),
        root.join("tools/aevora-luau/target/debug").join(executable),
        root.join("target/release").join(executable),
        root.join("target/debug").join(executable),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

/// Open the selected script in the platform's default text editor.
pub fn open_in_editor(path: &Path) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("notepad.exe");
        command.arg(path);
        command
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };

    command.spawn().map(|_| ())
}

/// Open the local scripts directory in the platform file manager.
pub fn open_scripts_folder() -> io::Result<()> {
    let directory = scripts_dir();
    fs::create_dir_all(&directory)?;

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer.exe");
        command.arg(&directory);
        command
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(&directory);
        command
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(&directory);
        command
    };

    command.spawn().map(|_| ())
}
