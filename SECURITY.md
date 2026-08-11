# Security Policy

## Reporting a vulnerability

DevDock is a security-sensitive application: it reads local filesystem metadata
and launches local commands. If you find a security issue, **do not open a
public issue**. Instead, report it privately by emailing the maintainer at
ardatekin+security@users.noreply.github.com.

Please include:

- A description of the vulnerability and its impact.
- Steps to reproduce.
- Any relevant logs (with secrets removed).

We aim to respond within 5 business days.

## Security model

- **Local-first.** No cloud, no backend, no account. Core functionality works
  fully offline.
- **No source upload.** DevDock reads only directory listings, small marker
  files (e.g. `package.json` scripts), and read-only Git status. It never reads
  or transmits source code, environment variables, or secrets.
- **No telemetry by default.** No analytics are collected or sent.
- **No auto-execution.** DevDock never runs project commands on its own. User
  actions launch binaries via explicit argv (`Command::new(bin).args(…)`,
  no shell), which eliminates shell-injection.
- **Path validation.** Scan paths are canonicalised; symlinked directories that
  escape the scan root are skipped; hidden/ignored dirs (`node_modules`,
  `target`, `.git`, build dirs) are never descended into.
- **No elevated privileges.** Normal operation never requests admin/root.
- **Logging.** Structured logs never contain env values, keys, or passwords.

## Supported versions

Only the current development version receives security fixes. There are no
published releases yet.

## Scope

In scope: the `src-tauri` Rust core, the Vite/React frontend, and the build/CI
configuration. Anything that could cause arbitrary code execution or a privacy
leak from a maliciously crafted project tree is our top concern.
