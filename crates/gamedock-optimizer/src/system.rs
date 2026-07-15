use gamedock_core::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub gpu_name: Option<String>,
    pub gpu_driver: Option<String>,
    pub display_server: String,
    pub compositor: Option<String>,
    pub kernel_version: String,
    pub distro: Option<String>,
    pub has_gamemode: bool,
    pub has_mangohud: bool,
    pub has_waydroid: bool,
}

impl SystemInfo {
    pub fn collect() -> Result<Self> {
        let cpu_model = Self::read_cpu_model();
        let cpu_cores = Self::read_cpu_cores();
        let (total_memory_mb, available_memory_mb) = Self::read_memory();
        let kernel_version = Self::read_kernel_version();

        let display_server = if std::env::var("WAYLAND_DISPLAY").is_ok() {
            "Wayland".to_string()
        } else if std::env::var("DISPLAY").is_ok() {
            "X11".to_string()
        } else {
            "Unknown".to_string()
        };

        let compositor = std::env::var("XDG_CURRENT_DESKTOP").ok();

        let (gpu_name, gpu_driver) = Self::detect_gpu();

        Ok(Self {
            cpu_model,
            cpu_cores,
            total_memory_mb,
            available_memory_mb,
            gpu_name,
            gpu_driver,
            display_server,
            compositor,
            kernel_version,
            distro: Self::read_distro(),
            has_gamemode: crate::gamemode::GameModeIntegration::is_available(),
            has_mangohud: crate::mangohud::MangoHUDIntegration::is_available(),
            has_waydroid: which::which("waydroid").is_ok(),
        })
    }

    fn detect_gpu() -> (Option<String>, Option<String>) {
        let gpu_name = Self::detect_gpu_name();
        let gpu_driver = Self::detect_gpu_driver();
        (gpu_name, gpu_driver)
    }

    fn detect_gpu_name() -> Option<String> {
        if let Ok(output) = Command::new("lspci").arg("-vnn").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("vga") || lower.contains("3d") || lower.contains("display") {
                    if let Some(name) = line.split(':').nth(2) {
                        return Some(name.trim().to_string());
                    }
                }
            }
        }

        if let Ok(output) = Command::new("glxinfo").arg("-B").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Device:") {
                    return Some(line.split(':').nth(1)?.trim().to_string());
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string("/sys/class/drm/card0/device/vendor") {
            let vendor_id = content.trim();
            let name = match vendor_id {
                "0x10de" => "NVIDIA GPU",
                "0x1002" => "AMD GPU",
                "0x8086" => "Intel GPU",
                _ => "Unknown GPU",
            };
            return Some(name.to_string());
        }

        None
    }

    fn detect_gpu_driver() -> Option<String> {
        if let Ok(output) = Command::new("lspci").arg("-k").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut in_vga = false;
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("vga") || lower.contains("3d") || lower.contains("display") {
                    in_vga = true;
                    continue;
                }
                if in_vga && lower.contains("kernel driver in use") {
                    if let Some(driver) = line.split(':').nth(1) {
                        return Some(driver.trim().to_string());
                    }
                }
                if in_vga && (lower.contains("lspci") || line.contains("Region")) {
                    in_vga = false;
                }
            }
        }

        if let Ok(content) =
            std::fs::read_to_string("/sys/class/drm/card0/device/driver/module/version")
                .or_else(|_| std::fs::read_to_string("/sys/module/nvidia/version"))
        {
            let version = content.trim();
            if !version.is_empty() {
                return Some(format!("nvidia {}", version));
            }
        }

        if let Ok(content) = std::fs::read_to_string("/sys/kernel/debug/dri/0/name") {
            let name = content.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }

        None
    }

    fn read_cpu_model() -> String {
        std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("model name"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| "Unknown CPU".to_string())
    }

    fn read_cpu_cores() -> u32 {
        std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1)
    }

    fn read_memory() -> (u64, u64) {
        let total = std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("MemTotal"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|kb| kb / 1024)
            })
            .unwrap_or(0);

        let available = std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("MemAvailable"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|kb| kb / 1024)
            })
            .unwrap_or(0);

        (total, available)
    }

    fn read_kernel_version() -> String {
        std::fs::read_to_string("/proc/version")
            .ok()
            .map(|s| s.lines().next().unwrap_or("Unknown").to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn read_distro() -> Option<String> {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("PRETTY_NAME"))
                    .and_then(|line| line.split('=').nth(1))
                    .map(|s| s.trim_matches('"').to_string())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_collect() {
        let info = SystemInfo::collect().unwrap();
        assert!(!info.cpu_model.is_empty());
        assert!(info.cpu_cores > 0);
        assert!(info.total_memory_mb > 0);
        assert!(!info.kernel_version.is_empty());
    }

    #[test]
    fn test_cpu_cores() {
        let cores = SystemInfo::read_cpu_cores();
        assert!(cores >= 1);
    }

    #[test]
    fn test_memory() {
        let (total, available) = SystemInfo::read_memory();
        assert!(total > 0);
        assert!(available <= total);
    }

    #[test]
    fn test_kernel_version() {
        let version = SystemInfo::read_kernel_version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_display_server_detection() {
        let info = SystemInfo::collect().unwrap();
        assert!(
            info.display_server == "Wayland"
                || info.display_server == "X11"
                || info.display_server == "Unknown"
        );
    }

    #[test]
    fn test_gpu_detection() {
        let (name, driver) = SystemInfo::detect_gpu();
        if name.is_some() {
            assert!(!name.as_ref().unwrap().is_empty());
        }
        if driver.is_some() {
            assert!(!driver.as_ref().unwrap().is_empty());
        }
    }
}
