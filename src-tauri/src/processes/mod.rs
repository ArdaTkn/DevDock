use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct PortInfo {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub label: String,
}

pub struct ProcScanner;

impl ProcScanner {
    /// Returns active listening TCP ports on the system.
    pub fn list_listening_ports() -> Vec<PortInfo> {
        #[cfg(target_os = "macos")]
        {
            Self::list_ports_macos()
        }
        #[cfg(target_os = "linux")]
        {
            Self::list_ports_linux()
        }
        #[cfg(target_os = "windows")]
        {
            Self::list_ports_windows()
        }
    }

    #[cfg(target_os = "macos")]
    fn list_ports_macos() -> Vec<PortInfo> {
        let output = match Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"])
            .output()
        {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return Vec::new(),
        };

        let mut ports = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }

            let command = parts[0].to_string();
            let pid: u32 = match parts[1].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let name_field = parts[8];
            // Format is like *:3000 or 127.0.0.1:5173
            if let Some(port_str) = name_field.split(':').next_back() {
                if let Ok(port) = port_str.parse::<u16>() {
                    if Self::is_dev_process(&command, port) && seen.insert(port) {
                        let label = Self::derive_label(&command, port);
                        ports.push(PortInfo {
                            port,
                            pid,
                            process_name: command,
                            label,
                        });
                    }
                }
            }
        }

        ports.sort_by_key(|p| p.port);
        ports
    }

    #[cfg(target_os = "linux")]
    fn list_ports_linux() -> Vec<PortInfo> {
        let output = match Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"])
            .output()
        {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return Vec::new(),
        };

        let mut ports = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }

            let command = parts[0].to_string();
            let pid: u32 = match parts[1].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let name_field = parts[8];
            if let Some(port_str) = name_field.split(':').next_back() {
                if let Ok(port) = port_str.parse::<u16>() {
                    if Self::is_dev_process(&command, port) && seen.insert(port) {
                        let label = Self::derive_label(&command, port);
                        ports.push(PortInfo {
                            port,
                            pid,
                            process_name: command,
                            label,
                        });
                    }
                }
            }
        }

        ports.sort_by_key(|p| p.port);
        ports
    }

    #[cfg(target_os = "windows")]
    fn list_ports_windows() -> Vec<PortInfo> {
        let output = match Command::new("netstat").args(["-ano"]).output() {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return Vec::new(),
        };

        let mut ports = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for line in output.lines() {
            if !line.contains("LISTENING") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            let addr = parts[1];
            let pid: u32 = match parts[4].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if let Some(port_str) = addr.split(':').next_back() {
                if let Ok(port) = port_str.parse::<u16>() {
                    let proc_name = format!("pid {pid}");
                    if Self::is_dev_process(&proc_name, port) && seen.insert(port) {
                        let label = Self::derive_label(&proc_name, port);
                        ports.push(PortInfo {
                            port,
                            pid,
                            process_name: proc_name,
                            label,
                        });
                    }
                }
            }
        }

        ports.sort_by_key(|p| p.port);
        ports
    }

    /// Filters out non-development desktop apps (Spotify, Discord, system daemons)
    /// and keeps only development web servers, databases, and runtimes.
    fn is_dev_process(proc_name: &str, port: u16) -> bool {
        let lower = proc_name.to_lowercase();

        // Blacklist non-dev desktop apps and system daemons
        let blacklist = [
            "spotify",
            "discord",
            "steam",
            "dropbox",
            "slack",
            "telegram",
            "chrome",
            "safari",
            "firefox",
            "launchd",
            "cupsd",
            "mdnsresponder",
            "cloudd",
            "netbios",
            "avahi",
            "systemd",
            "loginwindow",
            "identityservicesd",
            "rapportd",
            "sharingd",
            "trustd",
            "airplay",
            "finder",
            "dock",
        ];

        for item in blacklist {
            if lower.contains(item) {
                return false;
            }
        }

        // Whitelist known dev ports
        match port {
            1313
            | 1420
            | 3000..=3010
            | 3306
            | 4000..=4010
            | 4200..=4210
            | 5000..=5010
            | 5173..=5185
            | 5432
            | 6379
            | 8000..=8090
            | 8443
            | 8500
            | 8888
            | 9000..=9010
            | 9092
            | 9200
            | 11434
            | 27017 => return true,
            _ => {}
        }

        // Whitelist known dev runtimes & servers
        let dev_runtimes = [
            "node", "python", "ruby", "java", "go", "cargo", "rust", "deno", "bun", "vite", "next",
            "docker", "postgres", "mysql", "redis", "mongo", "php", "caddy", "nginx", "puma",
            "uvicorn", "gunicorn", "webpack", "ollama",
        ];

        for runtime in dev_runtimes {
            if lower.contains(runtime) {
                return true;
            }
        }

        false
    }

    fn derive_label(proc_name: &str, port: u16) -> String {
        let lower = proc_name.to_lowercase();
        match port {
            3000..=3002 => "React / Next.js / Node".into(),
            5173 | 5174 => "Vite Dev Server".into(),
            8000 | 8080 | 8001 | 5000 => "Python / Web Server".into(),
            5432 => "PostgreSQL".into(),
            3306 => "MySQL".into(),
            6379 => "Redis".into(),
            27017 => "MongoDB".into(),
            11434 => "Ollama (AI)".into(),
            1420 => "Tauri Dev".into(),
            _ if lower.contains("node") => "Node.js Server".into(),
            _ if lower.contains("python") => "Python Server".into(),
            _ if lower.contains("cargo") || lower.contains("rust") => "Rust Server".into(),
            _ if lower.contains("go") => "Go Server".into(),
            _ => format!("{proc_name}:{port}"),
        }
    }
}
