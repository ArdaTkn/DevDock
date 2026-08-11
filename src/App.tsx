import { BrowserRouter, NavLink, Route, Routes } from "react-router-dom";
import { Dashboard } from "./pages/Dashboard";
import { ProjectDetail } from "./pages/ProjectDetail";
import { Settings } from "./pages/Settings";

function Layout() {
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
        <div className="sidebar-foot muted">local · private · no cloud</div>
      </nav>
      <main className="main">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/project/:id" element={<ProjectDetail />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
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