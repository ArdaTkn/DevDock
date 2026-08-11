import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import type { Project } from "../types";
import { api } from "../services/api";
import { displayPath, gitState, humanSize, timeAgo } from "../lib/format";

export function ProjectDetail() {
  const { id } = useParams();
  const [project, setProject] = useState<Project | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    void api
      .getProject(Number(id))
      .then(setProject)
      .catch((e) => setError(String(e)));
  }, [id]);

  if (error) return <div className="banner error">{error}</div>;
  if (!project) return <div className="empty muted">Loading project…</div>;

  const gs = gitState(project.git);

  return (
    <div className="page detail">
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