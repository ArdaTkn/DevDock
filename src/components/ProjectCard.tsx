import { useNavigate } from "react-router-dom";
import type { Project } from "../types";
import { api } from "../services/api";
import { displayPath, gitState, timeAgo } from "../lib/format";
import { useProjectsStore } from "../stores/projectsStore";

export function ProjectCard({ project }: { project: Project }) {
  const nav = useNavigate();
  const toggleFavorite = useProjectsStore((s) => s.toggleFavorite);
  const gs = gitState(project.git);

  return (
    <div
      className="card"
      role="button"
      tabIndex={0}
      onClick={() => nav(`/project/${project.id}`)}
      onKeyDown={(e) => e.key === "Enter" && nav(`/project/${project.id}`)}
      data-dirty={gs === "dirty" ? "true" : "false"}
    >
      <div className="card-head">
        <span className="card-name">{project.name}</span>
        <span
          className="star"
          role="button"
          aria-label="toggle favorite"
          onClick={(e) => {
            e.stopPropagation();
            void toggleFavorite(project.id, !project.is_favorite);
          }}
        >
          {project.is_favorite ? "★" : "☆"}
        </span>
      </div>

      <div className="card-techs">
        {project.techs.slice(0, 6).map((t) => (
          <span key={t.name} className={`chip chip-${t.kind}`}>
            {t.name}
          </span>
        ))}
        {project.techs.length === 0 && <span className="chip">unknown</span>}
      </div>

      <div className="card-path" title={project.path}>
        {displayPath(project.path)}
      </div>

      <div className="card-meta">
        {project.git?.is_git ? (
          <span className="meta-item">
            <span className={`dot dot-${gs}`} />
            <span className="dot-label">
              {gs === "clean" ? "Clean" : `${project.git?.modified_count ?? 0} mod`}
            </span>
            {project.git.branch && <code>{project.git.branch}</code>}
          </span>
        ) : (
          <span className="meta-item muted">no git</span>
        )}
        <span className="meta-item muted">{timeAgo(project.last_modified)}</span>
      </div>

      <div className="card-actions">
        <button
          className="btn"
          onClick={(e) => {
            e.stopPropagation();
            void api.openEditor(project.path);
          }}
        >
          Editor
        </button>
        <button
          className="btn"
          onClick={(e) => {
            e.stopPropagation();
            void api.openTerminal(project.path);
          }}
        >
          Terminal
        </button>
        <button
          className="btn"
          onClick={(e) => {
            e.stopPropagation();
            void api.openFolder(project.path);
          }}
        >
          Open
        </button>
      </div>
    </div>
  );
}