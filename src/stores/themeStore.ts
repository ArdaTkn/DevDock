import { create } from "zustand";

export type ThemeKey = "emerald" | "cyberpunk" | "nordic" | "monokai" | "dracula";

export interface ThemeOption {
  key: ThemeKey;
  name: string;
  accent: string;
}

export const THEME_OPTIONS: ThemeOption[] = [
  { key: "emerald", name: "Emerald Night", accent: "#10b981" },
  { key: "cyberpunk", name: "Cyberpunk Neon", accent: "#a855f7" },
  { key: "nordic", name: "Nordic Frost", accent: "#3b82f6" },
  { key: "monokai", name: "Monokai Gold", accent: "#eab308" },
  { key: "dracula", name: "Dracula Pink", accent: "#ec4899" },
];

interface ThemeState {
  theme: ThemeKey;
  setTheme: (t: ThemeKey) => void;
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: (localStorage.getItem("devdock_theme") as ThemeKey) || "emerald",
  setTheme: (t) => {
    localStorage.setItem("devdock_theme", t);
    document.documentElement.setAttribute("data-theme", t);
    set({ theme: t });
  },
}));
