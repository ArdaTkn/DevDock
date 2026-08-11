# Contributing to DevDock

Thanks for your interest! DevDock is a small, focused developer tool, and we
value contributions that keep it small, correct, and privacy-first.

## Ground rules

- **Local-first and privacy-first.** Never add a feature that uploads source
  code, requires an account, or phones home by default.
- **Small but genuinely useful.** Resist scope creep. If a PR adds "Jira but
  local", it won't be merged. DevDock integrates with your tools; it doesn't
  replace your IDE, terminal, or GitHub.
- **Keep it lean.** The app is meant to stay lightweight and fast with 500–1000
  projects. Expensive full scans and heavy UI dependencies are the enemy.

## Development setup

See the [README](README.md#installation). In short:

```bash
npm install
npm run tauri dev      # desktop app with hot reload
```

## Adding a new project detector

1. Create `src-tauri/src/discovery/detectors/<name>.rs` implementing the
   `ProjectDetector` trait (see `discovery/detector.rs`). Only inspect the
   marker files you actually need — never read source code.
2. Register it in `DetectorRegistry::default_registry()` and in
   `discovery/detectors/mod.rs`.
3. Add a fixture under `src-tauri/tests/fixtures/<name>-project/`.
4. Add a test in `src-tauri/tests/discovery_test.rs`.
5. Update `docs/DETECTORS.md`.

## Testing & linting

```bash
cd src-tauri
cargo test
cargo clippy -- -D warnings
cd ..
npm run typecheck
npm run build
```

CI runs all of the above on macOS, Windows, and Linux. Your PR must be green
on all three.

## Commit conventions

- Small, focused commits. One logical change per commit.
- Clear messages: `feat(scanner): skip symlinked dirs that escape the scan root`.
- Reference issues/PRs where relevant.

## Code of conduct

All participants must follow our [Code of Conduct](CODE_OF_CONDUCT.md).
Be kind and constructive — this is a small community project.

## Questions?

Open an issue. For detector/architecture questions, mention the relevant doc
(`docs/ARCHITECTURE.md`, `docs/DETECTORS.md`).
