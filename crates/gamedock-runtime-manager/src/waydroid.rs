use async_trait::async_trait;
use gamedock_core::{AppInfo, RuntimeInfo, RuntimeStatus, Result, Error};
use gamedock_plugin_sdk::{RuntimePlugin, PluginMetadata};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::RwLock;
use std::sync::Arc;

const WAYDROID_IDLE_TIMEOUT_SECS: u64 = 300;

pub struct WaydroidRuntime {
    waydroid_bin: Option<PathBuf>,
    session_dir: PathBuf,
    last_activity: Arc<RwLock<Instant>>,
    idle_watcher_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl WaydroidRuntime {
    pub fn new() -> Result<Self> {
        let waydroid_bin = which::which("waydroid").ok();
        let session_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("waydroid");
        Ok(Self {
            waydroid_bin,
            session_dir,
            last_activity: Arc::new(RwLock::new(Instant::now())),
            idle_watcher_handle: Arc::new(RwLock::new(None)),
        })
    }

    pub fn is_available(&self) -> bool {
        self.waydroid_bin.is_some() || which::which("waydroid").is_ok()
    }

    pub async fn install_waydroid_system() -> Result<()> {
        tracing::info!("Installing Waydroid system package...");

        let family = Self::detect_distro_family().await;
        tracing::info!("Detected distro family: {:?}", family);

        // Try the detected package manager family first
        match family.as_deref() {
            Some("debian") | Some("ubuntu") | Some("linuxmint") | Some("fedora") => {
                // handled below by package manager detection
            }
            _ => {}
        }

        // Detect by available package manager (most reliable)
        if which::which("apt").is_ok() || which::which("apt-get").is_ok() {
            // Debian/Ubuntu/Mint/Kali/Pop/Zorin/Elementary etc.
            let status = Command::new("sudo")
                .args(["apt", "install", "-y", "waydroid"])
                .status()
                .await
                .map_err(|e| Error::Runtime(format!("Failed to run apt: {}", e)))?;

            if status.success() {
                tracing::info!("Waydroid installed via apt");
                return Ok(());
            }
        }

        if which::which("dnf").is_ok() {
            // Fedora/RHEL/Rocky/Alma
            let status = Command::new("sudo")
                .args(["dnf", "install", "-y", "waydroid"])
                .status()
                .await
                .map_err(|e| Error::Runtime(format!("Failed to run dnf: {}", e)))?;

            if status.success() {
                tracing::info!("Waydroid installed via dnf");
                return Ok(());
            }
        }

        if which::which("yum").is_ok() {
            // Older RHEL/CentOS
            let status = Command::new("sudo")
                .args(["yum", "install", "-y", "waydroid"])
                .status()
                .await
                .map_err(|e| Error::Runtime(format!("Failed to run yum: {}", e)))?;

            if status.success() {
                tracing::info!("Waydroid installed via yum");
                return Ok(());
            }
        }

        if which::which("pacman").is_ok() {
            // Arch/Manjaro/EndeavourOS/Garuda/CachyOS
            let status = Command::new("sudo")
                .args(["pacman", "-S", "--noconfirm", "waydroid"])
                .status()
                .await
                .map_err(|e| Error::Runtime(format!("Failed to run pacman: {}", e)))?;

            if status.success() {
                tracing::info!("Waydroid installed via pacman");
                return Ok(());
            }
        }

        if which::which("zypper").is_ok() {
            // OpenSUSE
            let status = Command::new("sudo")
                .args(["zypper", "--non-interactive", "install", "waydroid"])
                .status()
                .await
                .map_err(|e| Error::Runtime(format!("Failed to run zypper: {}", e)))?;

            if status.success() {
                tracing::info!("Waydroid installed via zypper");
                return Ok(());
            }
        }

        if which::which("apk").is_ok() {
            // Alpine
            let status = Command::new("sudo")
                .args(["apk", "add", "waydroid"])
                .status()
                .await
                .map_err(|e| Error::Runtime(format!("Failed to run apk: {}", e)))?;

            if status.success() {
                tracing::info!("Waydroid installed via apk");
                return Ok(());
            }
        }

        // Nothing worked — give manual instructions with the actual URL
        Err(Error::Runtime(
            "Could not auto-install Waydroid on your distro.\n\n\
             Please install it manually — visit https://docs.waydro.id and follow\n\
             the instructions for your distro. Once installed, run:\n\n\
             gamedock init --gapps".into()
        ))
    }

    async fn detect_distro_family() -> Option<String> {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let mut id = None;
            let mut id_like = None;

            for line in content.lines() {
                if let Some(value) = line.strip_prefix("ID=") {
                    id = Some(value.trim_matches('"').to_lowercase());
                }
                if let Some(value) = line.strip_prefix("ID_LIKE=") {
                    id_like = Some(value.trim_matches('"').to_lowercase());
                }
            }

            // ID_LIKE tells us the family (e.g. "ubuntu" for mint, "debian fedora" for rocky)
            if let Some(ref like) = id_like {
                if like.contains("debian") || like.contains("ubuntu") {
                    return Some("debian".to_string());
                }
                if like.contains("fedora") || like.contains("rhel") {
                    return Some("fedora".to_string());
                }
                if like.contains("arch") {
                    return Some("arch".to_string());
                }
            }

            // Fall back to ID itself
            if let Some(ref distro_id) = id {
                match distro_id.as_str() {
                    "ubuntu" | "debian" | "linuxmint" | "pop" | "zorin" | "elementary"
                    | "kali" | "parrot" | "raspbian" | "deepin" | "kaisen" | "vanilla"
                    | "devuan" | "antix" | " mx" => return Some("debian".to_string()),

                    "fedora" | "nobara" | "bazzite" | "silverblue" | "kinoite" | "sericea" => {
                        return Some("fedora".to_string());
                    }

                    "arch" | "manjaro" | "endeavouros" | "garuda" | "cachyos" | "artix"
                    | "blackarch" | "archcraft" | "bluestar" | "ystalos" => {
                        return Some("arch".to_string());
                    }

                    "opensuse-leap" | "opensuse-tumbleweed" | "opensuse" | "sles" => {
                        return Some("opensuse".to_string());
                    }

                    "centos" | "rhel" | "rocky" | "almalinux" | "ol" | "eurolinux" => {
                        return Some("fedora".to_string());
                    }

                    "alpine" => return Some("alpine".to_string()),

                    _ => {}
                }
            }
        }

        None
    }

