import { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "../services/api";
import type { GraphDataDto, GraphNode } from "../types";

export function KnowledgeGraph() {
  const navigate = useNavigate();
  const [data, setData] = useState<GraphDataDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [filterGroup, setFilterGroup] = useState<string>("all");
  const [search, setSearch] = useState("");
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      setLoading(true);
      try {
        const res = await api.getArchitectureGraph();
        if (!cancelled) setData(res);
      } catch (e) {
        console.error("Failed to load knowledge graph", e);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  // Filter nodes according to search and group filter
  const filteredNodes = useMemo(() => {
    if (!data) return [];
    return data.nodes.filter((n) => {
      const matchGroup = filterGroup === "all" || n.group === filterGroup;
      const matchSearch =
        !search ||
        n.label.toLowerCase().includes(search.toLowerCase()) ||
        n.details.toLowerCase().includes(search.toLowerCase());
      return matchGroup && matchSearch;
    });
  }, [data, filterGroup, search]);

  const activeNodeIds = useMemo(
    () => new Set(filteredNodes.map((n) => n.id)),
    [filteredNodes]
  );

  // Position nodes in a clean radial/clustered layout
  const nodePositions = useMemo(() => {
    const posMap = new Map<string, { x: number; y: number }>();
    if (!data) return posMap;

    const width = 900;
    const height = 580;
    const centerX = width / 2;
    const centerY = height / 2;

    const projects = data.nodes.filter((n) => n.group === "project");
    const techs = data.nodes.filter((n) => n.group === "tech");
    const workspaces = data.nodes.filter((n) => n.group === "workspace");
    const ports = data.nodes.filter((n) => n.group === "port");

    // 1. Projects in middle ellipse
    const projRadiusX = 220;
    const projRadiusY = 160;
    projects.forEach((n, i) => {
      const angle = (i / Math.max(projects.length, 1)) * 2 * Math.PI;
      posMap.set(n.id, {
        x: centerX + projRadiusX * Math.cos(angle),
        y: centerY + projRadiusY * Math.sin(angle),
      });
    });

    // 2. Techs in outer circle
    const techRadius = 270;
    techs.forEach((n, i) => {
      const angle = (i / Math.max(techs.length, 1)) * 2 * Math.PI + 0.3;
      posMap.set(n.id, {
        x: centerX + techRadius * Math.cos(angle),
        y: centerY + techRadius * Math.sin(angle),
      });
    });

    // 3. Workspaces at the top cluster
    workspaces.forEach((n, i) => {
      const spacing = 120;
      const startX = centerX - ((workspaces.length - 1) * spacing) / 2;
      posMap.set(n.id, {
        x: startX + i * spacing,
        y: 60,
      });
    });

    // 4. Ports at the bottom cluster
    ports.forEach((n, i) => {
      const spacing = 90;
      const startX = centerX - ((ports.length - 1) * spacing) / 2;
      posMap.set(n.id, {
        x: startX + i * spacing,
        y: height - 50,
      });
    });

    return posMap;
  }, [data]);

  return (
    <div className="page graph-page">
      <header className="toolbar">
        <div className="toolbar-left">
          <h2>🕸️ Architecture Knowledge Graph</h2>
          <span className="muted small" style={{ marginLeft: "10px" }}>
            Explore cross-project connections, shared tech stacks, and workspace clusters.
          </span>
        </div>
        <div className="toolbar-right">
          <input
            className="search"
            placeholder="Search graph nodes…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{ width: "200px" }}
          />
          <select
            className="select"
            value={filterGroup}
            onChange={(e) => setFilterGroup(e.target.value)}
            aria-label="Filter group"
          >
            <option value="all">All Elements ({data?.nodes.length ?? 0})</option>
            <option value="project">📁 Projects ({data?.total_projects ?? 0})</option>
            <option value="tech">⚙️ Tech Stacks ({data?.total_techs ?? 0})</option>
            <option value="workspace">🗂️ Workspaces ({data?.total_workspaces ?? 0})</option>
            <option value="port">🌸 Active Ports</option>
          </select>
        </div>
      </header>

      {loading ? (
        <div className="panel" style={{ textAlign: "center", padding: "60px" }}>
          <div className="spinner-mini" style={{ width: "32px", height: "32px" }} />
          <p className="muted" style={{ marginTop: "14px" }}>
            Building interactive architecture knowledge graph...
          </p>
        </div>
      ) : !data || data.nodes.length === 0 ? (
        <div className="panel" style={{ textAlign: "center", padding: "60px" }}>
          <h3>No projects found to map</h3>
          <p className="muted">Add scan locations in Settings or run a scan on Dashboard.</p>
        </div>
      ) : (
        <div className="graph-container">
          <div className="graph-canvas-wrapper">
            <svg className="graph-svg" viewBox="0 0 900 580">
              {/* Render Links */}
              <g className="graph-links">
                {data.links.map((link, idx) => {
                  const sourcePos = nodePositions.get(link.source);
                  const targetPos = nodePositions.get(link.target);
                  if (!sourcePos || !targetPos) return null;

                  const isHighlighted =
                    selectedNode &&
                    (link.source === selectedNode.id || link.target === selectedNode.id);
                  const isVisible =
                    activeNodeIds.has(link.source) && activeNodeIds.has(link.target);

                  if (!isVisible && filterGroup !== "all") return null;

                  return (
                    <line
                      key={`${link.source}-${link.target}-${idx}`}
                      x1={sourcePos.x}
                      y1={sourcePos.y}
                      x2={targetPos.x}
                      y2={targetPos.y}
                      stroke={isHighlighted ? "#00f2fe" : "var(--border-soft)"}
                      strokeWidth={isHighlighted ? 2.5 : 1}
                      strokeOpacity={isHighlighted ? 0.9 : 0.4}
                      strokeDasharray={link.label === "in workspace" ? "4,4" : undefined}
                    />
                  );
                })}
              </g>

              {/* Render Nodes */}
              <g className="graph-nodes">
                {filteredNodes.map((node) => {
                  const pos = nodePositions.get(node.id);
                  if (!pos) return null;
                  const isSelected = selectedNode?.id === node.id;

                  return (
                    <g
                      key={node.id}
                      className={`graph-node-group ${isSelected ? "selected" : ""}`}
                      transform={`translate(${pos.x}, ${pos.y})`}
                      onClick={() => setSelectedNode(node)}
                      style={{ cursor: "pointer" }}
                    >
                      <circle
                        r={isSelected ? node.size + 4 : node.size}
                        fill={node.color}
                        stroke={isSelected ? "#ffffff" : "rgba(0,0,0,0.3)"}
                        strokeWidth={isSelected ? 3 : 1.5}
                        className="graph-node-circle"
                      />
                      <text
                        dy={node.size + 14}
                        textAnchor="middle"
                        fill="var(--text)"
                        fontSize="11px"
                        fontWeight={isSelected ? "bold" : "normal"}
                        className="graph-node-label"
                      >
                        {node.label}
                      </text>
                    </g>
                  );
                })}
              </g>
            </svg>
          </div>

          {/* Side Inspector Panel */}
          <div className="graph-inspector">
            {selectedNode ? (
              <div className="graph-node-details">
                <div className="graph-details-header">
                  <span
                    className="graph-node-badge"
                    style={{ backgroundColor: selectedNode.color }}
                  >
                    {selectedNode.group.toUpperCase()}
                  </span>
                  <h3>{selectedNode.label}</h3>
                </div>
                <p className="muted" style={{ fontSize: "12.5px", margin: "8px 0 14px 0" }}>
                  {selectedNode.details}
                </p>

                {selectedNode.group === "project" && (
                  <button
                    className="btn primary"
                    style={{ width: "100%", marginTop: "10px" }}
                    onClick={() => {
                      const projId = selectedNode.id.replace("proj-", "");
                      navigate(`/project/${projId}`);
                    }}
                  >
                    🚀 Inspect Project Details
                  </button>
                )}

                {/* Connected Links Summary */}
                <div style={{ marginTop: "18px" }}>
                  <span className="muted small" style={{ fontWeight: 600, display: "block", marginBottom: "6px" }}>
                    Connected Graph Links:
                  </span>
                  <div className="graph-connections-list">
                    {data.links
                      .filter(
                        (l) => l.source === selectedNode.id || l.target === selectedNode.id
                      )
                      .map((l, i) => {
                        const otherId = l.source === selectedNode.id ? l.target : l.source;
                        const otherNode = data.nodes.find((n) => n.id === otherId);
                        if (!otherNode) return null;
                        return (
                          <div
                            key={i}
                            className="graph-conn-chip"
                            onClick={() => setSelectedNode(otherNode)}
                          >
                            <span>{l.label}:</span> <b>{otherNode.label}</b>
                          </div>
                        );
                      })}
                  </div>
                </div>
              </div>
            ) : (
              <div className="graph-inspector-empty">
                <span style={{ fontSize: "28px" }}>🕸️</span>
                <h4>Interactive Knowledge Graph</h4>
                <p className="muted small">
                  Click any project, technology, or workspace node to inspect connections and navigate directly.
                </p>
                <div className="graph-legend">
                  <div className="legend-item"><span className="legend-dot" style={{ background: "#10b981" }} /> Projects</div>
                  <div className="legend-item"><span className="legend-dot" style={{ background: "#3b82f6" }} /> Languages</div>
                  <div className="legend-item"><span className="legend-dot" style={{ background: "#8b5cf6" }} /> Frameworks</div>
                  <div className="legend-item"><span className="legend-dot" style={{ background: "#ec4899" }} /> Ports</div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
