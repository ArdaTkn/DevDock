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
        }
      } catch (e) {
        setError(String(e));
      }
    })();
  }, [id]);

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