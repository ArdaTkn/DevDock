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
  const [deleteConfirmWs, setDeleteConfirmWs] = useState<import("../types").WorkspaceDto | null>(null);
  const [bulkPulling, setBulkPulling] = useState(false);
  const [bulkAuditing, setBulkAuditing] = useState(false);
  const [bulkPullResults, setBulkPullResults] = useState<BulkGitResult[] | null>(null);
  const [bulkStatusList, setBulkStatusList] = useState<BulkGitStatusResult[] | null>(null);
  const [diskHogsReport, setDiskHogsReport] = useState<import("../types").DiskHogReport | null>(null);
  const [confirmCleanItem, setConfirmCleanItem] = useState<import("../types").DiskHogItem | null>(null);
  const [confirmCleanAllStale, setConfirmCleanAllStale] = useState(false);
  const [scanningHogs, setScanningHogs] = useState(false);
  const [cleaningHog, setCleaningHog] = useState(false);
  const [favoritesOnly, setFavoritesOnly] = useState(false);

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
    if (favoritesOnly) {
      list = list.filter((p) => p.is_favorite);
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
      case "favorites":
        list = [...list].sort((a, b) => {
          if (a.is_favorite === b.is_favorite) {
            return b.last_modified - a.last_modified;
          }
          return a.is_favorite ? -1 : 1;
        });
        break;
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
  }, [projects, activeWorkspaceId, workspaceProjectIds, favoritesOnly, search, techFilter, sort]);

  const sorting: Record<SortKey, string> = {
    favorites: "⭐ Favorites first",
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
    setBulkAuditing(true);
    setBulkStatusList(null);
    try {
      const res = await api.bulkGitStatus(paths);
      setBulkStatusList(res);
    } catch (e) {
      console.error(e);
    } finally {
      setBulkAuditing(false);
    }
  };

  const handleScanDiskHogs = async () => {
    const paths = filtered.map((p) => p.path);
    if (paths.length === 0) return;
    setScanningHogs(true);
    setDiskHogsReport(null);
    try {
      const res = await api.getDiskHogsReport(paths);
      setDiskHogsReport(res);
    } catch (e) {
      console.error(e);
    } finally {
      setScanningHogs(false);
    }
  };

  const handleCleanHogFolder = async (projectPath: string, folderName: string) => {
    setCleaningHog(true);
    try {
      await api.cleanCacheFolder(projectPath, folderName);
      const paths = filtered.map((p) => p.path);
      const res = await api.getDiskHogsReport(paths);
      setDiskHogsReport(res);
    } catch (e) {
      console.error(e);
    } finally {
      setCleaningHog(false);
    }
  };

  const handleCleanAllStaleCaches = async () => {
    if (!diskHogsReport) return;
    setCleaningHog(true);
    try {
      const staleItems = diskHogsReport.items.filter((item) => item.is_stale);
      for (const item of staleItems) {
        for (const f of item.cache_folders) {
          await api.cleanCacheFolder(item.project_path, f.name);
        }
      }
      const paths = filtered.map((p) => p.path);
      const res = await api.getDiskHogsReport(paths);
      setDiskHogsReport(res);
    } catch (e) {
      console.error(e);
    } finally {
      setCleaningHog(false);
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
            <button
              key={w.id}
              className={`ws-tab ${activeWorkspaceId === w.id ? "active" : ""}`}
              onClick={() => void setActiveWorkspaceId(w.id)}
            >
              <span className="ws-dot" style={{ backgroundColor: w.color }} />
              {w.name}
            </button>
          ))}
          <button className="ws-tab-add" onClick={() => setShowWsModal(true)}>
            + Add Workspace
          </button>
        </div>

        {/* Bulk Git Actions & Workspace Actions Bar */}
        <div className="ws-actions">
          {activeWorkspace && (
            <button
              className="btn btn-danger-soft ws-action-btn"
              onClick={() => setDeleteConfirmWs(activeWorkspace)}
              title={`Delete workspace "${activeWorkspace.name}"`}
            >
              🗑️ Delete Workspace
            </button>
          )}
          <button
            className="btn ws-action-btn"
            disabled={bulkPulling || bulkAuditing || filtered.length === 0}
            onClick={() => void handleBulkPull()}
            title="Git pull all projects in current view"
          >
            {bulkPulling ? (
              <>
                <span className="spinner-mini" /> Pulling…
              </>
            ) : (
              "⬇️ Pull All"
            )}
          </button>
          <button
            className="btn ws-action-btn"
            disabled={bulkPulling || bulkAuditing || scanningHogs || filtered.length === 0}
            onClick={() => void handleBulkStatusAudit()}
            title="Audit uncommitted changes across all workspace projects"
          >
            {bulkAuditing ? (
              <>
                <span className="spinner-mini" /> Auditing…
              </>
            ) : (
              "📋 Git Audit"
            )}
          </button>
          <button
            className="btn ws-action-btn"
            disabled={bulkPulling || bulkAuditing || scanningHogs || filtered.length === 0}
            onClick={() => void handleScanDiskHogs()}
            title="Scan build cache and find biggest disk space hogs"
          >
            {scanningHogs ? (
              <>
                <span className="spinner-mini" /> Scanning…
              </>
            ) : (
              "🧹 Disk Janitor"
            )}
          </button>
        </div>
      </div>

      {/* Disk Space Hog & Janitor Modal */}
      {diskHogsReport && (
        <div className="modal-overlay" onClick={() => setDiskHogsReport(null)}>
          <div className="modal-content hog-modal" onClick={(e) => e.stopPropagation()}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <h3>🧹 Disk Space Hog Visualizer</h3>
              {diskHogsReport.stale_projects_count > 0 && (
                <button
                  className="btn btn-sm btn-danger-soft"
                  disabled={cleaningHog}
                  onClick={() => setConfirmCleanAllStale(true)}
                >
                  {cleaningHog ? "Cleaning…" : `Clean All Stale (${diskHogsReport.stale_projects_count} projects)`}
                </button>
              )}
            </div>
            <p className="muted" style={{ fontSize: "12px", margin: "6px 0 14px 0" }}>
              Scanned <b>{filtered.length}</b> projects — Total reclaimable cache across workspace: <b>{diskHogsReport.total_reclaimable_human_size}</b>.
            </p>

            <div className="hog-summary-cards">
              <div className="hog-stat-card">
                <span className="hog-stat-label">Reclaimable Storage</span>
                <span className="hog-stat-value highlight">{diskHogsReport.total_reclaimable_human_size}</span>
              </div>
              <div className="hog-stat-card">
                <span className="hog-stat-label">Stale Projects (&gt;90d)</span>
                <span className="hog-stat-value">{diskHogsReport.stale_projects_count}</span>
              </div>
              <div className="hog-stat-card">
                <span className="hog-stat-label">Projects with Cache</span>
                <span className="hog-stat-value">{diskHogsReport.items.length}</span>
              </div>
            </div>

            <div className="hog-list">
              {diskHogsReport.items.map((item) => (
                <div key={item.project_path} className={`hog-row ${item.is_stale ? "stale" : ""}`}>
                  <div className="hog-info">
                    <div className="hog-name-row">
                      <span className="hog-name">{item.project_name}</span>
                      {item.is_stale && <span className="badge-stale">⏰ Stale (&gt;90d inactive)</span>}
                    </div>
                    <span className="hog-path">{item.project_path}</span>
                    <div className="hog-chips">
                      {item.cache_folders.map((f) => (
                        <span key={f.name} className="hog-chip">
                          {f.name} ({f.human_size})
                        </span>
                      ))}
                    </div>
                  </div>
                  <div className="hog-action-group">
                    <span className="hog-reclaim-badge">{item.reclaimable_human_size}</span>
                    <button
                      className="btn btn-sm danger"
                      disabled={cleaningHog}
                      onClick={() => setConfirmCleanItem(item)}
                    >
                      Clean Cache
                    </button>
                  </div>
                </div>
              ))}
              {diskHogsReport.items.length === 0 && (
                <div className="muted" style={{ textAlign: "center", padding: "20px" }}>
                  🎉 No heavy cache folders detected! Your disk is already optimized.
                </div>
              )}
            </div>

            <div className="modal-buttons" style={{ marginTop: "16px" }}>
              <button className="btn primary" onClick={() => setDiskHogsReport(null)}>
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Confirmation Modal for Cleaning Single Project Cache */}
      {confirmCleanItem && (
        <div className="modal-overlay" onClick={() => setConfirmCleanItem(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>🧹 Clean Cache: {confirmCleanItem.project_name}</h3>
            <p style={{ margin: "12px 0 6px 0", fontSize: "13.5px" }}>
              Are you sure you want to clean build cache for <b>{confirmCleanItem.project_name}</b>?
            </p>
            <div className="unignored-secrets-list" style={{ margin: "10px 0" }}>
              <span className="muted small" style={{ display: "block", marginBottom: "6px" }}>
                Folders to be removed ({confirmCleanItem.reclaimable_human_size} total):
              </span>
              {confirmCleanItem.cache_folders.map((f) => (
                <div key={f.name} className="unignored-secret-row">
                  <span>📁 <b>{f.name}</b> ({f.category})</span>
                  <span style={{ color: "#ef4444", fontWeight: "bold" }}>{f.human_size}</span>
                </div>
              ))}
            </div>
            <p className="muted" style={{ fontSize: "12px", marginTop: "10px" }}>
              🛡️ <b>100% Safe:</b> Source code and Git history are never touched. These folders can be re-downloaded or rebuilt anytime.
            </p>
            <div className="modal-buttons" style={{ marginTop: "16px" }}>
              <button type="button" className="btn" disabled={cleaningHog} onClick={() => setConfirmCleanItem(null)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn danger"
                disabled={cleaningHog}
                onClick={async () => {
                  const target = confirmCleanItem;
                  setConfirmCleanItem(null);
                  for (const f of target.cache_folders) {
                    await handleCleanHogFolder(target.project_path, f.name);
                  }
                }}
              >
                {cleaningHog ? "Cleaning…" : `Yes, Free ${confirmCleanItem.reclaimable_human_size}`}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Confirmation Modal for Cleaning All Stale Projects Cache */}
      {confirmCleanAllStale && diskHogsReport && (
        <div className="modal-overlay" onClick={() => setConfirmCleanAllStale(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>⏰ Clean Stale Projects Cache</h3>
            <p style={{ margin: "12px 0", fontSize: "13.5px" }}>
              Are you sure you want to clean build caches for all <b>{diskHogsReport.stale_projects_count} stale projects</b> untouched for over 90 days?
            </p>
            <p className="muted" style={{ fontSize: "12px" }}>
              🛡️ <b>100% Safe:</b> Source code is completely unaffected. You will reclaim gigabytes of disk space safely.
            </p>
            <div className="modal-buttons" style={{ marginTop: "16px" }}>
              <button type="button" className="btn" disabled={cleaningHog} onClick={() => setConfirmCleanAllStale(false)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn danger"
                disabled={cleaningHog}
                onClick={async () => {
                  setConfirmCleanAllStale(false);
                  await handleCleanAllStaleCaches();
                }}
              >
                {cleaningHog ? "Cleaning…" : "Clean All Stale Projects"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Interactive Bulk Operation Loading Modal */}
      {(bulkPulling || bulkAuditing) && (
        <div className="modal-overlay">
          <div className="modal-content loading-modal">
            <div className="loading-spinner-large" />
            <h3>
              {bulkPulling ? "⬇️ Synchronizing Git Repositories..." : "📋 Auditing Workspace Repositories..."}
            </h3>
            <p className="muted">
              {bulkPulling
                ? `Running git pull across ${filtered.filter((p) => p.git?.is_git).length} repositories. Fetching latest remote branches and updating working tree...`
                : `Inspecting branches, modified files, and uncommitted changes across ${filtered.filter((p) => p.git?.is_git).length} repositories...`}
            </p>
            <div className="loading-pulse-bar">
              <div className="loading-pulse-fill" />
            </div>
            <div className="loading-tip">This runs asynchronously in the background without freezing the UI.</div>
          </div>
        </div>
      )}

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
            className={`chip-btn ${!favoritesOnly && techFilter === null ? "active" : ""}`}
            onClick={() => {
              setFavoritesOnly(false);
              setTechFilter(null);
            }}
          >
            All Techs
          </button>
          <button
            className={`chip-btn ${favoritesOnly ? "active star-active" : ""}`}
            onClick={() => {
              setFavoritesOnly((prev) => !prev);
              if (!favoritesOnly) setTechFilter(null);
            }}
            title="Filter to show only starred/favorite projects"
          >
            ⭐ Starred ({projects.filter((p) => p.is_favorite).length})
          </button>
          {techs.map((t) => (
            <button
              key={t}
              className={`chip-btn ${!favoritesOnly && techFilter === t ? "active" : ""}`}
              onClick={() => {
                setFavoritesOnly(false);
                setTechFilter(techFilter === t ? null : t);
              }}
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