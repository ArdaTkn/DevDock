use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub group: String,     // "project", "tech", "workspace", "port"
    pub node_type: String, // "project", "tech", "workspace", "port"
    pub size: u32,
    pub color: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub label: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDataDto {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
    pub total_projects: usize,
    pub total_techs: usize,
    pub total_workspaces: usize,
}

pub struct GraphEngine;

impl GraphEngine {
    pub fn build_knowledge_graph(
        projects: &[crate::models::Project],
        workspaces: &[(i64, String, String)], // (id, name, color)
        project_workspaces: &HashMap<i64, Vec<i64>>, // project_id -> workspace_ids
        ports: &[crate::processes::PortInfo],
    ) -> GraphDataDto {
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        let mut tech_set: HashMap<String, (String, usize)> = HashMap::new(); // tech_name -> (color, count)
        let ws_set: std::collections::HashSet<i64> = workspaces.iter().map(|w| w.0).collect();

        // 1. Add Workspace Nodes
        for (w_id, w_name, w_color) in workspaces {
            nodes.push(GraphNode {
                id: format!("ws-{w_id}"),
                label: w_name.clone(),
                group: "workspace".to_string(),
                node_type: "workspace".to_string(),
                size: 24,
                color: w_color.clone(),
                details: format!("Workspace collection: {w_name}"),
            });
        }

        // 2. Add Project Nodes and link to Workspaces & Techs
        for p in projects {
            let p_node_id = format!("proj-{}", p.id);
            let p_color = if p.git.as_ref().map(|g| !g.clean()).unwrap_or(false) {
                "#f59e0b".to_string() // amber for dirty
            } else {
                "#10b981".to_string() // emerald for clean
            };

            nodes.push(GraphNode {
                id: p_node_id.clone(),
                label: p.name.clone(),
                group: "project".to_string(),
                node_type: "project".to_string(),
                size: 18,
                color: p_color,
                details: format!("Path: {}", p.path),
            });

            // Link project to assigned workspaces
            if let Some(ws_ids) = project_workspaces.get(&p.id) {
                for ws_id in ws_ids {
                    if ws_set.contains(ws_id) {
                        links.push(GraphLink {
                            source: p_node_id.clone(),
                            target: format!("ws-{ws_id}"),
                            label: "in workspace".to_string(),
                            weight: 2.0,
                        });
                    }
                }
            }

            // Collect and Link Technologies
            for t in &p.techs {
                let count_entry = tech_set.entry(t.name.clone()).or_insert_with(|| {
                    let color = match t.kind {
                        crate::models::TechKind::Language => "#3b82f6".to_string(), // blue
                        crate::models::TechKind::Framework => "#8b5cf6".to_string(), // purple
                        crate::models::TechKind::Tool => "#06b6d4".to_string(),     // cyan
                        crate::models::TechKind::Runtime => "#10b981".to_string(),
                    };
                    (color, 0)
                });
                count_entry.1 += 1;

                links.push(GraphLink {
                    source: p_node_id.clone(),
                    target: format!("tech-{}", t.name),
                    label: "uses".to_string(),
                    weight: 1.0,
                });
            }
        }

        // 3. Add Tech Nodes
        let total_techs = tech_set.len();
        for (tech_name, (color, count)) in tech_set {
            nodes.push(GraphNode {
                id: format!("tech-{tech_name}"),
                label: tech_name.clone(),
                group: "tech".to_string(),
                node_type: "tech".to_string(),
                size: (12 + (count * 2).min(20)) as u32,
                color,
                details: format!("Used in {count} project(s)"),
            });
        }

        // 4. Add Active Port Nodes
        for port in ports {
            let port_id = format!("port-{}", port.port);
            nodes.push(GraphNode {
                id: port_id,
                label: format!(":{}", port.port),
                group: "port".to_string(),
                node_type: "port".to_string(),
                size: 14,
                color: "#ec4899".to_string(), // pink
                details: format!("Port {} active (PID: {})", port.port, port.pid),
            });
        }

        GraphDataDto {
            total_projects: projects.len(),
            total_techs,
            total_workspaces: workspaces.len(),
            nodes,
            links,
        }
    }
}
