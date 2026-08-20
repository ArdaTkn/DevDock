import { useEffect, useMemo } from "react";
import { ProjectCard } from "../components/ProjectCard";
import { useProjectsStore, SortKey } from "../stores/projectsStore";
import { useScanStore } from "../stores/scanStore";
import { useSystemStore } from "../stores/systemStore";
import { allTechs } from "../lib/format";
import { api } from "../services/api";

export function Dashboard() {
  const {
    projects,
    recentProjects,
    loading,
    error,
    search,
    techFilter,
    sort,
    setSearch,
    setTechFilter,
    setSort,
    refresh,
    load,
  } = useProjectsStore();
  const scanStore = useScanStore();
  const { ports, loadPorts } = useSystemStore();

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      useScanStore.getState().listen();
      void useProjectsStore.getState().listenWatcher();
      void loadPorts();
      await load();
      if (cancelled) return;
      await useScanStore.getState().ensureScan();
      if (!cancelled) await refresh();
    })();

    const interval = setInterval(() => {
      void loadPorts();
    }, 5000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const techs = useMemo(() => allTechs(projects), [projects]);

  const filtered = useMemo(() => {
    let list = projects;
    if (techFilter) list = list.filter((p) => p.techs.some((t) => t.name === techFilter));
    const q = search.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          p.path.toLowerCase().includes(q) ||
          p.techs.some((t) => t.name.toLowerCase().includes(q)),
      );
    }
    switch (sort) {
      case "name":
        list = [...list].sort((a, b) => a.name.localeCompare(b.name));
        break;
      case "dirty":
        list = [...list].sort((a, b) => {
          const da = (a.git?.modified_count ?? 0) + (a.git?.untracked_count ?? 0);
          const db = (b.git?.modified_count ?? 0) + (b.git?.untracked_count ?? 0);
          return db - da;
        });
        break;
      case "path":
        list = [...list].sort((a, b) => a.path.localeCompare(b.path));
        break;
      case "recent":
      default:
        list = [...list].sort((a, b) => b.last_modified - a.last_modified);
    }
    return list;
  }, [projects, search, techFilter, sort]);

  const sorting: Record<SortKey, string> = {
    recent: "Recently modified",
    name: "Name",
    dirty: "Most uncommitted",
    path: "Path",
  };

  const running = scanStore.running;

  return (
    <div className="page">
      <header className="toolbar">
        <div className="toolbar-left">
          <input
            className="search"
            placeholder="Search projects…  (flutter, docker, my-app)"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <select
            className="select"
            value={sort}
            onChange={(e) => setSort(e.target.value as SortKey)}
            aria-label="sort"
          >
            {Object.entries(sorting).map(([k, v]) => (
              <option key={k} value={k}>
                {v}
              </option>
            ))}
          </select>
        </div>
        <div className="toolbar-right">
          <span className="count">{projects.length} projects</span>
          <button
            className="btn primary"
            disabled={running}
            onClick={async () => {
              const summary = await scanStore.start();
              if (summary) await refresh();
            }}
          >
            {running ? "Scanning…" : "Rescan"}
          </button>
        </div>
      </header>

      {error && <div className="banner error">{error}</div>}

      {scanStore.summary && !running && (
        <div className="banner summary">
          Found <b>{scanStore.summary.total}</b> projects —{" "}
          <b>{scanStore.summary.dirty_count}</b> with uncommitted changes,{" "}
          <b>{scanStore.summary.clean_count}</b> clean.
        </div>
      )}

      {running && (
        <div className="scan-panel">
          <div className="scan-top">
            <span className="spinner" aria-hidden="true" />
            <span className="scan-label">Scanning your projects…</span>
          </div>
          {scanStore.progress && (
            <>
              <div className="scan-bar">
                <div
                  className="scan-bar-fill"
                  style={{
                    width: scanStore.progress.total > 0
                      ? `${Math.min(100, Math.round((scanStore.progress.scanned / scanStore.progress.total) * 100))}%`
                      : "4%",
                  }}
                />
              </div>
              <div className="scan-meta">
                <span>
                  {scanStore.progress.scanned} / {scanStore.progress.total} projects
                  {scanStore.progress.found > 0 && ` · ${scanStore.progress.found} found`}
                </span>
                <span className="scan-path" title={scanStore.progress.current_path}>
                  {scanStore.progress.current_path}
                </span>
              </div>
            </>
          )}
          <button className="btn" onClick={() => void scanStore.cancel()}>
            Cancel
          </button>
        </div>
      )}

      {!running && loading && projects.length === 0 && (
        <div className="empty">
          <span className="spinner" aria-hidden="true" /> Loading projects…
        </div>
      )}

      {!running && !loading && projects.length === 0 && !error && (
        <div className="empty">
          No projects found.
          <br />
          <span className="muted">
            Press Rescan, or add a directory in Settings if your projects live outside your home folder.
          </span>
        </div>
      )}

      {techs.length > 0 && (
        <div className="chips-row">
          <button
            className={`chip-btn ${techFilter === null ? "active" : ""}`}
            onClick={() => setTechFilter(null)}
          >
            All
          </button>
          {techs.map((t) => (
            <button
              key={t}
              className={`chip-btn ${techFilter === t ? "active" : ""}`}
              onClick={() => setTechFilter(techFilter === t ? null : t)}
            >
              {t}
            </button>
          ))}
        </div>
      )}

      {ports.length > 0 && (
        <section className="ports-section">
          <div className="ports-header">
            <span className="ports-title">🟢 Active Local Servers & Ports ({ports.length})</span>
          </div>
          <div className="ports-row">
            {ports.map((p) => (
              <a
                key={`port-${p.port}`}
                className="port-chip"
                href={`http://localhost:${p.port}`}
                target="_blank"
                rel="noreferrer"
                title={`Open http://localhost:${p.port} in browser (PID ${p.pid})`}
              >
                <span className="port-dot">●</span>
                <span className="port-num">:{p.port}</span>
                <span className="port-label">{p.label}</span>
              </a>
            ))}
          </div>
        </section>
      )}

      {recentProjects.length > 0 && !search && techFilter === null && (
        <section className="recents-section">
          <div className="recents-header">
            <span className="recents-title">Recently Opened</span>
          </div>
          <div className="recents-row">
            {recentProjects.map((p) => (
              <div key={`recent-${p.id}`} className="recent-chip" onClick={() => void api.openEditor(p.path)}>
                <span className="recent-icon">📁</span>
                <span className="recent-name">{p.name}</span>
                <span className="recent-act">Editor ↵</span>
              </div>
            ))}
          </div>
        </section>
      )}

      <div className="card-grid">
        {filtered.map((p) => (
          <ProjectCard key={p.id} project={p} />
        ))}
      </div>
    </div>
  );
}