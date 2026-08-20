import { useEffect, useState } from "react";
import { useProjectsStore } from "../stores/projectsStore";
import { useSystemStore } from "../stores/systemStore";
import { useThemeStore, THEME_OPTIONS } from "../stores/themeStore";

export function Settings() {
  const { locations, error, load, addLocation, removeLocation } = useProjectsStore();
  const {
    terminals,
    terminalPref,
    editors,
    editorPref,
    loadTerminals,
    loadTerminalPref,
    setTerminal,
    loadEditors,
    loadEditorPref,
    setEditor,
  } = useSystemStore();
  const { theme, setTheme } = useThemeStore();
  const [dir, setDir] = useState("");
  const [adding, setAdding] = useState(false);

  useEffect(() => {
    void load();
    void loadEditors();
    void loadEditorPref();
    void loadTerminals();
    void loadTerminalPref();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const add = async () => {
    if (!dir.trim()) return;
    setAdding(true);
    await addLocation(dir.trim());
    setDir("");
    setAdding(false);
  };

  return (
    <div className="page">
      <h1>Settings</h1>

      {error && <div className="banner error">{error}</div>}

      <section className="panel">
        <h2>Scan directories</h2>
        <p className="muted">
          DevDock scans these folders for development projects. It only reads project
          metadata and marker files — never your source code.
        </p>
        <ul className="loc-list">
          {locations.map((l) => (
            <li key={l.id} className="loc-row">
              <code>{l.path}</code>
              <button
                className="btn danger"
                onClick={() => void removeLocation(l.id)}
              >
                Remove
              </button>
            </li>
          ))}
          {locations.length === 0 && <li className="muted">No directories added yet.</li>}
        </ul>
        <div className="add-row">
          <input
            className="search grow"
            placeholder="Absolute path, e.g. /Users/you/Projects or ~/Code"
            value={dir}
            onChange={(e) => setDir(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && void add()}
          />
          <button className="btn primary" disabled={adding} onClick={() => void add()}>
            Add Folder
          </button>
        </div>
      </section>

      <section className="panel">
        <h2>Code Editor</h2>
        <p className="muted">
          Choose which editor DevDock launches when you click Open Editor. Only
          editors detected on this system are listed.
        </p>
        <select
          className="select"
          value={editorPref}
          onChange={(e) => void setEditor(e.target.value)}
          aria-label="editor"
        >
          <option value="">System default (auto-detect)</option>
          {editors.map((e) => (
            <option key={e} value={e}>
              {e}
            </option>
          ))}
        </select>
        {editors.length === 0 && (
          <p className="muted small">
            No supported editors detected — DevDock uses the system default opener.
          </p>
        )}
      </section>

      <section className="panel">
        <h2>Terminal</h2>
        <p className="muted">
          Choose which terminal DevDock opens when you press Terminal. Only
          terminals detected on this system are listed.
        </p>
        <select
          className="select"
          value={terminalPref}
          onChange={(e) => void setTerminal(e.target.value)}
          aria-label="terminal"
        >
          <option value="">System default (auto)</option>
          {terminals.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>
        {terminals.length === 0 && (
          <p className="muted small">
            No third-party terminals detected — DevDock uses the system default.
          </p>
        )}
      </section>

      <section className="panel">
        <h2>Open actions</h2>
        <p className="muted">
          <b>Open Editor</b> opens the first detected editor (VS Code, Cursor, Zed).
          <br />
          <b>Open Terminal</b> opens Terminal.app on macOS.
          <br />
          <b>Open</b> reveals the folder in your file manager.
        </p>
      </section>

      <section className="panel">
        <h2>Appearance & Theme</h2>
        <p className="muted">
          Customize DevDock's visual theme and color palette.
        </p>
        <div className="theme-grid">
          {THEME_OPTIONS.map((t) => (
            <button
              key={t.key}
              className={`theme-card ${theme === t.key ? "active" : ""}`}
              onClick={() => setTheme(t.key)}
            >
              <span className="theme-swatch" style={{ background: t.accent }} />
              <span className="theme-name">{t.name}</span>
            </button>
          ))}
        </div>
      </section>

      <section className="panel">
        <h2>Terminal</h2>
        <p className="muted">
          DevDock reads only: directory listings, marker files (package.json scripts,
          Cargo.toml, pubspec.yaml…), Git status (read-only), and file sizes/mtimes.
          <br />
          No source code, no environment variables, no API keys are ever read or
          transmitted. Everything stays on this machine. No telemetry.
        </p>
      </section>
    </div>
  );
}