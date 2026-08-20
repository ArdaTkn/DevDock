import { invoke } from "@tauri-apps/api/core";
import type {
  ScanLocation,
  ScanSummary,
  Project,
  ErrorDto,
} from "../types";

// Thin typed RPC wrappers over Tauri commands. No business logic here.
// All real work happens in the Rust core; failures come back as ErrorDto.

export const api = {
  listScanLocations: () =>
    invoke<ScanLocation[]>("list_scan_locations"),

  addScanLocation: (path: string) =>
    invoke<number>("add_scan_location", { path }),

  removeScanLocation: (id: number) =>
    invoke<void>("remove_scan_location", { id }),

  scanProjects: () =>
    invoke<ScanSummary>("scan_projects"),

  cancelScan: () =>
    invoke<void>("cancel_scan"),

  listProjects: (limit?: number) =>
    invoke<Project[]>("list_projects", { limit }),

  getProject: (id: number) =>
    invoke<Project | null>("get_project", { id }),

  setFavorite: (id: number, favorite: boolean) =>
    invoke<void>("set_favorite", { id, favorite }),

  listRecent: (limit?: number) =>
    invoke<Project[]>("list_recent", { limit: limit ?? 10 }),

  openFolder: (path: string) =>
    invoke<void>("open_project_folder", { path }),

  openTerminal: (path: string) =>
    invoke<void>("open_project_terminal", { path }),

  openEditor: (path: string) =>
    invoke<void>("open_project_editor", { path }),

  detectEditor: () =>
    invoke<string | null>("detect_editor"),

  listEditors: () =>
    invoke<string[]>("list_editors"),

  getEditorPref: () =>
    invoke<string | null>("get_editor_pref"),

  setEditorPref: (pref: string) =>
    invoke<void>("set_editor_pref", { pref }),

  listTerminals: () =>
    invoke<string[]>("list_terminals"),

  getTerminalPref: () =>
    invoke<string | null>("get_terminal_pref"),

  setTerminalPref: (pref: string) =>
    invoke<void>("set_terminal_pref", { pref }),

  listListeningPorts: () =>
    invoke<import("../types").PortInfo[]>("list_listening_ports"),

  listDockerContainers: () =>
    invoke<import("../types").DockerContainerInfo[]>("list_docker_containers"),

  getProjectHealth: (path: string, isGitDirty: boolean) =>
    invoke<import("../types").ProjectHealth>("get_project_health", {
      path,
      isGitDirty,
    }),

  listProjectScripts: (path: string) =>
    invoke<import("../types").ProjectScript[]>("list_project_scripts", {
      path,
    }),

  runProjectScript: (path: string, scriptCommand: string) =>
    invoke<void>("run_project_script", { path, scriptCommand }),

  getGithubInfo: (remoteUrl: string) =>
    invoke<import("../types").GitHubRepoInfo | null>("get_github_info", {
      remoteUrl,
    }),

  getProjectTags: (projectId: number) =>
    invoke<string[]>("get_project_tags", { projectId }),

  addProjectTag: (projectId: number, tag: string) =>
    invoke<void>("add_project_tag", { projectId, tag }),

  removeProjectTag: (projectId: number, tag: string) =>
    invoke<void>("remove_project_tag", { projectId, tag }),

  getProjectNotes: (projectId: number) =>
    invoke<string | null>("get_project_notes", { projectId }),

  setProjectNotes: (projectId: number, content: string) =>
    invoke<void>("set_project_notes", { projectId, content }),

  listCustomCommands: (projectId: number) =>
    invoke<import("../types").CustomCommandDto[]>("list_custom_commands", {
      projectId,
    }),

  addCustomCommand: (projectId: number, name: string, command: string) =>
    invoke<void>("add_custom_command", { projectId, name, command }),

  removeCustomCommand: (id: number) =>
    invoke<void>("remove_custom_command", { id }),

  getProjectDependencies: (path: string) =>
    invoke<import("../types").DependencyInfo[]>("get_project_dependencies", {
      path,
    }),
};

/** Turns a Rust ErrorDto into a readable string for inline UI display. */
export function errorText(e: unknown): string {
  const dto = e as ErrorDto;
  if (dto?.message) return dto.message;
  if (!dto?.hint) return String(e ?? "Unknown error");
  return dto.message + (dto.hint ? " " + dto.hint : "");
}