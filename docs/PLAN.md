# DevDock — Technical Implementation Plan

> Your local development command center.

This document is the product/architecture plan for DevDock. It is the authoritative
reference for why the project is shaped the way it is and how it is built. It precedes
all implementation.

Status: **v0.1 plan — approved for Phase 2 (MVP bootstrap)**

---

## 1. Product architecture

DevDock is a **local-first desktop shell** with a Rust core and a React UI, bridged by Tauri.

```
┌───────────────────────────────┐
│          React UI             │   Vite + React + TS (renderer)
└──────────────┬────────────────┘
               │  Tauri IPC (commands + events)
┌──────────────▼────────────────┐
│           Rust Core           │   tokio async workers
│                               │
│  discovery/   project scanner │
│  git/         git metadata    │
│  processes/   process & port  │
│  docker/      container info  │
│  watch/       fs events       │
│  storage/     SQLite          │
│  security/    path + cmd guard│
└──────────────┬────────────────┘
        ┌──────▼───────┐
        │    SQLite    │   metadata + paths only (never source code)
        └──────────────┘
```

Responsibilities are strictly separated:

- **Renderer** never touches the filesystem, never spawns processes, never runs git.
  It only calls typed Tauri commands and receives typed results/events.
- **Rust core** owns all privileged/system/IO work and returns serde-serialisable DTOs.
- **SQLite** stores only paths and metadata — never project file contents.

The UI is a *view* over data the Rust core already computed and cached. Nothing
expensive happens on the JS thread.

---

## 2. Technology stack

| Layer        | Choice                            | Why |
|--------------|-----------------------------------|-----|
| Desktop shell| Tauri 2                           | Small binary (~10 MB), Rust core, low RAM vs Electron, per-spec |
| Frontend     | React 18 + TypeScript (strict)    | Maintainable, typed, huge ecosystem |
| Build        | Vite                             | Fast dev, native TS/ESM, Tauri's default blessing |
| State        | Zustand                          | Tiny (~1 KB), minimal boilerplate, no reducer ceremony |
| Routing      | React Router                     | Simple, well-known, enough for pages + project detail routes |
| Styling      | Plain CSS + CSS variables        | No heavy component lib → keeps it lean, matches dark premium look |
| Backend      | Rust (edition 2021)              | Per-spec; safe, fast, single-binary, no GC pauses during scans |
| Async        | tokio                            | Async scanning/watching without blocking UI or each other |
| SQLite       | rusqlite (bundled)               | Embedded, zero external service, structured persistent state |
| File watch   | notify crate                     | Cross-platform fs events for incremental refresh |
| Git          | `git` CLI via `std::process`     | Reliable, no native-lib compile cost, trivially graceful when missing |
| Docker       | `docker` CLI via `std::process`  | Same reasoning; fully optional at runtime |
| Errors       | `thiserror` + a single `Error`   | Human-readable messages surfaced to UI, no panic on one bad project |
| Logging      | `tracing`                        | Structured, cheap, filterable; no secrets ever logged |
| Testing      | Rust `cargo test` + Vitest       | Detector/git/health unit tests in Rust; UI tests in Vitest + Testing Library |

Deliberate omissions for the MVP: no AI, no auth, no cloud, no network sync,
no microservices, no libgit2 (CLI is simpler and more robust to edge cases),
no heavy UI component library.

---

## 3. Why each choice (short)

- **Tauri over Electron**: ~10× smaller install, dramatically lower RAM, native Rust
  for fs/process work, and the exact "local command center" weight class we want.
- **Rust CLI for git/docker over native libs**: `git`/`docker` are already installed on
  every dev machine we target. Shelling out is robust, keeps compile times and binary
  size down, and makes "graceful when missing" trivial (command not found → clear error
  instead of a crash or a hard dependency).
- **Zustand over Redux**: we have 2–3 stores (projects, scan-progress, settings). Redux
  is ceremony we don't need.
- **Plain CSS over Tailwind/shadcn**: the product identity is "dark, compact,
  information-dense, minimal animation". A component library drags in SaaS-dashboard
  aesthetics and bundle weight. Custom CSS gives the exact visual language we want.

---

## 4. Folder structure

