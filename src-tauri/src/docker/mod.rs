use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct DockerContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: String,
    pub is_running: bool,
}

pub struct DockerScanner;

impl DockerScanner {
    /// Returns a list of active and recent Docker containers on the system.
    pub fn list_containers() -> Vec<DockerContainerInfo> {
        let output = match Command::new("docker")
            .args([
                "ps",
                "-a",
                "--format",
                "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}",
            ])
            .output()
        {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return Vec::new(),
        };

        let mut containers = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 5 {
                continue;
            }

            let id = parts[0].trim().to_string();
            let name = parts[1].trim().to_string();
            let image = parts[2].trim().to_string();
            let status = parts[3].trim().to_string();
            let ports = parts[4].trim().to_string();
            let is_running = status.to_lowercase().contains("up");

            containers.push(DockerContainerInfo {
                id,
                name,
                image,
                status,
                ports,
                is_running,
            });
        }

        containers
    }
}
