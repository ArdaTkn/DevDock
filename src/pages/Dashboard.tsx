import { useEffect, useMemo, useState } from "react";
import { ProjectCard } from "../components/ProjectCard";
import { useProjectsStore, SortKey } from "../stores/projectsStore";
import { useScanStore } from "../stores/scanStore";
import { useSystemStore } from "../stores/systemStore";
import { allTechs } from "../lib/format";
import { api } from "../services/api";
import type { BulkGitResult, BulkGitStatusResult } from "../types";

export function Dashboard() {
  const {
    projects,
    recentProjects,
    workspaces,
    activeWorkspaceId,
    workspaceProjectIds,
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
    loadWorkspaces,
    createWorkspace,
    deleteWorkspace,
    setActiveWorkspaceId,
  } = useProjectsStore();
  const scanStore = useScanStore();
  const { ports, loadPorts } = useSystemStore();

  const [showWsModal, setShowWsModal] = useState(false);
  const [wsName, setWsName] = useState("");
  const [wsColor, setWsColor] = useState("#10b981");
  const [bulkPulling, setBulkPulling] = useState(false);
  const [bulkPullResults, setBulkPullResults] = useState<BulkGitResult[] | null>(null);
  const [bulkStatusList, setBulkStatusList] = useState<BulkGitStatusResult[] | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      useScanStore.getState().listen();
      void useProjectsStore.getState().listenWatcher();
      void loadPorts();
      void loadWorkspaces();
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
    if (activeWorkspaceId !== null) {
      list = list.filter((p) => workspaceProjectIds.includes(p.id));
    }
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
  }, [projects, activeWorkspaceId, workspaceProjectIds, search, techFilter, sort]);

  const sorting: Record<SortKey, string> = {
    recent: "Recently modified",
    name: "Name",
    dirty: "Most uncommitted",
    path: "Path",
  };

  const running = scanStore.running;

  const handleCreateWorkspace = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!wsName.trim()) return;
    await createWorkspace(wsName.trim(), wsColor);
    setWsName("");
    setShowWsModal(false);
  };

  const handleBulkPull = async () => {
    const paths = filtered.filter((p) => p.git?.is_git).map((p) => p.path);
    if (paths.length === 0) return;
    setBulkPulling(true);
    setBulkPullResults(null);
    try {
      const res = await api.bulkGitPull(paths);
      setBulkPullResults(res);
      await refresh();
    } catch (e) {
      console.error(e);
    } finally {
      setBulkPulling(false);
    }
  };

  const handleBulkStatusAudit = async () => {
    const paths = filtered.filter((p) => p.git?.is_git).map((p) => p.path);
    if (paths.length === 0) return;
    try {
      const res = await api.bulkGitStatus(paths);
      setBulkStatusList(res);
    } catch (e) {
      console.error(e);
    }
  };

  const activeWorkspace = workspaces.find((w) => w.id === activeWorkspaceId);

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
          <span className="count">{filtered.length} projects</span>
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

      {/* Workspaces Tab Bar */}
      <div className="workspaces-bar">
        <div className="workspaces-tabs">
          <button
            className={`ws-tab ${activeWorkspaceId === null ? "active" : ""}`}
            onClick={() => void setActiveWorkspaceId(null)}
          >
            🌐 All Projects ({projects.length})
          </button>
          {workspaces.map((w) => (
            <div key={w.id} className="ws-tab-wrapper">
              <button
                className={`ws-tab ${activeWorkspaceId === w.id ? "active" : ""}`}
                onClick={() => void setActiveWorkspaceId(w.id)}
              >
                <span className="ws-dot" style={{ backgroundColor: w.color }} />
                {w.name}
              </button>
              {activeWorkspaceId === w.id && (
                <button
                  className="ws-delete-btn"
                  title="Delete this workspace"
                  onClick={(e) => {
                    e.stopPropagation();
                    if (confirm(`Delete workspace "${w.name}"? (Projects will not be deleted)`)) {
                      void deleteWorkspace(w.id);
                    }
                  }}
                >
                  ✕
                </button>
              )}
            </div>
          ))}
          <button className="ws-tab-add" onClick={() => setShowWsModal(true)}>
            + Add Workspace
          </button>
        </div>

        {/* Bulk Git Actions Bar */}
        <div className="ws-actions">
          <button
            className="btn ws-action-btn"
            disabled={bulkPulling || filtered.length === 0}
            onClick={() => void handleBulkPull()}
            title="Git pull all projects in current view"
          >
            {bulkPulling ? "Pulling…" : "⬇️ Pull All"}
          </button>
          <button
            className="btn ws-action-btn"
            disabled={filtered.length === 0}
            onClick={() => void handleBulkStatusAudit()}
            title="Audit uncommitted changes across all workspace projects"
          >
            📋 Git Audit
          </button>
        </div>
      </div>

      {/* New Workspace Modal */}
      {showWsModal && (
        <div className="modal-overlay" onClick={() => setShowWsModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>📁 Create New Workspace</h3>
            <form onSubmit={(e) => void handleCreateWorkspace(e)}>
              <div className="form-group">
                <label>Workspace Name:</label>
                <input
                  className="input"
                  placeholder="e.g. Client Work, Open Source, Microservices"
                  value={wsName}
                  onChange={(e) => setWsName(e.target.value)}
                  autoFocus
                  required
                />
              </div>
              <div className="form-group">
                <label>Badge Color:</label>
                <div className="color-picker-row">
                  {["#10b981", "#06b6d4", "#8b5cf6", "#ec4899", "#f59e0b", "#3b82f6", "#ef4444"].map((c) => (
                    <button
                      type="button"
                      key={c}
                      className={`color-chip ${wsColor === c ? "selected" : ""}`}
                      style={{ backgroundColor: c }}
                      onClick={() => setWsColor(c)}
                    />
                  ))}
                </div>
              </div>
              <div className="modal-buttons">
                <button type="button" className="btn" onClick={() => setShowWsModal(false)}>
                  Cancel
                </button>
                <button type="submit" className="btn primary">
                  Create Workspace
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Bulk Pull Results Modal */}
      {bulkPullResults && (
        <div className="modal-overlay" onClick={() => setBulkPullResults(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>⬇️ Bulk Git Pull Results ({bulkPullResults.length} repos)</h3>
            <div className="bulk-results-list">
              {bulkPullResults.map((r, i) => (
                <div key={i} className={`bulk-item ${r.success ? "success" : "failed"}`}>
                  <span className="bulk-path">{r.path.split("/").pop()}</span>
                  <span className="bulk-msg">{r.message || (r.success ? "Updated" : "Failed")}</span>
                </div>
              ))}
            </div>
            <div className="modal-buttons">
              <button className="btn primary" onClick={() => setBulkPullResults(null)}>
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Bulk Git Audit Modal */}
      {bulkStatusList && (
        <div className="modal-overlay" onClick={() => setBulkStatusList(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>📋 Workspace Git Audit ({bulkStatusList.length} Repos)</h3>
            <div className="bulk-results-list">
              {bulkStatusList.map((s, i) => (
                <div key={i} className={`bulk-item ${s.is_dirty ? "dirty" : "clean"}`}>
                  <span className="bulk-path">
                    {s.path.split("/").pop()} <span className="bulk-branch">({s.branch})</span>
                  </span>
                  <span className={`bulk-badge ${s.is_dirty ? "badge-dirty" : "badge-clean"}`}>
                    {s.is_dirty ? `⚠️ ${s.uncommitted_count} uncommitted files` : "✅ Clean"}
                  </span>
                </div>
              ))}
            </div>
            <div className="modal-buttons">
              <button className="btn primary" onClick={() => setBulkStatusList(null)}>
                Close Audit
              </button>
            </div>
          </div>
        </div>
      )}

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

      {!running && !loading && filtered.length === 0 && !error && (
        <div className="empty">
          {activeWorkspace
            ? `No projects assigned to workspace "${activeWorkspace.name}" yet.`
            : "No projects found."}
          <br />
          <span className="muted">
            {activeWorkspace
              ? "Open any project detail page to assign it to this workspace!"
              : "Press Rescan, or add a directory in Settings if your projects live outside your home folder."}
          </span>
        </div>
      )}

      {techs.length > 0 && (
        <div className="chips-row">
          <button
            className={`chip-btn ${techFilter === null ? "active" : ""}`}
            onClick={() => setTechFilter(null)}
          >
            All Techs
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
            <span className="ports-title">🟢 Active Local Dev Servers & Services ({ports.length})</span>
            <span className="ports-subtitle">Click to open localhost in browser</span>
          </div>
          <div className="ports-grid">
            {ports.map((p) => (
              <a
                key={`port-${p.port}`}
                className="port-card"
                href={`http://localhost:${p.port}`}
                target="_blank"
                rel="noreferrer"
                title={`Open http://localhost:${p.port} in browser (PID ${p.pid})`}
              >
                <div className="port-card-top">
                  <span className="port-pulse-dot" />
                  <span className="port-num">:{p.port}</span>
                  <span className="port-pid">PID {p.pid}</span>
                </div>
                <div className="port-card-bottom">
                  <span className="port-label">{p.label}</span>
                  <span className="port-open-icon">↗</span>
                </div>
              </a>
            ))}
          </div>
        </section>
      )}

      {recentProjects.length > 0 && !search && techFilter === null && activeWorkspaceId === null && (
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