```
devdock/
├── src/                      # React renderer
│   ├── components/           # ProjectCard, ProjectRow, StatusBadge, ...
│   ├── pages/                # Dashboard, ProjectDetail, Settings, Onboarding
│   ├── hooks/                # useProjects, useScanProgress, useSettings
│   ├── stores/               # Zustand stores (projects, scan, settings, ui)
│   ├── services/             # typed Tauri command wrappers (thin, no logic)
│   ├── types/                # shared TS types mirroring Rust DTOs
│   ├── App.tsx
│   └── main.tsx
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # entry, tauri builder
│   │   ├── lib.rs            # command registration, app state
│   │   ├── commands/         # typed IPC handlers (project, git, scan, system)
│   │   ├── discovery/        # scanner orchestration
│   │   │   ├── mod.rs
│   │   │   ├── scanner.rs    # async walker, incremental
│   │   │   ├── detector.rs   # ProjectDetector trait + registry
│   │   │   └── detectors/    # git, node, python, rust, go, flutter,
│   │   │                     #   dotnet, docker, unity, (java)
│   │   ├── git/              # git CLI wrapper + porcelain parsing
│   │   ├── processes/        # process + listening-port scanner (trait, per-OS)
│   │   ├── docker/           # docker CLI wrapper
│   │   ├── fs/               # safe path helpers, symlink guard
│   │   ├── watch/            # notify-based watcher + select refresh
│   │   ├── health/           # deterministic health checks
│   │   ├── storage/          # rusqlite schema + migrations + repos
│   │   └── error.rs          # thiserror Error → serde message map
│   └── Cargo.toml
│
├── tests/                    # Rust integration tests
│   └── fixtures/             # node/, python/, rust/, flutter/, docker/,
│                             # git-dirty/, git-clean/, broken-git/, mixed/
│
├── docs/                     # PLAN.md, ARCHITECTURE.md, SECURITY.md, DETECTORS.md
├── .github/
│   ├── workflows/ci.yml
│   ├── ISSUE_TEMPLATE/
│   └── PULL_REQUEST_TEMPLATE.md
├── README.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── LICENSE                    # MIT
└── package.json / vite.config.ts / tsconfig.json
```

---

## 5. Data model (SQLite)

Migrations applied on startup (a tiny home-grown migrator — no ORM).

```sql
-- scan_locations: user-added root dirs
CREATE TABLE scan_locations (
  id INTEGER PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  name TEXT,
  added_at INTEGER NOT NULL
);

-- projects: every discovered project root
CREATE TABLE projects (
  id INTEGER PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  relative_path TEXT,
  scan_location_id INTEGER REFERENCES scan_locations(id) ON DELETE CASCADE,
  size_bytes INTEGER,
  last_modified INTEGER,          -- mtime of most relevant files
  first_seen INTEGER,
  last_scanned INTEGER,
  is_favorite INTEGER DEFAULT 0,
  UNIQUE(scan_location_id, path)
);

-- project_techs: many-to-many technology/language tags
CREATE TABLE project_techs (
  project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
  tech       TEXT NOT NULL,
  PRIMARY KEY (project_id, tech)
);

-- git_metadata: cached read-only git info
CREATE TABLE git_metadata (
  project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  is_git INTEGER, branch TEXT, remote_url TEXT, repo_name TEXT,
  staged_count INTEGER, modified_count INTEGER, untracked_count INTEGER,
  last_commit_message TEXT, last_commit_date INTEGER,
  latest_short_hash TEXT, refreshed_at INTEGER
);

-- project_tags (future, schema reserved v0.4)
CREATE TABLE project_tags (
  project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  PRIMARY KEY (project_id, tag)
);

-- recent_projects: local "opened" history
CREATE TABLE recent_projects (
  project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
  opened_at INTEGER NOT NULL,
  PRIMARY KEY (project_id)
);

-- settings: key/value
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
```

Rule: **only paths + metadata**. Never store source code, env values, or API keys.

---

## 6. Rust architecture

- `AppState`: holds a `tokio::sync::Mutex<Connection>` to SQLite, settings,
  a cancelled-flag/cancel token for scans, and the active watcher handle.
