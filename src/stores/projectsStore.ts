import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import type { Project, ScanLocation, WorkspaceDto } from "../types";
import { api } from "../services/api";

export type SortKey =
  | "favorites"
  | "recent"
  | "name"
  | "dirty"
  | "path";

interface ProjectsState {
  projects: Project[];
  recentProjects: Project[];
  locations: ScanLocation[];
  workspaces: WorkspaceDto[];
  activeWorkspaceId: number | null;
  workspaceProjectIds: number[];
  loading: boolean;
  search: string;
  techFilter: string | null;
  sort: SortKey;
  error: string | null;
  load: () => Promise<void>;
  refresh: () => Promise<void>;
  loadRecent: () => Promise<void>;
  loadWorkspaces: () => Promise<void>;
  createWorkspace: (name: string, color: string) => Promise<void>;
  deleteWorkspace: (id: number) => Promise<void>;
  setActiveWorkspaceId: (id: number | null) => Promise<void>;
  listenWatcher: () => Promise<void>;
  setSearch: (s: string) => void;
  setTechFilter: (t: string | null) => void;
  setSort: (s: SortKey) => void;
  toggleFavorite: (id: number, fav: boolean) => Promise<void>;
  addLocation: (path: string) => Promise<void>;
  removeLocation: (id: number) => Promise<void>;
}

let watcherUnsub: (() => void) | null = null;

export type ProjectWithKey = Project; // placeholder for future memoisation

async function loadProjects(): Promise<Project[]> {
  const all = await api.listProjects(2000);
  return all;
}

export const useProjectsStore = create<ProjectsState>((set, get) => ({
  projects: [],
  recentProjects: [],
  locations: [],
  workspaces: [],
  activeWorkspaceId: null,
  workspaceProjectIds: [],
  loading: false,
  search: "",
  techFilter: null,
  sort: "recent",
  error: null,

  loadWorkspaces: async () => {
    try {
      const ws = await api.listWorkspaces();
      set({ workspaces: ws });
    } catch {
      // ignore
    }
  },

  createWorkspace: async (name: string, color: string) => {
    try {
      const w = await api.createWorkspace(name, color);
      set({ workspaces: [...get().workspaces, w] });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteWorkspace: async (id: number) => {
    try {
      await api.deleteWorkspace(id);
      set({
        workspaces: get().workspaces.filter((w) => w.id !== id),
        activeWorkspaceId: get().activeWorkspaceId === id ? null : get().activeWorkspaceId,
        workspaceProjectIds: get().activeWorkspaceId === id ? [] : get().workspaceProjectIds,
      });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setActiveWorkspaceId: async (id: number | null) => {
    if (id === null) {
      set({ activeWorkspaceId: null, workspaceProjectIds: [] });
    } else {
      try {
        const pids = await api.listWorkspaceProjectIds(id);
        set({ activeWorkspaceId: id, workspaceProjectIds: pids });
      } catch {
        set({ activeWorkspaceId: id, workspaceProjectIds: [] });
      }
    }
  },

  listenWatcher: async () => {
    if (watcherUnsub) return;
    try {
      const unsub = await listen("fs-change", () => {
        void get().refresh();
      });
      watcherUnsub = unsub;
    } catch {
      // Ignored in non-Tauri browser dev mode
    }
  },

  load: async () => {
    set({ loading: true, error: null });
    try {
      const [projects, locations, recentProjects] = await Promise.all([
        loadProjects(),
        api.listScanLocations(),
        api.listRecent(5),
      ]);
      set({ projects, locations, recentProjects, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  refresh: async () => {
    try {
      const [projects, recentProjects] = await Promise.all([
        loadProjects(),
        api.listRecent(5),
      ]);
      set({ projects, recentProjects });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadRecent: async () => {
    try {
      const recentProjects = await api.listRecent(5);
      set({ recentProjects });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setSearch: (search) => set({ search }),
  setTechFilter: (techFilter) => set({ techFilter }),
  setSort: (sort) => set({ sort }),

  toggleFavorite: async (id, favorite) => {
    set({ error: null });
    try {
      await api.setFavorite(id, favorite);
      await get().refresh();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  addLocation: async (path) => {
    set({ error: null });
    try {
      await api.addScanLocation(path);
      await get().load();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  removeLocation: async (id) => {
    set({ error: null });
    try {
      await api.removeScanLocation(id);
      await get().load();
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));