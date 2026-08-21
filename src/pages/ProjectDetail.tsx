import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import type {
  Project,
  ProjectHealth,
  ProjectScript,
  GitHubRepoInfo,
  CustomCommandDto,
  DependencyInfo,
  WorkspaceDto,
  EnvDiffReport,
  GitIgnoreAuditReport,
  RuntimeVersionInfo,
} from "../types";
import { api } from "../services/api";
import { displayPath, gitState, humanSize, timeAgo } from "../lib/format";

export function ProjectDetail() {
  const { id } = useParams();
  const nav = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [health, setHealth] = useState<ProjectHealth | null>(null);
  const [scripts, setScripts] = useState<ProjectScript[]>([]);
  const [deps, setDeps] = useState<DependencyInfo[]>([]);
  const [github, setGithub] = useState<GitHubRepoInfo | null>(null);
  const [tags, setTags] = useState<string[]>([]);
  const [newTag, setNewTag] = useState("");
  const [notes, setNotes] = useState("");
  const [customCmds, setCustomCmds] = useState<CustomCommandDto[]>([]);
  const [allWorkspaces, setAllWorkspaces] = useState<WorkspaceDto[]>([]);
  const [projectWsIds, setProjectWsIds] = useState<number[]>([]);
  const [envReport, setEnvReport] = useState<EnvDiffReport | null>(null);
  const [gitignoreReport, setGitignoreReport] = useState<GitIgnoreAuditReport | null>(null);
  const [runtimeVersions, setRuntimeVersions] = useState<RuntimeVersionInfo[]>([]);
  const [cacheReport, setCacheReport] = useState<import("../types").ProjectCacheReport | null>(null);
  const [cleanTarget, setCleanTarget] = useState<import("../types").CacheFolderInfo | null>(null);
  const [cleanAllConfirm, setCleanAllConfirm] = useState(false);
  const [cleaning, setCleaning] = useState(false);
  const [newCmdName, setNewCmdName] = useState("");
  const [newCmdStr, setNewCmdStr] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    const pid = Number(id);
    void (async () => {
      try {
        const p = await api.getProject(pid);
        setProject(p);
        if (p) {
          const isDirty = gitState(p.git) === "dirty";
          void api.getProjectHealth(p.path, isDirty).then(setHealth);
          void api.listProjectScripts(p.path).then(setScripts);
          void api.getProjectDependencies(p.path).then(setDeps);
          if (p.git?.remote_url) {
            void api.getGithubInfo(p.git.remote_url).then(setGithub);
          }
          void api.getProjectTags(pid).then(setTags);
          void api.getProjectNotes(pid).then((n) => setNotes(n ?? ""));
          void api.listCustomCommands(pid).then(setCustomCmds);
          void api.listWorkspaces().then(setAllWorkspaces);
          void api.getProjectWorkspaces(pid).then(setProjectWsIds);
          void api.checkEnvDiff(p.path).then(setEnvReport);
          void api.checkSecretGitIgnore(p.path).then(setGitignoreReport);
          void api.checkRuntimeVersions(p.path).then(setRuntimeVersions);
          void api.getProjectCacheInfo(p.path).then(setCacheReport);
        }
      } catch (e) {
        setError(String(e));
      }
    })();
  }, [id]);

  const handleCleanFolder = async (folderName: string) => {
    if (!project) return;
    setCleaning(true);
    try {
      await api.cleanCacheFolder(project.path, folderName);
      const rep = await api.getProjectCacheInfo(project.path);
      setCacheReport(rep);
      setCleanTarget(null);
    } catch (e) {
      console.error(e);
    } finally {
      setCleaning(false);
    }
  };

  const handleCleanAllFolders = async () => {
    if (!project || !cacheReport) return;
    setCleaning(true);
    try {
      for (const f of cacheReport.cache_folders) {
        await api.cleanCacheFolder(project.path, f.name);
      }
      const rep = await api.getProjectCacheInfo(project.path);
      setCacheReport(rep);
      setCleanAllConfirm(false);
    } catch (e) {
      console.error(e);
    } finally {
      setCleaning(false);
    }
  };

  const handleAddToGitignore = async (entry: string) => {
    if (!project) return;
    try {
      await api.addToGitIgnore(project.path, entry);
      const rep = await api.checkSecretGitIgnore(project.path);
      setGitignoreReport(rep);
    } catch (e) {
      console.error(e);
    }
  };

  const handleToggleWorkspace = async (wsId: number) => {
    if (!id) return;
    const pid = Number(id);
    const updated = projectWsIds.includes(wsId)
      ? projectWsIds.filter((w) => w !== wsId)
      : [...projectWsIds, wsId];
    setProjectWsIds(updated);
    try {
      await api.setProjectWorkspaces(pid, updated);
    } catch (e) {
      console.error(e);
    }
  };

  const handleAddTag = async () => {
    if (!project || !newTag.trim()) return;
    const t = newTag.trim().toLowerCase();
    await api.addProjectTag(project.id, t);
    setTags((prev) => (prev.includes(t) ? prev : [...prev, t]));
    setNewTag("");
  };

  const handleRemoveTag = async (t: string) => {
    if (!project) return;
    await api.removeProjectTag(project.id, t);
    setTags((prev) => prev.filter((item) => item !== t));
  };

  const handleSaveNotes = async (val: string) => {
    if (!project) return;
    setNotes(val);
    await api.setProjectNotes(project.id, val);
  };

  const handleAddCustomCmd = async () => {
    if (!project || !newCmdName.trim() || !newCmdStr.trim()) return;
    await api.addCustomCommand(project.id, newCmdName.trim(), newCmdStr.trim());
    const list = await api.listCustomCommands(project.id);
    setCustomCmds(list);
    setNewCmdName("");
    setNewCmdStr("");
  };

  const handleRemoveCustomCmd = async (cmdId: number) => {
    if (!project) return;
    await api.removeCustomCommand(cmdId);
    setCustomCmds((prev) => prev.filter((c) => c.id !== cmdId));
  };

  if (error) return <div className="banner error">{error}</div>;
  if (!project) return <div className="empty muted">Loading project…</div>;

  const gs = gitState(project.git);

  return (
    <div className="page detail">
      <div className="detail-top-nav">
        <button className="back-btn" onClick={() => nav("/")}>
          ← Back to Projects
        </button>
      </div>

      <h1 className="detail-title">{project.name}</h1>
      <div className="detail-path">{displayPath(project.path)}</div>

      <div className="detail-actions">
        <button className="btn primary" onClick={() => void api.openEditor(project.path)}>
          Open in Editor
        </button>
        <button className="btn" onClick={() => void api.openTerminal(project.path)}>
          Open Terminal
        </button>
        <button className="btn" onClick={() => void api.openFolder(project.path)}>
          Show in Finder
        </button>
        {github && (
          <>
            <a
              className="btn btn-gh"
              href={github.repo_url}
              target="_blank"
              rel="noreferrer"
            >
              GitHub ↗
            </a>
            <a
              className="btn btn-gh"
              href={github.issues_url}
              target="_blank"
              rel="noreferrer"
            >
              Issues ↗
            </a>
          </>
        )}
      </div>

      <section className="panel">
        <h2>📁 Assigned Workspaces</h2>
        {allWorkspaces.length === 0 ? (
          <p className="muted">No workspaces created yet. Create one from the Dashboard tab bar!</p>
        ) : (
          <div className="tags-row">
            {allWorkspaces.map((w) => {
              const isAssigned = projectWsIds.includes(w.id);
              return (
                <button
                  key={w.id}
                  type="button"
                  className={`ws-chip ${isAssigned ? "assigned" : ""}`}
                  style={{
                    borderColor: isAssigned ? w.color : "transparent",
                    backgroundColor: isAssigned ? `${w.color}22` : undefined,
                  }}
                  onClick={() => void handleToggleWorkspace(w.id)}
                >
                  <span className="ws-dot" style={{ backgroundColor: w.color }} />
                  {w.name} {isAssigned ? "✓" : "+"}
                </button>
              );
            })}
          </div>
        )}
      </section>

      <section className="panel">
        <h2>Tags & Categories</h2>
        <div className="tags-row">
          {tags.map((t) => (
            <span key={t} className="tag-chip">
              {t}
              <button
                className="tag-del"
                onClick={() => void handleRemoveTag(t)}
                title="Remove tag"
              >
                ×
              </button>
            </span>
          ))}
          <div className="tag-input-group">
            <input
              type="text"
              className="tag-input"
              placeholder="+ add tag..."
              value={newTag}
              onChange={(e) => setNewTag(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && void handleAddTag()}
            />
            {newTag.trim() && (
              <button className="btn btn-sm" onClick={() => void handleAddTag()}>
                Add
              </button>
            )}
          </div>
        </div>
      </section>

      <section className="panel">
        <h2>Project Notes & Todo</h2>
        <textarea
          className="notes-textarea"
          placeholder="Write project notes, reminders, or TODOs..."
          value={notes}
          onChange={(e) => void handleSaveNotes(e.target.value)}
          rows={4}
        />
        <div className="notes-footer">Auto-saved to local SQLite database</div>
      </section>

      <section className="panel">
        <h2>Configurable Custom Commands</h2>
        {customCmds.length > 0 && (
          <div className="scripts-grid">
            {customCmds.map((c) => (
              <div key={c.id} className="script-card">
                <div className="script-info">
                  <span className="script-name">{c.name}</span>
                  <span className="script-cmd">{c.command}</span>
                </div>
                <div style={{ display: "flex", gap: "4px" }}>
                  <button
                    className="btn btn-sm"
                    onClick={() => void api.runProjectScript(project.path, c.command)}
                  >
                    Run ↵
                  </button>
                  <button
                    className="btn btn-sm btn-danger"
                    onClick={() => void handleRemoveCustomCmd(c.id)}
                  >
                    ×
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
        <div className="cmd-add-form" style={{ marginTop: "12px" }}>
          <input
            type="text"
            className="input-sm"
            placeholder="Command Name (e.g. dev:watch)"
            value={newCmdName}
            onChange={(e) => setNewCmdName(e.target.value)}
          />
          <input
            type="text"
            className="input-sm"
            placeholder="Shell Command (e.g. cargo watch -x run)"
            value={newCmdStr}
            onChange={(e) => setNewCmdStr(e.target.value)}
          />
          <button className="btn btn-sm primary" onClick={() => void handleAddCustomCmd()}>
            + Add Command
          </button>
        </div>
      </section>

      {health && (
        <section className="panel">
          <h2>Project Health Audit ({health.score}/100)</h2>
          <div className="health-bar-bg">
            <div
              className="health-bar-fill"
              style={{
                width: `${health.score}%`,
                background: health.score >= 80 ? "var(--green)" : "var(--yellow)",
              }}
            />
          </div>
          <dl className="kv" style={{ marginTop: "12px" }}>
            <dt>Status</dt>
            <dd>{health.status}</dd>
            <dt>Dependencies</dt>
            <dd>{health.deps_installed ? "🟢 Installed" : "🔴 Missing / Uninstalled"}</dd>
            <dt>Documentation</dt>
            <dd>{health.has_readme ? "🟢 README Present" : "🟡 Missing README"}</dd>
          </dl>
          {health.issues.length > 0 && (
            <div className="health-issues">
              <strong>Audit Findings:</strong>
              <ul>
                {health.issues.map((issue) => (
                  <li key={issue}>{issue}</li>
                ))}
              </ul>
            </div>
          )}
        </section>
      )}

      {/* Environment Sentinel & Secret Leak Prevention */}
      <section className="panel">
        <h2>🛡️ Environment & Security Sentinel</h2>
        <p className="muted" style={{ marginBottom: "16px" }}>
          Inspect environment template keys, prevent accidental secret leaks in Git, and verify runtime toolchain versions.
        </p>

        {/* 1. Env Diff Checker */}
        <div className="security-subpanel">
          <div className="security-subpanel-header">
            <span className="security-subpanel-title">📄 Environment Variables (.env) Status</span>
            {envReport?.has_template ? (
              envReport.missing_keys.length === 0 ? (
                <span className="badge-clean">✅ All keys synced with {envReport.template_file}</span>
              ) : (
                <span className="badge-dirty">⚠️ {envReport.missing_keys.length} Missing Key(s)</span>
              )
            ) : (
              <span className="muted small">No .env.example template detected</span>
            )}
          </div>

          {envReport?.missing_keys && envReport.missing_keys.length > 0 && (
            <div className="env-missing-box">
              <span className="env-missing-label">Missing keys in local {envReport.local_env_file ?? ".env"}:</span>
              <div className="env-keys-list">
                {envReport.missing_keys.map((k) => (
                  <span key={k} className="env-key-chip missing">
                    {k}
                  </span>
                ))}
              </div>
            </div>
          )}

          {envReport?.has_template && envReport.missing_keys.length === 0 && (
            <div className="env-sync-ok">
              All {envReport.template_keys.length} variable keys from <code>{envReport.template_file}</code> exist in your local environment file. (Values are never inspected for 100% privacy).
            </div>
          )}
        </div>

        {/* 2. GitIgnore Secret Leak Checker */}
        {gitignoreReport && (
          <div className="security-subpanel" style={{ marginTop: "14px" }}>
            <div className="security-subpanel-header">
              <span className="security-subpanel-title">🔒 Git Secret Leak Prevention</span>
              {gitignoreReport.unignored_sensitive_files.length === 0 ? (
                <span className="badge-clean">✅ No unignored secrets found</span>
              ) : (
                <span className="badge-danger">🚨 {gitignoreReport.unignored_sensitive_files.length} Unignored Sensitive File(s)</span>
              )}
            </div>

            {gitignoreReport.unignored_sensitive_files.length > 0 ? (
              <div className="unignored-secrets-list">
                <p className="danger-text small">
                  The following sensitive credential/env files are present in the project directory but are <b>NOT</b> ignored in <code>.gitignore</code>:
                </p>
                {gitignoreReport.unignored_sensitive_files.map((file) => (
                  <div key={file} className="unignored-secret-row">
                    <span className="unignored-file-name">⚠️ {file}</span>
                    <button
                      className="btn btn-sm btn-danger-soft"
                      onClick={() => void handleAddToGitignore(file)}
                    >
                      + Add to .gitignore
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="muted small" style={{ marginTop: "6px" }}>
                {gitignoreReport.sensitive_files_found.length > 0
                  ? `All ${gitignoreReport.sensitive_files_found.length} detected sensitive file(s) are properly excluded by .gitignore.`
                  : "No sensitive key/env files detected in root directory."}
              </div>
            )}
          </div>
        )}

        {/* 3. Runtime Version Mismatch Inspector */}
        {runtimeVersions.length > 0 && (
          <div className="security-subpanel" style={{ marginTop: "14px" }}>
            <div className="security-subpanel-header">
              <span className="security-subpanel-title">⚙️ Toolchain & Runtime Versions</span>
            </div>
            <div className="runtime-versions-grid">
              {runtimeVersions.map((rv) => (
                <div key={rv.toolchain} className="runtime-card">
                  <div className="runtime-header">
                    <b>{rv.toolchain}</b>
                    <span className={rv.is_matched ? "badge-clean" : "badge-dirty"}>
                      {rv.is_matched ? "✅ Version Match" : "⚠️ Version Mismatch"}
                    </span>
                  </div>
                  <div className="runtime-details">
                    <div>Required: <code>{rv.required_version}</code> (via {rv.source_file})</div>
                    <div>Installed: <code>{rv.detected_version ?? "Not detected"}</code></div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </section>

      {/* Disk Usage & Safe Cache Janitor Panel */}
      {cacheReport && (
        <section className="panel">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <h2>🧹 Disk Usage & Cache Janitor</h2>
            {cacheReport.cache_folders.length > 0 && (
              <button
                className="btn btn-sm btn-danger-soft"
                disabled={cleaning}
                onClick={() => setCleanAllConfirm(true)}
              >
                {cleaning ? "Cleaning…" : `🧹 Clean All Cache (${cacheReport.reclaimable_human_size})`}
              </button>
            )}
          </div>
          <p className="muted" style={{ marginBottom: "14px" }}>
            Analyze build cache, package dependencies, and reclaim disk space safely without breaking code.
          </p>

          <div className="cache-overview-cards">
            <div className="cache-metric-card">
              <span className="cache-metric-label">Total Project Footprint</span>
              <span className="cache-metric-value">{cacheReport.total_human_size}</span>
            </div>
            <div className="cache-metric-card highlight">
              <span className="cache-metric-label">Reclaimable Cache</span>
              <span className="cache-metric-value">{cacheReport.reclaimable_human_size}</span>
            </div>
            <div className="cache-metric-card">
              <span className="cache-metric-label">Detected Cache Folders</span>
              <span className="cache-metric-value">{cacheReport.cache_folders.length}</span>
            </div>
          </div>

          {cacheReport.cache_folders.length > 0 ? (
            <div className="cache-folders-list" style={{ marginTop: "14px" }}>
              {cacheReport.cache_folders.map((f) => (
                <div key={f.name} className="cache-folder-row">
                  <div className="cache-folder-info">
                    <span className="cache-folder-name">📁 {f.name}</span>
                    <span className="cache-folder-desc">{f.category}</span>
                  </div>
                  <div className="cache-folder-actions">
                    <span className="cache-folder-size">{f.human_size}</span>
                    <button
                      className="btn btn-sm danger"
                      disabled={cleaning}
                      onClick={() => setCleanTarget(f)}
                    >
                      Clean
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="muted small" style={{ marginTop: "10px" }}>
              ✨ Clean! No heavy build cache or dependency directories detected.
            </div>
          )}
        </section>
      )}

      {/* Confirmation Modal for Single Cache Clean */}
      {cleanTarget && (
        <div className="modal-overlay" onClick={() => setCleanTarget(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>🧹 Clean Cache: {cleanTarget.name}</h3>
            <p style={{ margin: "14px 0", fontSize: "13.5px" }}>
              Are you sure you want to delete <b>`{cleanTarget.name}`</b> ({cleanTarget.human_size})?
              <br />
              <span className="muted" style={{ fontSize: "12px", display: "inline-block", marginTop: "8px" }}>
                This is a build artifact / cache folder ({cleanTarget.category}). It can be regenerated anytime by running your project's install or build command.
              </span>
            </p>
            <div className="modal-buttons">
              <button type="button" className="btn" disabled={cleaning} onClick={() => setCleanTarget(null)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn danger"
                disabled={cleaning}
                onClick={() => void handleCleanFolder(cleanTarget.name)}
              >
                {cleaning ? "Cleaning…" : `Delete ${cleanTarget.name}`}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Confirmation Modal for Clean All Cache */}
      {cleanAllConfirm && cacheReport && (
        <div className="modal-overlay" onClick={() => setCleanAllConfirm(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>🧹 Reclaim All Cache ({cacheReport.reclaimable_human_size})</h3>
            <p style={{ margin: "14px 0", fontSize: "13.5px" }}>
              This will remove all {cacheReport.cache_folders.length} cache folders in this project:
              <br />
              <b>{cacheReport.cache_folders.map((f) => f.name).join(", ")}</b>
              <br />
              <span className="muted" style={{ fontSize: "12px", display: "inline-block", marginTop: "8px" }}>
                You will reclaim <b>{cacheReport.reclaimable_human_size}</b> of disk space. Source code is never affected.
              </span>
            </p>
            <div className="modal-buttons">
              <button type="button" className="btn" disabled={cleaning} onClick={() => setCleanAllConfirm(false)}>
                Cancel
              </button>
              <button
                type="button"
                className="btn danger"
                disabled={cleaning}
                onClick={() => void handleCleanAllFolders()}
              >
                {cleaning ? "Cleaning All…" : "Clean All Cache Folders"}
              </button>
            </div>
          </div>
        </div>
      )}

      {scripts.length > 0 && (
        <section className="panel">
          <h2>Detected Scripts & Tasks ({scripts.length})</h2>
          <div className="scripts-grid">
            {scripts.map((s) => (
              <div key={`${s.source}-${s.name}`} className="script-card">
                <div className="script-info">
                  <span className="script-name">{s.name}</span>
                  <span className="script-cmd">{s.command}</span>
                </div>
                <button
                  className="btn btn-sm"
                  onClick={() => void api.runProjectScript(project.path, s.command)}
                >
                  Run ↵
                </button>
              </div>
            ))}
          </div>
        </section>
      )}

      {deps.length > 0 && (
        <section className="panel">
          <h2>Dependencies & Libraries ({deps.length})</h2>
          <div className="deps-grid">
            {deps.map((d) => (
              <div key={d.name} className="dep-chip">
                <span className="dep-name">{d.name}</span>
                <span className="dep-ver">{d.version}</span>
                {d.is_dev && <span className="dep-dev-badge">dev</span>}
              </div>
            ))}
          </div>
        </section>
      )}

      <section className="panel">
        <h2>Overview</h2>
        <dl className="kv">
          <dt>Path</dt>
          <dd>{project.path}</dd>
          <dt>Size</dt>
          <dd>{humanSize(project.size_bytes)}</dd>
          <dt>Last modified</dt>
          <dd>{timeAgo(project.last_modified)}</dd>
          <dt>Technologies</dt>
          <dd>
            {project.techs.map((t) => (
              <span key={t.name} className={`chip chip-${t.kind}`}>
                {t.name}
              </span>
            ))}
          </dd>
        </dl>
      </section>

      <section className="panel">
        <h2>Git Metadata</h2>
        {!project.git?.is_git ? (
          <p className="muted">Not a git repository.</p>
        ) : (
          <dl className="kv">
            <dt>Status</dt>
            <dd>
              <span className={`dot dot-${gs}`} />
              {gs === "clean" ? "Clean" : "Dirty"}
            </dd>
            <dt>Branch</dt>
            <dd>{project.git.branch ?? "detached"}</dd>
            <dt>Staged</dt>
            <dd>{project.git.staged_count}</dd>
            <dt>Modified</dt>
            <dd>{project.git.modified_count}</dd>
            <dt>Untracked</dt>
            <dd>{project.git.untracked_count}</dd>
            {project.git.last_commit_message && (
              <>
                <dt>Last commit</dt>
                <dd>{project.git.last_commit_message}</dd>
              </>
            )}
            {project.git.remote_url && (
              <>
                <dt>Remote</dt>
                <dd className="mono">{project.git.remote_url}</dd>
              </>
            )}
          </dl>
        )}
      </section>
    </div>
  );
}