- **Scanner** (`discovery/scanner.rs`): walks each scan location **asynchronously**,
  bounded depth (default e.g. 4), skips hidden dirs/`.git`/`node_modules`/`target`/build,
  guards against symlink loops, emits progress events, cancellable.
- **Detector registry** (`discovery/detector.rs`): a `ProjectDetector` trait;

  ```rust
  #[async_trait]
  pub trait ProjectDetector: Send + Sync {
      fn id(&self) -> &'static str;
      fn priority(&self) -> u8;
      /// Returns metadata iff this dir is detected as a project of this kind.
      fn detect(&self, dir: &Path, ctx: &DetectContext) -> Option<DetectedTech>;
  }
  ```

  Each detector is independent + unit-testable. A directory is a *project* if any
  detector matches; techs are accumulated across all matching detectors.
- **Git** (`git/mod.rs`): a `Git` struct wrapping `git -C <dir> …` invocations
  (`rev-parse`, `status --porcelain -b`, `log -1`). Returns typed `GitStatus`.
  Never runs write-commands in the MVP (read-only).
- **Processes/ports** (`processes/`): a `PlatformProcScanner` trait with
  `list_listening_ports() -> Vec<PortInfo>` and `ps() -> Vec<ProcInfo>`.
  - macOS/Linux: `lsof -iTCP -sTCP:LISTEN -P -n` + `ps -eo pid,comm`.
  - Windows: `netstat -ano` + `tasklist` (behind the same trait).
- **Docker** (`docker/mod.rs`): shells out to `docker ps -a --format …`.
  Optional — returns "unavailable" when the binary is missing.
- **Watch** (`watch/mod.rs`): `notify` watcher per scanned project root; on relevant
  file events (package.json, Cargo.toml, git HEAD, etc.) it emits a targeted refresh
  of that one project. No full rescans.
- **Health** (`health/`): pure deterministic functions:
  `is_git() -> has pkg manager -> runtime present(which) -> required scripts -> dirty`.
  No AI, structured result object.
- **Errors** (`error.rs`): single `Error` enum via `thiserror`; every variant maps to a
  `{ code, message, hint? }` serde struct. UI renders `message` + `hint`. One bad
  project → its own error, never a crash.
- **Command execution** (`security/`): all user-invoked commands run via
  `Command::new(bin).args(…).current_dir(…)` (no shell, no string interpolation),
  `.kill_on_drop`, with a strict allow-list of which bins may be launched and explicit
  UI confirmation before anything destructive (none in MVP).

---

## 7. Frontend architecture

- **Stores (Zustand):**
  - `projectsStore`: list, filters, sort, favorites toggle, selected project, detail.
  - `scanStore`: progress (scanned/total, current path), running/cancelled state.
  - `settingsStore`: scan locations, editor, terminal, scan frequency.
  - `systemStore`: git/docker/node presence flags, running services/ports.
- **Services (`src/services/`)**: thin wrapped calls to `invoke<T>("cmd", args)`.
  All business logic lives in Rust; services are just typed RPC clients.
- **Pages:**
  - `Onboarding` → pick dirs, run first scan, show "magic moment" summary.
  - `Dashboard` → search bar, filters, sort, project rows/cards, running-services strip.
  - `ProjectDetail` → tabs: Overview / Git / Environment / Services / Scripts / Files.
  - `Settings` → dirs, editor/terminal, scan frequency, privacy (what we read).
  - `CommandPalette` → global `Cmd+K` overlay (routes to open/search/scan/refresh).
- **Accessibility/UX:** keyboard-first, `Cmd+K` palette, `Cmd+1..n` nav, ESC closes
  overlays; dense rows with status dots; minimal motion.
- **Loading/empty/error states** for every async surface. Never a white screen.

---

## 8. Project detection architecture

Trait + registry (extensible, plugin-ready in spirit, no plugin marketplace yet):

