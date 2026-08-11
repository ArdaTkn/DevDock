import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import type { ScanProgress, ScanSummary } from "../types";
import { api } from "../services/api";
import { useProjectsStore } from "./projectsStore";

// Fires the automatic home scan only once per app session, even across remounts.
let hasBootstrapped = false;

interface ScanState {
  running: boolean;
  progress: ScanProgress | null;
  summary: ScanSummary | null;
  error: string | null;
  start: () => Promise<ScanSummary>;
  cancel: () => Promise<void>;
  clear: () => void;
  /** Subscribe once to Rust `scan-progress` events. */
  listen: () => void;
  /** Kick a scan if the project list is empty and we haven't scanned yet. */
  ensureScan: () => Promise<void>;
}

export const useScanStore = create<ScanState>((set, get) => ({
  running: false,
  progress: null,
  summary: null,
  error: null,

  start: async () => {
    set({ running: true, error: null, summary: null, progress: null });
    try {
      const summary = await api.scanProjects();
      set({ running: false, summary, progress: null });
      return summary;
    } catch (e) {
      set({ running: false, error: String(e), progress: null });
      throw e;
    }
  },

  cancel: async () => {
    try {
      await api.cancelScan();
      set({ running: false, progress: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clear: () => set({ summary: null, error: null, progress: null }),

  listen: () => {
    (window as unknown as Record<string, boolean>).__devdock_scan_listener__ ??= true;
    void listen<ScanProgress>("scan-progress", (e) => {
      const p = e.payload;
      set({
        progress: p,
        running: !p.done && !p.cancelled,
      });
    });
  },

  ensureScan: async () => {
    if (hasBootstrapped) return;
    hasBootstrapped = true;
    const projects = useProjectsStore.getState().projects;
    if (projects.length > 0 || get().running) return;
    try {
      const summary = await get().start();
      if (summary) await useProjectsStore.getState().refresh();
    } catch {
      // error is stored in state; Dashboard renders it
    }
  },
}));