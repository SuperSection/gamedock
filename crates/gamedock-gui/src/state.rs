use gamedock_backup::BackupManager;
use gamedock_controller::ControllerManager;
use gamedock_core::{AppConfig, AppInfo, EventBus, RuntimeStatus};
use gamedock_game_library::GameLibrary;
use gamedock_runtime_manager::RuntimeManager;
use std::sync::Arc;

pub struct AppState {
    pub config: AppConfig,
    pub event_bus: EventBus,
    pub runtime_manager: Option<Arc<RuntimeManager>>,
    pub library: Option<Arc<GameLibrary>>,
    pub controller_manager: Option<ControllerManager>,
    pub backup_manager: Option<BackupManager>,
    pub apps: Vec<AppInfo>,
    pub selected_app: Option<usize>,
    pub search_query: String,
    pub runtime_status: Option<RuntimeStatus>,
    pub notification: Option<String>,
    pub show_install_dialog: bool,
    pub show_settings: bool,
    pub active_tab: Tab,
    pub first_run: bool,
    pub installing_waydroid: bool,
    pub setup_gapps: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Library,
    Installed,
    Favorites,
    RecentlyPlayed,
    Controllers,
    Settings,
    Setup,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            event_bus: EventBus::default(),
            runtime_manager: None,
            library: None,
            controller_manager: None,
            backup_manager: None,
            apps: Vec::new(),
            selected_app: None,
            search_query: String::new(),
            runtime_status: None,
            notification: None,
            show_install_dialog: false,
            show_settings: false,
            active_tab: Tab::Setup,
            first_run: true,
            installing_waydroid: false,
            setup_gapps: true,
        }
    }
}

impl AppState {
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        self.config = AppConfig::load()?;
        self.event_bus = EventBus::default();

        let runtime_manager = Arc::new(RuntimeManager::new(
            self.config.clone(),
            self.event_bus.clone(),
        ));
        self.runtime_manager = Some(runtime_manager);

        let library = Arc::new(GameLibrary::new(
            self.config.clone(),
            self.event_bus.clone(),
        ));
        self.library = Some(library);

        self.controller_manager = Some(ControllerManager::new(
            self.config.clone(),
            self.event_bus.clone(),
        ));

        self.backup_manager = Some(BackupManager::new(
            self.config.clone(),
            self.event_bus.clone(),
        ));

        // Check if waydroid is available
        if let Some(ref rm) = self.runtime_manager {
            let rt = tokio::runtime::Handle::current();
            let rm_clone = rm.clone();
            let has_waydroid = tokio::task::block_in_place(|| {
                rt.block_on(async {
                    match rm_clone.get_runtime("waydroid").await {
                        Ok(r) => r.is_available(),
                        Err(_) => false,
                    }
                })
            });

            if !has_waydroid {
                self.first_run = true;
                self.active_tab = Tab::Setup;
            } else {
                self.first_run = false;
                self.active_tab = Tab::Library;
            }
        }

        tracing::info!("GUI AppState initialized (first_run: {})", self.first_run);
        Ok(())
    }

    pub fn load_apps(&mut self) {
        if let Some(ref library) = self.library {
            let rt = tokio::runtime::Handle::current();
            let lib = library.clone();
            let apps =
                tokio::task::block_in_place(|| rt.block_on(async { lib.list_all_apps().await }));
            self.apps = apps;
        }
    }

    pub fn filtered_apps(&self) -> Vec<&AppInfo> {
        self.apps
            .iter()
            .filter(|app| {
                if self.search_query.is_empty() {
                    return true;
                }
                app.matches_search(&self.search_query)
            })
            .collect()
    }

    pub fn set_notification(&mut self, msg: String) {
        self.notification = Some(msg);
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }
}
