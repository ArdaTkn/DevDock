import { useState, useEffect } from "react";
import { BrowserRouter, NavLink, Route, Routes } from "react-router-dom";
import { Dashboard } from "./pages/Dashboard";
import { ProjectDetail } from "./pages/ProjectDetail";
import { Settings } from "./pages/Settings";
import { CommandPalette } from "./components/CommandPalette";
import { useThemeStore } from "./stores/themeStore";

function Layout() {
  const [isPaletteOpen, setIsPaletteOpen] = useState(false);
  const theme = useThemeStore((s) => s.theme);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setIsPaletteOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <div className="app">
      <nav className="sidebar">
        <div className="brand">
          <span className="brand-mark">D</span>
          <span className="brand-name">DevDock</span>
        </div>
        <div className="nav-links">
          <NavLink to="/" end>
            Projects
          </NavLink>
          <NavLink to="/settings">Settings</NavLink>
        </div>
        <button
          className="cmd-trigger-btn"
          onClick={() => setIsPaletteOpen(true)}
          title="Open Command Palette (⌘K)"
        >
          <span className="cmd-trigger-icon">🔍</span>
          <span>Command Palette</span>
          <kbd className="cmd-trigger-kbd">⌘K</kbd>
        </button>
        <div className="sidebar-foot muted">local · private · no cloud</div>
      </nav>
      <main className="main">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/project/:id" element={<ProjectDetail />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>

      <CommandPalette
        isOpen={isPaletteOpen}
        onClose={() => setIsPaletteOpen(false)}
      />
    </div>
  );
}

export function App() {
  // The shell always renders. Loading/empty/scanning states live inside the
  // page components with their own spinners — the app can NEVER hang on a
  // blank/boot screen if a command is slow.
  return (
    <BrowserRouter>
      <Layout />
    </BrowserRouter>
  );
}