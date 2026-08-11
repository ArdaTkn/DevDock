import type { GitInfo, Project } from "../types";

/** Compact human time: "2h ago", "3d ago", "just now". */
export function timeAgo(unix: number): string {
  if (!unix) return "never";
  const secs = Math.max(0, Math.floor(Date.now() / 1000) - unix);
  if (secs < 60) return "just now";
  const m = Math.floor(secs / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.floor(h / 24);
  if (d < 30) return `${d}d ago`;
  const mo = Math.floor(d / 30);
  return `${mo}mo ago`;
}

/** Human-readable byte size. */
export function humanSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = bytes;
  let i = -1;
  do {
    v /= 1024;
    i++;
  } while (v >= 1024 && i < units.length - 1);
  return `${v >= 100 ? Math.round(v) : v.toFixed(1)} ${units[i]}`;
}

/** Git dirty state: "clean" | "dirty" | "none". */
export type GitState = "clean" | "dirty" | "none";

export function gitState(git: GitInfo | null | undefined): GitState {
  if (!git || !git.is_git) return "none";
  return git.staged_count === 0 &&
    git.modified_count === 0 &&
    git.untracked_count === 0
    ? "clean"
    : "dirty";
}

/** Short path with ~ for home dir. */
export function displayPath(path: string): string {
  const home = "/Users/ardatekin"; // replaced at scan time conventionally
  if (path.startsWith(home)) return "~" + path.slice(home.length);
  return path;
}

/** All distinct tech names across projects, for filter chips. */
export function allTechs(projects: Project[]): string[] {
  const set = new Set<string>();
  for (const p of projects) {
    for (const t of p.techs) set.add(t.name);
  }
  return Array.from(set).sort();
}