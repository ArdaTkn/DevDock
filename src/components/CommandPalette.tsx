import { useEffect, useState, useMemo, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useProjectsStore } from "../stores/projectsStore";
import { useScanStore } from "../stores/scanStore";
import { api } from "../services/api";
import type { Project } from "../types";

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
}

type ActionItem =
  | { type: "project"; project: Project; action: "editor" | "terminal" | "folder" | "detail" }
  | { type: "global"; id: "rescan" | "settings" | "dashboard"; label: string; icon: string };

export function CommandPalette({ isOpen, onClose }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  const { projects } = useProjectsStore();
  const scanStore = useScanStore();

  // Focus input when palette opens
  useEffect(() => {
    if (isOpen) {
      setQuery("");
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 30);
    }
  }, [isOpen]);

  const items = useMemo<ActionItem[]>(() => {
    const q = query.trim().toLowerCase();
    const result: ActionItem[] = [];

    // Filter projects matching search
    const matchedProjects = q
      ? projects.filter(
          (p) =>
            p.name.toLowerCase().includes(q) ||
            p.path.toLowerCase().includes(q) ||
            p.techs.some((t) => t.name.toLowerCase().includes(q)),
        )
      : projects.slice(0, 8); // Top 8 when no query

    for (const p of matchedProjects) {
      result.push({ type: "project", project: p, action: "editor" });
    }

    // Global actions
    if (!q || "rescan".includes(q) || "scan".includes(q)) {
      result.push({
        type: "global",
        id: "rescan",
        label: "Rescan All Projects",
        icon: "⚡",
      });
    }
    if (!q || "settings".includes(q) || "preferences".includes(q)) {
      result.push({
        type: "global",
        id: "settings",
        label: "Open Settings",
        icon: "⚙️",
      });
    }
    if (!q || "dashboard".includes(q) || "home".includes(q) || "projects".includes(q)) {
      result.push({
        type: "global",
        id: "dashboard",
        label: "Go to Dashboard",
        icon: "🏠",
      });
    }

    return result;
  }, [query, projects]);

  // Keep selection in bounds
  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleExecute = async (item: ActionItem) => {
    onClose();
    if (item.type === "global") {
      if (item.id === "rescan") {
        const summary = await scanStore.start();
        if (summary) await useProjectsStore.getState().refresh();
      } else if (item.id === "settings") {
        navigate("/settings");
      } else if (item.id === "dashboard") {
        navigate("/");
      }
    } else if (item.type === "project") {
      const p = item.project;
      if (item.action === "editor") {
        await api.openEditor(p.path);
      } else if (item.action === "terminal") {
        await api.openTerminal(p.path);
      } else if (item.action === "folder") {
        await api.openFolder(p.path);
      } else if (item.action === "detail") {
        navigate(`/project/${p.id}`);
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => (items.length > 0 ? (prev + 1) % items.length : 0));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => (items.length > 0 ? (prev - 1 + items.length) % items.length : 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (items[selectedIndex]) {
        void handleExecute(items[selectedIndex]);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="cmd-backdrop" onClick={onClose}>
      <div
        className="cmd-modal"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <div className="cmd-search-bar">
          <span className="cmd-icon">🔍</span>
          <input
            ref={inputRef}
            className="cmd-input"
            placeholder="Type a project or command… (Esc to close)"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <kbd className="cmd-kbd">ESC</kbd>
        </div>

        <div className="cmd-results">
          {items.length === 0 ? (
            <div className="cmd-empty">No matching projects or commands found.</div>
          ) : (
            items.map((item, idx) => {
              const isSelected = idx === selectedIndex;
              if (item.type === "global") {
                return (
                  <div
                    key={`global-${item.id}`}
                    className={`cmd-item ${isSelected ? "selected" : ""}`}
                    onClick={() => void handleExecute(item)}
                    onMouseEnter={() => setSelectedIndex(idx)}
                  >
                    <span className="cmd-item-icon">{item.icon}</span>
                    <span className="cmd-item-label">{item.label}</span>
                    <span className="cmd-item-hint">Action</span>
                  </div>
                );
              }

              const p = item.project;
              return (
                <div
                  key={`project-${p.id}`}
                  className={`cmd-item ${isSelected ? "selected" : ""}`}
                  onClick={() => void handleExecute(item)}
                  onMouseEnter={() => setSelectedIndex(idx)}
                >
                  <span className="cmd-item-icon">📁</span>
                  <div className="cmd-item-content">
                    <div className="cmd-item-title">
                      <span className="cmd-item-name">{p.name}</span>
                      <div className="cmd-item-techs">
                        {p.techs.slice(0, 3).map((t) => (
                          <span key={t.name} className={`chip chip-${t.kind}`}>
                            {t.name}
                          </span>
                        ))}
                      </div>
                    </div>
                    <span className="cmd-item-path">{p.path}</span>
                  </div>
                  <div className="cmd-item-actions">
                    <button
                      className="cmd-act-btn"
                      title="Open in Editor"
                      onClick={(e) => {
                        e.stopPropagation();
                        void handleExecute({ type: "project", project: p, action: "editor" });
                      }}
                    >
                      Editor
                    </button>
                    <button
                      className="cmd-act-btn"
                      title="Open in Terminal"
                      onClick={(e) => {
                        e.stopPropagation();
                        void handleExecute({ type: "project", project: p, action: "terminal" });
                      }}
                    >
                      Terminal
                    </button>
                    <button
                      className="cmd-act-btn"
                      title="Open Detail"
                      onClick={(e) => {
                        e.stopPropagation();
                        void handleExecute({ type: "project", project: p, action: "detail" });
                      }}
                    >
                      Detail
                    </button>
                  </div>
                </div>
              );
            })
          )}
        </div>
        <div className="cmd-footer">
          <span><kbd>↑</kbd> <kbd>↓</kbd> navigate</span>
          <span><kbd>↵</kbd> open editor</span>
          <span><kbd>ESC</kbd> close</span>
        </div>
      </div>
    </div>
  );
}
