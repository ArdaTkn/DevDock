import { create } from "zustand";
import type { Project, ScanLocation } from "../types";
import { api } from "../services/api";

export type SortKey =
  | "recent"
  | "name"
  | "dirty"
  | "path";

interface ProjectsState {
  projects: Project[];
  locations: ScanLocation[];
  loading: boolean;
  search: string;
  techFilter: string | null;
  sort: SortKey;
  error: string | null;
  load: () => Promise<void>;
  refresh: () => Promise<void>;
  setSearch: (s: string) => void;
  setTechFilter: (t: string | null) => void;
  setSort: (s: SortKey) => void;
  toggleFavorite: (id: number, fav: boolean) => Promise<void>;
  addLocation: (path: string) => Promise<void>;
  removeLocation: (id: number) => Promise<void>;
}

export type ProjectWithKey = Project; // placeholder for future memoisation

async function loadProjects(): Promise<Project[]> {
  const all = await api.listProjects(2000);
  return all;
}

export const useProjectsStore = create<ProjectsState>((set, get) => ({
  projects: [],
  locations: [],
  loading: false,
  search: "",
  techFilter: null,
  sort: "recent",
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const [projects, locations] = await Promise.all([
        loadProjects(),
        api.listScanLocations(),
      ]);
      set({ projects, locations, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  refresh: async () => {
    try {
      const projects = await loadProjects();
      set({ projects });
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