```text
ProjectDetector
 ├── GitDetector        (.git)           — also marks it a project
 ├── NodeDetector       (package.json)   — scripts, package manager, deps present
 ├── PythonDetector     (pyproject.toml, requirements.txt, Pipfile, setup.py)
 ├── RustDetector       (Cargo.toml)
 ├── GoDetector         (go.mod)
 ├── FlutterDetector    (pubspec.yaml)  — dart/flutter
 ├── DotNetDetector     (*.csproj, *.sln)
 ├── DockerDetector     (Dockerfile, docker-compose.yml, compose.yml)
 ├── UnityDetector      (Project Settings/ProjectVersion.txt style markers)
 └── JavaDetector       (pom.xml, build.gradle(.kts))
```

A `DetectorRegistry::all()` supplies default order; each returns tech metadata only
(no aggressive file reads — checks for the marker files, reads package.json *scripts*
and dependency keys, nothing deeper). Adding a detector = implement trait + register.

---

## 9. Git integration architecture

- Read-only in MVP. `Git` wrapper runs `git -C <path> <read-only cmd>`.
- `status --porcelain -b` → branch + counts (staged/modified/untracked).
- `log -1 --format=…` → message, date, short hash.
- `remote get-url origin` → remote URL + derived repo name.
- Cached in `git_metadata`; refreshed on demand or on watcher-triggered change.
- If `git` is missing or repo is broken → `is_git=false`/error surfaced on that card,
  rest of the app keeps working.
- Future (not MVP): pull/push/commit/branch-switch — designed as new commands on the
  same `Git` struct with explicit confirmation, but **not built now**.

---

## 10. Process/port detection architecture

- `PlatformProcScanner` trait, per-OS impl (lsof/ps on macOS+Linux, netstat/tasklist
  on Windows). Cross-compiles; no unsafe.
- Returns `Vec<Listener>` (port, proto, pid, process name) and `Vec<ProcInfo>`.
- UI maps known ports → labels (3000 Vite/Node, 5173 Vite, 8000 Python, 5432 PG, …);
  clicking a port opens `http://localhost:<port>` in the browser (system opener).
- Detection runs on a schedule + on dashboard focus; not continuous.
- **No auto-kill.** Stopping always requires an explicit user confirm dialog.

---

## 11. Docker integration architecture

- `docker` CLI wrapper; `docker ps -a --format json` → running/stopped containers,
  names, ports, compose project. Optional: if binary missing → feature hidden.
- Start/Stop/Restart only with explicit user action.
- Never a hard dependency; absence is graceful.

---

## 12. Security model

- **No arbitrary auto-execution.** Every command is user-initiated, shown before run.
- All spawned processes use `Command` with explicit argv (no shell, no interpolation)
  → no shell-injection surface.
- **Path validation:** scan roots and project paths are canonicalised, confirmed under
  the chosen scan roots, symlink loops bounded, hidden/build dirs skipped.
- **No elevated privileges.** Normal operation never requests admin/root.
- **Privacy:** reads only metadata + marker files needed for detection; never source
  code, never env values, never keys. No telemetry by default. Settings shows exactly
  what DevDock reads.
- **Logging:** structured, secrets redacted by policy; env values/keys never logged.

---

## 13. Cross-platform strategy

- All platform-specific work sits behind traits (proc/ports). UI is pure web tech.
- Path handling via `std::path` + canonicalisation; no hardcoded `/` separators.
- Editor/terminal detection uses a per-OS candidate list (VS Code/Cursor/Zed/… and
  `open`/`xdg-open`/`start` for the system opener).
- CI builds macOS + Windows + Linux via GitHub Actions; release artifacts attached to
  GitHub Releases.

---

## 14. MVP scope (Phase 2, this build)

1. Tauri app shell + onboarding (add dirs → scan).
2. Async project scanner + detector registry (git, node, python, rust, go, flutter,
   dotnet, docker, unity, java).
3. SQLite storage (projects, techs, git_metadata, scan_locations, settings).
4. Dashboard (dense rows, tech badges, git status, favorite star, sort, search).
5. Git metadata (read-only) + graceful missing-git handling.
6. Project detail (Overview tab).
7. Open project (system file manager), open terminal, open editor (detected).

No AI, no process/port/docker/health UI yet (Phase 4), no watcher yet (Phase 3).

---

## 15. Development phases

- **Phase 1 (here):** architecture, decisions, structure, data model, this plan. ✅
- **Phase 2 (MVP):** shell, onboarding, scanner, detectors, DB, dashboard, git, detail,
  open/terminal/editor. — *next*