    async fn touch_activity(&self) {
        *self.last_activity.write().await = Instant::now();
    }

    pub async fn start_idle_watcher(self: &Arc<Self>) {
        let mut handle = self.idle_watcher_handle.write().await;
        if handle.is_some() {
            return;
        }

        let runtime = Arc::clone(self);
        let h = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let last = *runtime.last_activity.read().await;
                if last.elapsed() > Duration::from_secs(WAYDROID_IDLE_TIMEOUT_SECS) {
                    tracing::info!(
                        "Waydroid idle for {}s, stopping session to free resources",
                        WAYDROID_IDLE_TIMEOUT_SECS
                    );
                    let _ = runtime.stop().await;
                    break;
                }
            }
        });
        *handle = Some(h);
    }

    pub async fn stop_idle_watcher(&self) {
        let mut handle = self.idle_watcher_handle.write().await;
        if let Some(h) = handle.take() {
            h.abort();
        }
    }

    async fn exec_command(&self, args: &[&str]) -> Result<String> {
        // Re-detect binary if missing (e.g. just installed via pacman)
        let bin_path = if self.waydroid_bin.is_some() {
            self.waydroid_bin.clone().unwrap()
        } else {
            match which::which("waydroid") {
                Ok(p) => p,
                Err(_) => return Err(Error::Runtime(
                    "Waydroid is not installed. Run 'gamedock init' to install it.".into()
                )),
            }
        };

        let output = Command::new(&bin_path)
            .args(args)
            .output()
            .await
            .map_err(|e| Error::Runtime(format!("Failed to execute waydroid: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Runtime(format!("waydroid command failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn run_command_sudo(&self, args: &[&str]) -> Result<String> {
        let mut sudo_args = vec!["waydroid"];
        sudo_args.extend_from_slice(args);

        let output = Command::new("sudo")
            .args(&sudo_args)
            .output()
            .await
            .map_err(|e| Error::Runtime(format!("Failed to execute sudo waydroid: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Runtime(format!("sudo waydroid command failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn is_container_running(&self) -> Result<bool> {
        // Check if waydroid session process is running
        let output = Command::new("pgrep")
            .args(["-f", "waydroid.*session"])
            .output()
            .await;

        match output {
            Ok(o) => Ok(o.status.success()),
            Err(_) => {
                // Fallback: check if session socket exists
                let session_dir = dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("waydroid");
                Ok(session_dir.join("waydroid.cfg").exists())
            }
        }
    }

    async fn ensure_session_dir(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.session_dir).await?;
        Ok(())
    }

    async fn is_android_booted(&self) -> Result<bool> {
        // Check if the Android display is ready by looking for the surfaceflinger process
        let output = Command::new("pgrep")
            .args(["-f", "surfaceflinger"])
            .output()
            .await;

        match output {
            Ok(o) => Ok(o.status.success()),
            Err(_) => Ok(false),
        }
    }
}

#[async_trait]
impl RuntimePlugin for WaydroidRuntime {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "waydroid".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: "GameDock".to_string(),
            description: "Waydroid-based Android runtime backend".to_string(),
            plugin_type: gamedock_plugin_sdk::PluginType::Runtime,
        }
    }

    async fn check_status(&self) -> Result<RuntimeStatus> {
        if !self.is_available() {
            return Ok(RuntimeStatus::NotInstalled);
        }

        match self.is_container_running().await {
            Ok(true) => Ok(RuntimeStatus::Running),
            Ok(false) => Ok(RuntimeStatus::Installed),
            Err(_) => Ok(RuntimeStatus::Installed),
        }
    }

    async fn install(&self) -> Result<()> {
        tracing::info!("Installing Waydroid...");

        if !self.is_available() {
            tracing::info!("Waydroid binary not found, installing system package...");
            Self::install_waydroid_system().await?;

            // Re-detect the binary after install
            if let Ok(new_bin) = which::which("waydroid") {
                tracing::info!("Waydroid installed at: {:?}", new_bin);
            } else {
                return Err(Error::Runtime(
                    "Waydroid installed but binary not found in PATH. \
                     You may need to log out and back in.".into()
                ));
            }
        } else {
            tracing::info!("Waydroid already installed, skipping");
        }

        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        tracing::info!("Uninstalling Waydroid...");
        let _ = self.stop().await;
        if self.is_available() {
            self.exec_command(&["init", "-f"]).await?;
        }
        Ok(())
    }

    async fn start(&self) -> Result<()> {
        tracing::info!("Starting Waydroid session...");
        self.ensure_session_dir().await?;

        if self.is_container_running().await? {
            tracing::info!("Waydroid session already running");
            self.touch_activity().await;
            return Ok(());
        }

        self.exec_command(&["session", "start"]).await?;
        self.touch_activity().await;

        // Wait for Android to actually boot (session start returns before Android is ready)
        tracing::info!("Waiting for Android to boot...");
        for i in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if let Ok(true) = self.is_android_booted().await {
                tracing::info!("Android booted successfully");
                return Ok(());
            }
            if i % 5 == 0 && i > 0 {
                tracing::info!("Still waiting for Android boot... ({}s)", i);
            }
        }

        tracing::warn!("Android boot timed out, continuing anyway");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping Waydroid session...");
        self.stop_idle_watcher().await;
        let _ = self.exec_command(&["session", "stop"]).await;
        tracing::info!("Waydroid session stopped");
        Ok(())
    }

    async fn update(&self) -> Result<()> {
        tracing::info!("Updating Waydroid...");
        self.exec_command(&["update"]).await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        match self.check_status().await? {
            RuntimeStatus::Running => Ok(true),
            RuntimeStatus::Installed => {
                self.start().await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    async fn install_app(&self, package: &Path) -> Result<String> {
        tracing::info!("Installing app: {:?}", package);
        self.touch_activity().await;
        let output = self.exec_command(&[
            "app", "install",
            &package.to_string_lossy(),
        ]).await?;
        tracing::info!("App installation output: {}", output);
        Ok(output)
    }

    async fn uninstall_app(&self, package_name: &str) -> Result<()> {
        tracing::info!("Uninstalling app: {}", package_name);
        self.exec_command(&["app", "uninstall", package_name]).await?;
        Ok(())
    }

    async fn launch_app(&self, package_name: &str) -> Result<()> {
        tracing::info!("Launching app: {}", package_name);

        if !self.is_container_running().await? {
            self.start().await?;
        }

        self.touch_activity().await;
        self.exec_command(&["app", "launch", package_name]).await?;
        Ok(())
    }

    async fn launch_play_store(&self) -> Result<()> {
        self.launch_app("com.android.vending").await
    }

    async fn list_installed_apps(&self) -> Result<Vec<AppInfo>> {
        let output = self.exec_command(&["app", "list"]).await?;
        let mut apps = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with("Package") && !line.starts_with('-') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pkg_name) = parts.first() {
                    let mut app = AppInfo::new(
                        *pkg_name,
                        *pkg_name,
                        "unknown".to_string(),
                        0,
                    );
                    app.status = gamedock_core::AppStatus::Installed;
                    apps.push(app);
                }
            }
        }

        Ok(apps)
    }

    async fn get_runtime_info(&self) -> Result<RuntimeInfo> {
        let status = self.check_status().await?;
        let mut info = RuntimeInfo::new("waydroid", "Waydroid");
        info.status = status;

        if self.is_available() {
            if let Ok(output) = self.exec_command(&["--version"]).await {
                info.version = Some(output.trim().to_string());
            }
        }

        Ok(info)
    }

    async fn push_file(&self, local: &Path, remote: &str) -> Result<()> {
        self.exec_command(&[
            "push",
            &local.to_string_lossy(),
            remote,
        ]).await?;
        Ok(())
    }

    async fn pull_file(&self, remote: &str, local: &Path) -> Result<()> {
        self.exec_command(&[
            "pull",
            remote,
            &local.to_string_lossy(),
        ]).await?;
        Ok(())
    }

    async fn run_command(&self, command: &[&str]) -> Result<String> {
        self.exec_command(command).await
    }
}
