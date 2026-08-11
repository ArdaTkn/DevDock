# Security Model

DevDock executes local commands and reads local filesystem metadata, so
security is a first-class concern. This document states the model explicitly.

## Threat model

The realistic threats are:

1. A maliciously crafted **project tree** on disk (e.g. a repo whose
   `package.json` or `.git` content tries to abuse DevDock).
2. A compromised / malicious **renderer** (React code) trying to do arbitrary
   filesystem or command work on the host.
3. Accidental **data leakage** of source code or secrets.

## Principles

### 1. No auto-execution
DevDock never runs project commands, scripts, or binaries on its own. Every
open/terminal/editor action is **user-initiated**.

### 2. No shell → no injection
All external commands run with **explicit argv**:
```rust
Command::new("git").arg("-C").arg(dir).args(["status", "--porcelain", "-b"])
```
There is no `sh -c` and no string interpolation into a shell. Project-provided
strings (paths, remote URLs) never become shell syntax.

### 3. Renderer is not privileged
The React renderer performs no filesystem access and spawns no processes. It
only calls a narrow, typed set of Tauri commands. Even a fully compromised
renderer is confined to that command surface (which is read-only for project
data and user-gated for open actions).

### 4. Path validation
- Scan roots are canonicalised at add-time.
- Only directories under a user-chosen scan root are scanned.
- Symlinked directories are not followed during walks, so a link cannot silently
  escape the scan root.
- Hidden/ignored dirs (`node_modules`, `target`, `.git`, build dirs) are skipped.

### 5. No elevated privileges
Normal operation never requests admin/root. DevDock runs as the current user.

### 6. Privacy / no source upload
- **Local-first:** core functionality is fully offline. No account, no cloud,
  no backend.
- The scanner reads **only**: directory listings, small marker files
  (e.g. `package.json` `scripts`/dependency names), Git status (read-only), and
  file sizes/mtimes. It does **not** read source code, `.env`, or secrets.
- **No telemetry by default.** Nothing is sent anywhere.

### 7. Logging hygiene
Structured logs are filtered to never contain environment-variable values, API
keys, passwords, or project source content.

## Command surface (MVP)

| Command | Action | Requires user gesture? |
|---------|--------|------------------------|
| `list_projects` / `get_project` | Read DB | no (read-only) |
| `scan_projects` / `cancel_scan` | Walk user dirs | yes (Rescan button) |
| `add/remove_scan_location` | Manage roots | yes (user adds path) |
| `set_favorite`, `list_recent` | Local metadata | no |
| `open_project_editor` | Launch editor | yes (button) |
| `open_project_terminal` | Launch terminal | yes (button) |
| `open_project_folder` | Open file manager | yes (button) |

None of these can modify a project's contents or run project code automatically.

## Reporting

See [SECURITY.md](../SECURITY.md) at the repo root for the reporting policy.
