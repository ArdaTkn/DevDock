# DevDock Architecture

This document explains how DevDock is put together: the layers, the flow of
data, the security boundaries, and the conventions any contributor needs to know.

## High-level layering

DevDock is a Tauri 2 desktop app: a **React** renderer driven by a **Rust** core.

```
┌─────────────────────────────────┐
│          React UI               │  Vite + React + TS (renderer)
│  components · pages · stores    │  never touches fs, never spawns procs
└──────────────┬──────────────────┘
               │  Tauri IPC — typed commands + events
┌──────────────▼──────────────────┐
│           Rust Core             │  tokio async workers
│                                 │
│ discovery/   async scanner +    │
│              detector registry  │
│ git/         read-only git CLI  │
│ system/      editor/terminal/   │
│              file-manager open  │
│ storage/     SQLite (Mutex)     │
└──────────────┬──────────────────┘
        ┌──────▼───────┐
        │    SQLite    │   metadata + paths only
        └──────────────┘
```

### The hard rule

The **renderer never performs privileged work**. It calls typed Tauri commands
and renders the results. All filesystem access, process spawning, Git calls,
and Docker calls live in the Rust core. This keeps a clean security boundary:
even a compromised/malicious renderer cannot do arbitrary filesystem or command
work outside the narrow, explicit command surface.

## The Rust core

Source: `src-tauri/src/`

### `lib.rs`
- Initialises `tracing` (structured logging).
- Builds the Tauri app, opens the SQLite DB into `AppDataDir`, manages
  `AppState`, and registers all commands.

### `commands.rs`
- `AppState { db: AppDb, scan_handle }` — the global state.
- Every `#[tauri::command]` is a **thin** RPC handler: parse args, call the
  relevant module, return a serde DTO or `Err(ErrorDto)`.
- `scan_projects` runs the scan on a blocking task (`spawn_blocking`) so the
  async runtime / UI never blocks.

### `error.rs`
- One `Error` enum (`thiserror`) mapped to a human-readable `ErrorDto
  { message, hint }` that the UI renders. **A failure in one project never
  aborts the whole scan or app.** Errors are per-domain and per-project.

### `fs.rs`
- Path safety helpers: canonicalisation, "is under" checks, ignored-dir list,
  name extraction. Symlinked dirs that escape a scan root are skipped.

### `discovery/`
- `detector.rs` — the `ProjectDetector` trait + `DetectorRegistry`.
  A directory is a project if the union of detector results is non-empty.
- `detectors/*` — one file per ecosystem. Each detector inspects only its
  marker files (never source code) and returns `Option<Vec<Tech>>`.
- `scanner.rs` — bounded-depth async walker. Collects project roots, then
  ingests each: detects techs, computes git info, dir size, mtime, and upserts
  into SQLite. Per-project errors are swallowed so one broken repo can't stop
  the scan. Supports cancellation via `ScanHandle`.

### `git/`
- `GitCommand`: runs `git -C <dir> …` with explicit argv (no shell → no
  injection). Read-only in the MVP. Parses `--porcelain` output, last commit,
  remote URL. Detects a missing `git` binary and reports it gracefully.

### `storage/db.rs`
- `AppDb` wraps the SQLite `Connection` in a `Mutex` so the struct is
  `Send + Sync` (required by Tauri's `manage`). Migrations run on startup.
- Schema: `scan_locations`, `projects`, `project_techs`, `git_metadata`,
  `recent_projects`, `settings`. **Only paths + metadata — never source code.**

### `storage/project_repo.rs`
- All SQL for projects/techs/git/locations. Each function locks the connection
  once (`let conn = db.conn()`); helper queries take `&Connection` to avoid
  recursive locking.

### `system/`
- `SystemActions`: open folder (file manager), open terminal (macOS
  Terminal.app), open editor (detect VS Code / Cursor / Zed / JetBrains).
  Later these move behind a per-OS trait for Windows/Linux parity.

## The frontend

Source: `src/`

- `main.tsx` — React boot.
- `App.tsx` — boot-loads the project list; shows `Onboarding` when there are no
  scan locations, else the `Layout` (sidebar + routed pages).
- `pages/` — `Dashboard`, `ProjectDetail`, `Settings`, `Onboarding`.
- `components/` — `ProjectCard` (+ future `StatusBadge`, `CommandPalette`).
- `stores/` — **Zustand**: `projectsStore`, `scanStore`, `systemStore`.
- `services/api.ts` — thin typed wrappers over `invoke("…")`. No business logic
  here; all of it lives in Rust.
- `lib/format.ts` — pure formatting helpers (time-ago, sizes, git state).

### State flow
1. `App` calls `load()` → `list_projects` + `list_scan_locations` fill the store.
2. `Dashboard` derives filtered/sorted lists via `useMemo` from the store.
3. Actions (favorite, scan, open) call `services/api` → Rust command → store
   refresh. The store is the single source of truth for UI.

## Data / IPC types

Rust DTOs in `models.rs` are mirrored 1:1 in `src/types.ts` (`Project`, `GitInfo`,
`Tech`, `ScanLocation`, `ScanSummary`). Keep them in sync when changing either.

## Concurrency & performance

- Scans run on `spawn_blocking` (CPU/IO work) so the UI stays responsive.
- The walker is bounded (max depth ~5) and skips ignored dirs, so it degrades
  gracefully even with huge trees; per-project work is isolated.
- `dir_size` is capped (~200k entries) to avoid runaway scans.
- Data is cached in SQLite; full rescans replace per-project metadata idempotently.

## Security boundaries

See [SECURITY.md](SECURITY.md). Core principles: no auto-execution, explicit
argv (no shell), no elevated privileges, path validation, no source upload,
no telemetry.

## Adding a subsystem

1. Create the module + re-export from `lib.rs`.
2. Expose any UI-needed behaviour as a `#[tauri::command]` in `commands.rs`
   and a typed wrapper in `services/api.ts`.
3. Add tests under `src-tauri/tests/` (or `#[cfg(test)]` inline).
4. Document it in this file and/or `docs/`.
