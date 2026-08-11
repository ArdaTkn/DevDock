// Shared types mirroring the Rust DTOs (src-tauri/src/models.rs).

export type TechKind = "language" | "framework" | "tool" | "runtime";

export interface Tech {
  name: string;
  kind: TechKind;
}

export interface GitInfo {
  is_git: boolean;
  branch: string | null;
  remote_url: string | null;
  repo_name: string | null;
  staged_count: number;
  modified_count: number;
  untracked_count: number;
  last_commit_message: string | null;
  last_commit_date: number | null;
  latest_short_hash: string | null;
}

export interface Project {
  id: number;
  path: string;
  name: string;
  relative_path: string | null;
  size_bytes: number;
  last_modified: number;
  is_favorite: boolean;
  techs: Tech[];
  git: GitInfo | null;
}

export interface ScanLocation {
  id: number;
  path: string;
  name: string;
}

export interface ScanSummary {
  total: number;
  tech_breakdown: [string, number][];
  dirty_count: number;
  clean_count: number;
}

export interface ScanProgress {
  scanned: number;
  total: number;
  current_path: string;
  done: boolean;
  found: number;
  cancelled: boolean;
}

export interface ErrorDto {
  message: string;
  hint: string | null;
}