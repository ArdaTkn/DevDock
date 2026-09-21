import { useEffect, useState } from "react";
import { api } from "../services/api";
import { useProjectsStore } from "../stores/projectsStore";
import { useSystemStore } from "../stores/systemStore";
import { useThemeStore, THEME_OPTIONS } from "../stores/themeStore";

export function Settings() {
  const {
    locations,
    workspaces,
    error,
    load,
    loadWorkspaces,
    deleteWorkspace,
    addLocation,
    removeLocation,
  } = useProjectsStore();
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
  const [deleteConfirmWs, setDeleteConfirmWs] = useState<import("../types").WorkspaceDto | null>(null);
  
  // Hardware Station State
  const [hardwarePorts, setHardwarePorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState<string>("");
  const [telemetry, setTelemetry] = useState<import("../types").HardwareTelemetryDto | null>(null);
  const [signalLog, setSignalLog] = useState<string | null>(null);
  const [sendingSignal, setSendingSignal] = useState(false);

  useEffect(() => {
    void load();
    void loadWorkspaces();
    void loadEditors();
    void loadEditorPref();
    void loadTerminals();
    void loadTerminalPref();
    void loadHardware();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const loadHardware = async () => {
    try {
      const ports = await api.listHardwarePorts();
      setHardwarePorts(ports);
      if (ports.length > 0 && !selectedPort) {
        setSelectedPort(ports[0]);
      }
      const tel = await api.getHardwareTelemetry();
      setTelemetry(tel);
    } catch (e) {
      console.error(e);
    }
  };

  const handleSendSignal = async (sig: string) => {
    setSendingSignal(true);
    try {
      const res = await api.sendHardwareSignal(selectedPort || null, sig);
      setSignalLog(res.message);
    } catch (e) {
      setSignalLog(`Failed to transmit signal: ${String(e)}`);
    } finally {
      setSendingSignal(false);
    }
  };

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
        <h2>📁 Workspace Management</h2>
        <p className="muted">
          Manage your project workspaces and collections.
        </p>
        <ul className="loc-list">
          {workspaces.map((w) => (
            <li key={w.id} className="loc-row">
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <span
                  style={{
                    width: "10px",
                    height: "10px",
                    borderRadius: "50%",
                    backgroundColor: w.color,
                    display: "inline-block",
                  }}
                />
                <b>{w.name}</b>
              </div>
              <button
                className="btn danger"
                onClick={() => setDeleteConfirmWs(w)}
              >
                Delete Workspace
              </button>
            </li>
          ))}
          {workspaces.length === 0 && (
            <li className="muted">No custom workspaces created yet.</li>
          )}
        </ul>
      </section>

      {/* Delete Workspace Confirmation Modal */}
      {deleteConfirmWs && (
        <div className="modal-overlay" onClick={() => setDeleteConfirmWs(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>🗑️ Delete Workspace</h3>
            <p style={{ margin: "14px 0", fontSize: "13.5px" }}>
              Are you sure you want to delete the workspace <b>"{deleteConfirmWs.name}"</b>?
              <br />
              <span className="muted" style={{ fontSize: "12px", display: "inline-block", marginTop: "8px" }}>
                Your project files and folders on disk will <b>not</b> be deleted.
              </span>
            </p>
            <div className="modal-buttons">
              <button type="button" className="btn" onClick={() => setDeleteConfirmWs(null)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn danger"
                onClick={async () => {
                  const id = deleteConfirmWs.id;
                  setDeleteConfirmWs(null);
                  await deleteWorkspace(id);
                }}
              >
                Delete Workspace
              </button>
            </div>
          </div>
        </div>
      )}

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
        <h2>🧭 System Tray & Menu Bar</h2>
        <p className="muted">
          DevDock runs in your macOS Menu Bar / Windows System Tray for instant background access.
        </p>
        <dl className="kv" style={{ marginTop: "10px" }}>
          <dt>Menu Bar Status</dt>
          <dd>🟢 Active (Click tray icon to toggle DevDock)</dd>
          <dt>Quick Actions</dt>
          <dd>Right-click tray icon to show window, hide, or quit</dd>
          <dt>Command Palette</dt>
          <dd>Global search and quick actions via <code>⌘K</code> / <code>Ctrl+K</code></dd>
        </dl>
      </section>

      {/* Hardware & IoT Station Panel */}
      <section className="panel">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <h2>📡 Hardware & IoT Station (Arduino / Raspberry Pi)</h2>
          <button className="btn" style={{ fontSize: "11px", padding: "3px 8px" }} onClick={() => void loadHardware()}>
            🔄 Refresh Ports
          </button>
        </div>
        <p className="muted">
          Synchronize your real-time repository health with physical Arduino or Raspberry Pi LEDs (Green/Yellow/Red traffic station).
        </p>

        <div style={{ marginTop: "14px", display: "flex", flexDirection: "column", gap: "10px" }}>
          <div>
            <label style={{ fontSize: "12px", color: "var(--muted)", display: "block", marginBottom: "4px" }}>
              Hardware USB Serial Port:
            </label>
            <div style={{ display: "flex", gap: "8px" }}>
              <select
                className="select"
                style={{ flex: 1 }}
                value={selectedPort}
                onChange={(e) => setSelectedPort(e.target.value)}
              >
                <option value="">Virtual Hardware Simulator (No device plugged in)</option>
                {hardwarePorts.map((p) => (
                  <option key={p} value={p}>
                    🔌 {p} (Physical USB Device)
                  </option>
                ))}
              </select>
            </div>
            {hardwarePorts.length === 0 && (
              <p className="muted small" style={{ marginTop: "4px" }}>
                No physical serial devices plugged in. Virtual simulator mode active. Plug Arduino via USB and hit Refresh Ports!
              </p>
            )}
          </div>

          {telemetry && (
            <div style={{ background: "var(--bg-raise)", padding: "12px", borderRadius: "8px", border: "1px solid var(--border)" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
                <span style={{ fontSize: "12px", fontWeight: "bold" }}>Current DevDock Telemetry:</span>
                <span style={{
                  fontSize: "11px",
                  fontWeight: "bold",
                  padding: "2px 8px",
                  borderRadius: "999px",
                  background: telemetry.status === "C" ? "rgba(16, 185, 129, 0.2)" : "rgba(245, 158, 11, 0.2)",
                  color: telemetry.status === "C" ? "#10b981" : "#f59e0b",
                  border: `1px solid ${telemetry.status === "C" ? "#10b981" : "#f59e0b"}`
                }}>
                  {telemetry.status === "C" ? "🟢 STATUS: CLEAN (Green LED)" : "🟡 STATUS: DIRTY (Yellow LED)"}
                </span>
              </div>
              <p className="muted small" style={{ margin: 0 }}>
                {telemetry.status_text} • {telemetry.dirty_projects_count} uncommitted repos • {telemetry.active_ports_count} active listening ports
              </p>
            </div>
          )}

          <div>
            <label style={{ fontSize: "12px", color: "var(--muted)", display: "block", marginBottom: "6px" }}>
              Test Hardware LED Signals:
            </label>
            <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
              <button
                className="btn"
                disabled={sendingSignal}
                onClick={() => void handleSendSignal("C")}
                title="Send 'C' to turn on Green LED (Clean)"
              >
                🟢 Test Clean ('C')
              </button>
              <button
                className="btn"
                disabled={sendingSignal}
                onClick={() => void handleSendSignal("D")}
                title="Send 'D' to turn on Yellow LED (Dirty)"
              >
                🟡 Test Dirty ('D')
              </button>
              <button
                className="btn"
                disabled={sendingSignal}
                onClick={() => void handleSendSignal("E")}
                title="Send 'E' to turn on Red LED (Error)"
              >
                🔴 Test Error ('E')
              </button>
              <button
                className="btn"
                disabled={sendingSignal}
                onClick={() => void handleSendSignal("S")}
                title="Send 'S' to blink LEDs (Syncing)"
              >
                ✨ Test Sync Blink ('S')
              </button>
            </div>
          </div>

          {signalLog && (
            <div style={{
              background: "rgba(0, 242, 254, 0.08)",
              border: "1px solid rgba(0, 242, 254, 0.3)",
              padding: "8px 12px",
              borderRadius: "6px",
              fontSize: "12px",
              color: "#38bdf8",
              fontFamily: "monospace"
            }}>
              📡 {signalLog}
            </div>
          )}
        </div>
      </section>

      <section className="panel">
        <h2>Open actions</h2>
        <p className="muted">
          <b>Open Editor</b> opens the first detected editor (Antigravity IDE, Cursor, VS Code, Zed).
          <br />
          <b>Open Terminal</b> opens your preferred terminal (iTerm2, Warp, Ghostty, Terminal.app).
          <br />
          <b>Open Folder</b> reveals the directory in your system file manager.
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