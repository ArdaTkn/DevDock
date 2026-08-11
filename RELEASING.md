# Releasing DevDock

This explains how to cut a release so the CI builds installers for all three
platforms and attaches them to a GitHub Release.

## How releases are built

GitHub Actions (`.github/workflows/release.yml`) listens for `v*` tags. Pushing a
tag like `v0.1.0` triggers a build of:

- **macOS** → `DevDock_0.1.0_aarch64.dmg`
- **Windows** → `DevDock_0.1.0_x64.msi` (and `.exe` installer)
- **Linux** → `DevDock_0.1.0_amd64.AppImage` and `.deb`

The first release is created as a **draft** (`releaseDraft: true`). Publish it
from the GitHub Releases page once you've smoke-tested the artifacts.

> Binaries are not code-signed. Note the macOS right-click→Open / `xattr -dr
> com.apple.quarantine` instruction is included in the release body automatically.

## Cutting a release (1-2-3)

```bash
# 1. Bump the version
npm version patch        # or minor / major (bumps package.json)
# also bump src-tauri/Cargo.toml + src-tauri/tauri.conf.json manually to match

# 2. Commit and push
git add -A && git commit -m "chore: release vX.Y.Z"
git push

# 3. Tag and push the tag (this triggers CI)
git tag vX.Y.Z
git push origin vX.Y.Z
```

Go to https://github.com/ArdaTkn/DevDock/releases — wait for the workflow to
finish, verify the installer names look right, then click **Publish release**.

## Manual / local build (for your own machine)

```bash
npm run tauri build
# → src-tauri/target/release/bundle/macos/DevDock.app
cp -R src-tauri/target/release/bundle/macos/DevDock.app ~/Applications/
```

## Version alignment

Keep these in sync on a release: `package.json`, `src-tauri/Cargo.toml`
(`[package] version`), and `src-tauri/tauri.conf.json` (`version`). The CI
builds the version string from `tauri.conf.json`/`Cargo.toml`.

## Pre-release / RC

Push a tag like `v0.2.0-rc.1` and set `prerelease: true` in `release.yml`, or
simply publish as a prerelease from the Releases UI (the workflow keeps it a
draft by default; you can flip "prerelease" manually).