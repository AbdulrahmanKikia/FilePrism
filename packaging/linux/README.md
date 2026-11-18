# Linux shell actions (Nautilus / Dolphin / Thunar)

Install the desktop entry so file managers expose **Convert / Compress / Extract** actions.

```bash
# From the repo root (or after packaging installs the same file under /usr/share/applications)
mkdir -p ~/.local/share/applications
cp packaging/linux/fileprism.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
```

Ensure `fileprism` is on `PATH` (or edit `Exec=` to the absolute binary path from a Tauri install).

Actions invoke:

```text
fileprism --shell-open <convert|compress|extract> <paths…>
```

The desktop app parses `--shell-open` at startup (same schema as the Windows named-pipe IPC).
