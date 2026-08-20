import { create } from "zustand";
import { api } from "../services/api";
import type { PortInfo } from "../types";

interface SystemState {
  editor: string | null;
  editors: string[];
  editorPref: string;
  /** Installed terminals detected on the system (empty = not detected). */
  terminals: string[];
  /** User-chosen terminal; "" = "System default (auto)". */
  terminalPref: string;
  /** Active listening TCP ports on the machine */
  ports: PortInfo[];
  loading: boolean;
  detectEditor: () => Promise<void>;
  loadEditors: () => Promise<void>;
  loadEditorPref: () => Promise<void>;
  setEditor: (e: string) => Promise<void>;
  loadTerminals: () => Promise<void>;
  loadTerminalPref: () => Promise<void>;
  setTerminal: (t: string) => Promise<void>;
  loadPorts: () => Promise<void>;
}

export const useSystemStore = create<SystemState>((set) => ({
  editor: null,
  editors: [],
  editorPref: "",
  terminals: [],
  terminalPref: "",
  ports: [],
  loading: false,

  detectEditor: async () => {
    set({ loading: true });
    const editor = await api.detectEditor();
    set({ editor, loading: false });
  },

  loadEditors: async () => {
    const editors = await api.listEditors();
    set({ editors });
  },

  loadEditorPref: async () => {
    const pref = await api.getEditorPref();
    set({ editorPref: pref ?? "" });
  },

  setEditor: async (e) => {
    await api.setEditorPref(e);
    set({ editorPref: e });
  },

  loadTerminals: async () => {
    const terminals = await api.listTerminals();
    set({ terminals });
  },

  loadTerminalPref: async () => {
    const pref = await api.getTerminalPref();
    set({ terminalPref: pref ?? "" });
  },

  setTerminal: async (t) => {
    await api.setTerminalPref(t);
    set({ terminalPref: t });
  },

  loadPorts: async () => {
    try {
      const ports = await api.listListeningPorts();
      set({ ports });
    } catch {
      set({ ports: [] });
    }
  },
}));