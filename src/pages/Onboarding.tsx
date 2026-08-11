import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useProjectsStore } from "../stores/projectsStore";
import { useScanStore } from "../stores/scanStore";

export function Onboarding() {
  const nav = useNavigate();
  const { locations, addLocation, load } = useProjectsStore();
  const scanStore = useScanStore();
  const [dir, setDir] = useState("");
  const [started, setStarted] = useState(false);

  const startScan = async () => {
    setStarted(true);
    const summary = await scanStore.start();
    await load();
    if (summary && summary.total > 0) {
      nav("/");
    } else {
      setStarted(false);
    }
  };

  return (
    <div className="onboard">
      <div className="onboard-box">
        <div className="logo">D</div>
        <h1>Welcome to DevDock</h1>
        <p className="muted">Let's find your projects.</p>
        <p className="muted small">
          Choose directories where you keep your development projects.
        </p>

        <div className="add-row">
          <input
            className="search grow"
            placeholder="~/Projects  onaylı yol girin, ör: /Users/ardatekin"
            value={dir}
            onChange={(e) => setDir(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && dir.trim() && void addLocation(dir.trim()).then(() => setDir(""))}
          />
          <button
            className="btn"
            onClick={() => dir.trim() && void addLocation(dir.trim()).then(() => setDir(""))}
          >
            Add Folder
          </button>
        </div>

        {locations.length > 0 && (
          <ul className="loc-list compact">
            {locations.map((l) => (
              <li key={l.id} className="loc-row">
                <code>{l.path}</code>
              </li>
            ))}
          </ul>
        )}

        {scanStore.error && <div className="banner error">{scanStore.error}</div>}

        <button
          className="btn primary big"
          disabled={locations.length === 0 || scanStore.running || started}
          onClick={() => void startScan()}
        >
          {scanStore.running || started ? "Scanning…" : "Scan"}
        </button>
      </div>
    </div>
  );
}