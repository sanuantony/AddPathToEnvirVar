# 📦 pathctl

[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

A safe, minimal CLI tool to manage your Windows PATH environment variable.

Unlike `setx` or manual registry edits, **pathctl** focuses on:

- **Safety** — automatic backups + confirmation prompts before every change
- **Correctness** — path normalization and case-insensitive duplicate detection
- **Convenience** — supports `.`, `~`, `%VAR%` expansion out of the box

## ✨ Features

- **Add / remove** entries from PATH
- **User and system** PATH support (`--scope user` or `--scope system`)
- **Automatic backup** before every mutation (saved to `%TEMP%`)
- **Dry-run mode** to preview changes without writing
- **Confirmation prompt** (skip with `--yes`)
- **Path resolution:**
  - `.` → current directory
  - `..` → parent directory
  - `~` → home directory (`%USERPROFILE%`)
  - `%VAR%` → Windows environment variables
- **Duplicate prevention** (case-insensitive comparison on Windows)
- **Environment broadcast** — notifies running applications via `WM_SETTINGCHANGE`

## 📋 Requirements

- **Rust 1.70+** (edition 2021)
- **Windows** (currently Windows-only)

## 📥 Installation

Build from source (dependencies are downloaded automatically):

```sh
cargo build --release
```

Binary will be at:

```
target\release\pathctl.exe
```

(Optional) Add it to your PATH:

```sh
pathctl add target\release
```

## 🚀 Usage

### List PATH

```sh
pathctl list
pathctl list --scope system
```

### Add a path

```sh
pathctl add "C:\Tools\bin"
pathctl add .                    # current directory
pathctl add ~                    # home directory
pathctl add %USERPROFILE%\bin
```

System PATH (requires Administrator):

```sh
pathctl add "C:\Tools\bin" --scope system
```

### Remove a path

```sh
pathctl remove "C:\Tools\bin"
pathctl remove .
```

### Backup PATH

```sh
pathctl backup backup.txt
```

### Restore PATH

```sh
pathctl restore backup.txt
```

## ⚙️ Options

| Option           | Description                                  |
|------------------|----------------------------------------------|
| `--scope user`   | Target user PATH (default)                   |
| `--scope system` | Target system PATH (requires Administrator)  |
| `--dry-run`      | Preview changes without modifying PATH       |
| `--yes`          | Skip confirmation prompt                     |

## 🧪 Examples

Preview adding a path:

```sh
pathctl add C:\Test --dry-run
```

Add without confirmation:

```sh
pathctl add C:\Test --yes
```

Remove from system PATH:

```sh
pathctl remove C:\Test --scope system --yes
```

## 🔐 Safety Guarantees

Before modifying PATH, **pathctl**:

1. Validates the input path exists and is a directory
2. Normalizes the path for comparison
3. Prevents duplicate entries
4. Creates a timestamped backup in `%TEMP%`
5. Prompts for confirmation (unless `--yes`)
6. Writes to the registry
7. Broadcasts the change to Windows

## ⚠️ Notes

- **System PATH** requires Administrator privileges
- Changes apply to **new processes** only — existing terminals may need to be restarted
- Backup files are written in semicolon-delimited format (native Windows PATH format)

## 🧠 Why not `setx`?

`setx`:

- Can **truncate** PATH (1024-character limit)
- **Overwrites** instead of merging safely
- Provides **no backup** mechanism

**pathctl** avoids all of that.

## 📁 Project Structure

```
src/
  main.rs        # CLI entry point and command orchestration
  cli.rs         # Argument parsing (clap derive)
  core.rs        # Path logic, normalization, backup/restore
  windows.rs     # Registry read/write and WM_SETTINGCHANGE broadcast
```

### Dependencies

| Crate          | Purpose                                |
|----------------|----------------------------------------|
| `clap`         | CLI argument parsing (derive macros)   |
| `winreg`       | Windows registry access                |
| `windows-sys`  | `SendMessageTimeoutW` for env broadcast|

##  Future Ideas

- JSON output (`--json`)
- PATH diff preview
- Cross-platform support (Linux/macOS)
- Deduplicate existing PATH entries
- Reorder entries