- **Phase 3:** search × filters polish, favorites, recents, command palette, fs watcher.
- **Phase 4:** process/port detection, docker, project health, running-services strip.
- **Phase 5+:** advanced integrations, optional AI (local/OpenAI/Anthropic/Google,
  never on by default), tags/notes, `.devdock` config, optional plugins.

---

## 16. Testing strategy

- **Rust unit tests:** every detector (fixtures), git porcelain parsing, path
  canonicalisation, health checks, command detection, error mapping.
- **Rust integration tests:** scan a fixture tree → expected projects discovered;
  git-clean vs git-dirty fixtures; broken-git fixture verified isolated.
- **Frontend (Vitest + Testing Library):** dashboard renders projects, search/filter
  behaviour, onboarding flow, command palette, empty states.
- `tests/fixtures/` holds tiny ecosystem projects (no real code, just markers + a
  script + minimal package.json) so scans are fast and deterministic.

---

## 17. CI/CD strategy (GitHub Actions)

`.github/workflows/ci.yml`:
- frontend: `npm ci`, `npm run lint`, `npm run test`, `npx tsc --noEmit`.
- rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`,
  `cargo build` (per-OS jobs: ubuntu, macos, windows).
- release workflow: tag → build binaries → attach to GitHub Releases
  (macOS `dmg`, Windows `msi/nsis`, Linux `AppImage/deb`).

Local parity rule: CI must match local toolchain versions (see FalMug lesson — align
CI to working local versions, don't downgrade deps to fit stale images).

---

## 18. GitHub repository structure

Mirrors section 4. Top-level: README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, LICENSE
(MIT), docs/, src/, src-tauri/, tests/, .github/{workflows, ISSUE_TEMPLATE,
PULL_REQUEST_TEMPLATE.md}. Owner: ArdaTkn, repo: `DevDock` (public).

---

## 19. Potential technical risks & mitigations

| Risk | Mitigation |
|------|-----------|
| Scanning perf on thousands of files | Async bounded walker, skip build/hidden dirs, depth limit, cache, cancel |
| Git edge cases / corrupt repos | CLI wrapper, per-project error isolation, cached metadata |
| Platform differences (win path/processes) | Trait abstraction from day one; CI on all 3 OSes |
| Tauri 2 API churn | Pin versions, verify against docs; CI build is the gate |
| Huge UI when 1000+ projects | Dense rows + virtualization (react-window) if needed; lazy detail |
| Feature creep into PM-tool territory | Scope guardrails in section 20 of product spec; this plan's MVP list |
| Command-exec accidents | No auto-exec, explicit argv, confirmation, allow-listed bins |

---

## 20. Future extensibility

- Detector trait → future plugin detectors.
- `.devdock` YAML (name/commands/services/ports) — reads as another metadata source.
- Optional AI layer behind a `ProjectAdvisor` trait (local/OAI/Anthropic/Google),
  opt-in only, source never uploaded by default.
- Read-write git commands behind confirmations.
- tags, notes, richer env detection (Phase 4+).

---

## Smallest useful first implementation (Phase 2 kickoff)

Materialise the repo skeleton + **spine**, proven end-to-end on a few of the user's
real directories, before adding every detector:

1. `cargo init` a Tauri 2 app (needs the Rust toolchain — currently missing on this machine).
2. Storage layer + `scan_locations`/`projects`/`project_techs`/`git_metadata` schema.
3. Scanner + **Node + Git** detectors (the user's dominant stack) over `~/Projects`
   and a couple real dirs, writing to SQLite.
4. Minimal React dashboard rendering discovered projects from SQLite, with:
   open project / open terminal / open editor.
5. Tests + CI on that spine; then iterate detectors (flutter, python, rust, docker…)
   one at a time with fixtures.

**Gate:** the Rust toolchain + Tauri CLI are not installed on this machine, so Phase 2
requires installing rustup (≈1 GB toolchain + Xcode CLT if absent). This is the one
environmental prerequisite before we can compile anything real.

---

### Next step

Approve installing the Rust toolchain (rustup) and bootstrap the Tauri 2 skeleton, then
build the section-20 spine and verify it against your real projects.
