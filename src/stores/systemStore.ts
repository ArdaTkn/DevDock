import { create } from "zustand";
import { api } from "../services/api";

interface SystemState {
  editor: string | null;
  /** Installed terminals detected on the system (empty = not detected). */
  terminals: string[];
  /** User-chosen terminal; "" = "System default (auto)". */
  terminalPref: string;
  loading: boolean;
  detectEditor: () => Promise<void>;
  loadTerminals: () => Promise<void>;
  loadTerminalPref: () => Promise<void>;
  setTerminal: (t: string) => Promise<void>;
}

export const useSystemStore = create<SystemState>((set) => ({
  editor: null,
  terminals: [],
  terminalPref: "",
  loading: false,

  detectEditor: async () => {
    set({ loading: true });
    const editor = await api.detectEditor();
    set({ editor, loading: false });
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
}));