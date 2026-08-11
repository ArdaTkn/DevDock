import { create } from "zustand";
import { api } from "../services/api";

interface SystemState {
  editor: string | null;
  loading: boolean;
  detectEditor: () => Promise<void>;
}

export const useSystemStore = create<SystemState>((set) => ({
  editor: null,
  loading: false,

  detectEditor: async () => {
    set({ loading: true });
    const editor = await api.detectEditor();
    set({ editor, loading: false });
  },
}));