use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareTelemetryDto {
    pub status: String, // "C" (Clean), "D" (Dirty), "E" (Error), "S" (Syncing)
    pub status_text: String,
    pub total_projects: usize,
    pub dirty_projects_count: usize,
    pub active_ports_count: usize,
    pub recommended_led: String, // "green", "yellow", "red", "blink"
    pub detected_ports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSignalResult {
    pub success: bool,
    pub mode: String, // "physical_serial" or "simulated"
    pub port: String,
    pub signal_sent: String,
    pub message: String,
}

pub struct HardwareStation;

impl HardwareStation {
    /// Detects all connected serial/USB ports across macOS, Linux, and Windows.
    pub fn list_serial_ports() -> Vec<String> {
        let mut ports = Vec::new();

        #[cfg(unix)]
        {
            if let Ok(entries) = fs::read_dir("/dev") {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    // Match common Arduino / Raspberry Pi / USB-to-Serial descriptors
                    if file_name.starts_with("cu.usbmodem")
                        || file_name.starts_with("tty.usbmodem")
                        || file_name.starts_with("cu.usbserial")
                        || file_name.starts_with("tty.usbserial")
                        || file_name.starts_with("ttyACM")
                        || file_name.starts_with("ttyUSB")
                    {
                        ports.push(format!("/dev/{}", file_name));
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            for i in 1..=32 {
                let port_name = format!("COM{}", i);
                if Path::new(&format!("\\\\.\\{}", port_name)).exists() {
                    ports.push(port_name);
                }
            }
        }

        ports.sort();
        ports.dedup();
        ports
    }

    /// Transmits a 1-byte command ('C', 'D', 'E', 'S') to the connected hardware station.
    pub fn send_signal(port_opt: Option<String>, signal: &str) -> HardwareSignalResult {
        let clean_signal = signal.trim().to_uppercase();
        let sig_char = clean_signal.chars().next().unwrap_or('C');

        // Check if a physical serial port was provided and is accessible
        if let Some(ref port_path) = port_opt {
            let path = Path::new(port_path);
            if path.exists() {
                // Try writing single byte to device node
                if let Ok(mut file) = fs::OpenOptions::new().write(true).open(path) {
                    if file.write_all(format!("{}\n", sig_char).as_bytes()).is_ok() {
                        let _ = file.flush();
                        return HardwareSignalResult {
                            success: true,
                            mode: "physical_serial".into(),
                            port: port_path.clone(),
                            signal_sent: sig_char.to_string(),
                            message: format!(
                                "Transmitted signal '{}' directly to Arduino on {}",
                                sig_char, port_path
                            ),
                        };
                    }
                }
            }
        }

        // Fallback: Virtual / Simulated Hardware Station Mode
        HardwareSignalResult {
            success: true,
            mode: "simulated".into(),
            port: port_opt.unwrap_or_else(|| "Virtual Simulator".into()),
            signal_sent: sig_char.to_string(),
            message: format!(
                "Hardware Station Simulated: Signal '{}' broadcasted successfully (Virtual LED: {})",
                sig_char,
                match sig_char {
                    'C' => "🟢 Green (Clean)",
                    'D' => "🟡 Yellow (Dirty)",
                    'E' => "🔴 Red (Error)",
                    'S' => "✨ Blinking Yellow (Syncing)",
                    _ => "⚪ Off",
                }
            ),
        }
    }

    /// Calculates current repository health telemetry to determine hardware state.
    pub fn get_telemetry(db: &crate::storage::AppDb) -> HardwareTelemetryDto {
        let projects =
            crate::storage::project_repo::ProjectRepo::list_projects(db, None).unwrap_or_default();
        let total_projects = projects.len();
        let dirty_count = projects
            .iter()
            .filter(|p| p.git.as_ref().map(|g| !g.clean()).unwrap_or(false))
            .count();

        let ports = crate::processes::ProcScanner::list_listening_ports();
        let detected_ports = Self::list_serial_ports();

        let (status, status_text, recommended_led) = if dirty_count > 0 {
            (
                "D".to_string(),
                format!("{} uncommitted repositories require attention", dirty_count),
                "yellow".to_string(),
            )
        } else if total_projects == 0 {
            (
                "C".to_string(),
                "No repositories scanned yet".to_string(),
                "green".to_string(),
            )
        } else {
            (
                "C".to_string(),
                "All repositories clean and synchronized".to_string(),
                "green".to_string(),
            )
        };

        HardwareTelemetryDto {
            status,
            status_text,
            total_projects,
            dirty_projects_count: dirty_count,
            active_ports_count: ports.len(),
            recommended_led,
            detected_ports,
        }
    }
}
