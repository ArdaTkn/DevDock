import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import type { Project, ProjectHealth, ProjectScript } from "../types";
import { api } from "../services/api";
import { displayPath, gitState, humanSize, timeAgo } from "../lib/format";

export function ProjectDetail() {
  const { id } = useParams();
  const nav = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [health, setHealth] = useState<ProjectHealth | null>(null);
  const [scripts, setScripts] = useState<ProjectScript[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    void (async () => {
      try {
        const p = await api.getProject(Number(id));
        setProject(p);
        if (p) {
          const isDirty = gitState(p.git) === "dirty";
          const h = await api.getProjectHealth(p.path, isDirty);
          setHealth(h);
          const s = await api.listProjectScripts(p.path);
          setScripts(s);
        }
      } catch (e) {
        setError(String(e));
      }
    })();
  }, [id]);

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
      </div>

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
          <h2>Project Scripts & Tasks ({scripts.length})</h2>
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
        <h2>Git</h2>
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