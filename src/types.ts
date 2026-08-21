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

export interface PortInfo {
  port: number;
  pid: number;
  process_name: string;
  label: String;
}

export interface DockerContainerInfo {
  id: string;
  name: string;
  image: string;
  status: string;
  ports: string;
  is_running: boolean;
}

export interface ProjectHealth {
  score: number;
  status: string;
  deps_installed: boolean;
  has_readme: boolean;
  is_git_clean: boolean;
  issues: string[];
}

export interface ProjectScript {
  name: string;
  command: string;
  source: string;
}

export interface GitHubRepoInfo {
  owner: string;
  repo: string;
  repo_url: string;
  issues_url: string;
  pulls_url: string;
}

export interface CustomCommandDto {
  id: number;
  name: string;
  command: string;
}

export interface DependencyInfo {
  name: string;
  version: string;
  is_dev: boolean;
}

export interface WorkspaceDto {
  id: number;
  name: string;
  color: string;
}

export interface BulkGitResult {
  path: string;
  success: boolean;
  message: string;
}

export interface BulkGitStatusResult {
  path: string;
  is_dirty: boolean;
  branch: string;
  uncommitted_count: number;
}

export interface EnvDiffReport {
  has_template: boolean;
  template_file: string | null;
  has_local_env: boolean;
  local_env_file: string | null;
  template_keys: string[];
  local_keys: string[];
  missing_keys: string[];
  extra_keys: string[];
}

export interface GitIgnoreAuditReport {
  has_gitignore: boolean;
  sensitive_files_found: string[];
  unignored_sensitive_files: string[];
}

export interface RuntimeVersionInfo {
  toolchain: string;
  required_version: string;
  detected_version: string | null;
  source_file: string;
  is_matched: boolean;
}

export interface CacheFolderInfo {
  name: string;
  path: string;
  size_bytes: number;
  human_size: string;
  category: string;
  is_safe: boolean;
}

export interface ProjectCacheReport {
  total_size_bytes: number;
  total_human_size: string;
  reclaimable_bytes: number;
  reclaimable_human_size: string;
  cache_folders: CacheFolderInfo[];
}

export interface DiskHogItem {
  project_path: string;
  project_name: string;
  total_size_bytes: number;
  reclaimable_bytes: number;
  reclaimable_human_size: string;
  last_modified: number;
  is_stale: boolean;
  cache_folders: CacheFolderInfo[];
}

export interface DiskHogReport {
  total_reclaimable_bytes: number;
  total_reclaimable_human_size: string;
  stale_projects_count: number;
  items: DiskHogItem[];
}