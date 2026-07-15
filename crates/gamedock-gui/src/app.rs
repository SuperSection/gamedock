use crate::state::{AppState, Tab};
use egui::{self, Align, Color32, Layout, RichText};

pub struct GameDockApp {
    state: AppState,
}

impl GameDockApp {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl eframe::App for GameDockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_top_panel(ctx);
        self.render_sidebar(ctx);
        self.render_main_content(ctx);
    }
}

impl GameDockApp {
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("GameDock").size(24.0).strong());
                ui.separator();

                if ui.button("Refresh").clicked() {
                    self.state.load_apps();
                }

                if ui.button("Install").clicked() {
                    self.state.show_install_dialog = true;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(ref status) = self.state.runtime_status {
                        let (color, text) = match status {
                            gamedock_core::RuntimeStatus::Running => (Color32::GREEN, "Running"),
                            gamedock_core::RuntimeStatus::Installed => (Color32::YELLOW, "Ready"),
                            gamedock_core::RuntimeStatus::NotInstalled => {
                                (Color32::RED, "Not Installed")
                            }
                            _ => (Color32::GRAY, "Unknown"),
                        };
                        ui.label(RichText::new(text).color(color));
                    }

                    ui.separator();

                    let search = &mut self.state.search_query;
                    ui.add(
                        egui::TextEdit::singleline(search)
                            .hint_text("Search games...")
                            .desired_width(200.0),
                    );
                });
            });
        });
    }

    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading(RichText::new("Library").size(16.0));
                ui.add_space(4.0);

                let tabs = [
                    (Tab::Library, "All Games"),
                    (Tab::Installed, "Installed"),
                    (Tab::Favorites, "Favorites"),
                    (Tab::RecentlyPlayed, "Recently Played"),
                ];

                for (tab, label) in &tabs {
                    let is_selected = self.state.active_tab == *tab;
                    if ui.selectable_label(is_selected, *label).clicked() {
                        self.state.active_tab = tab.clone();
                        self.state.load_apps();
                    }
                }

                ui.separator();
                ui.heading(RichText::new("Tools").size(16.0));
                ui.add_space(4.0);

                if ui
                    .selectable_label(self.state.active_tab == Tab::Controllers, "Controllers")
                    .clicked()
                {
                    self.state.active_tab = Tab::Controllers;
                }

                if ui
                    .selectable_label(self.state.active_tab == Tab::Settings, "Settings")
                    .clicked()
                {
                    self.state.active_tab = Tab::Settings;
                }

                ui.add_space(16.0);
                ui.separator();

                if ui.button("Play Store").clicked() {
                    self.launch_play_store();
                }
            });
    }

    fn launch_play_store(&mut self) {
        let config = self.state.config.clone();
        let runtime_manager = match &self.state.runtime_manager {
            Some(rm) => rm.clone(),
            None => {
                self.state
                    .set_notification("Runtime not initialized".into());
                return;
            }
        };

        if !config.play_store.enabled {
            self.state
                .set_notification("Play Store is disabled. Enable it in Settings.".into());
            return;
        }

        let rt = tokio::runtime::Handle::current();
        let manager = runtime_manager.clone();
        let runtime_name = config.default_runtime.clone();
        let result = tokio::task::block_in_place(|| {
            rt.block_on(async { manager.launch_play_store(&runtime_name).await })
        });

        match result {
            Ok(()) => self.state.set_notification("Opening Play Store...".into()),
            Err(e) => self
                .state
                .set_notification(format!("Failed to open Play Store: {}", e)),
        }
    }

    fn render_main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref notification) = self.state.notification.clone() {
                egui::TopBottomPanel::top("notification").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(notification).color(Color32::LIGHT_BLUE));
                        if ui.small_button("X").clicked() {
                            self.state.clear_notification();
                        }
                    });
                });
            }

            match self.state.active_tab.clone() {
                Tab::Library => self.render_library_tab(ui),
                Tab::Installed => self.render_installed_tab(ui),
                Tab::Favorites => self.render_favorites_tab(ui),
                Tab::RecentlyPlayed => self.render_recent_tab(ui),
                Tab::Controllers => self.render_controllers_tab(ui),
                Tab::Settings => self.render_settings_tab(ui),
                Tab::Setup => self.render_setup_tab(ui),
            }

            self.render_install_dialog(ctx);
        });
    }

    fn render_library_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Game Library");
        ui.separator();

        let apps: Vec<(String, String, String, bool, bool, gamedock_core::AppStatus)> = self
            .state
            .filtered_apps()
            .into_iter()
            .map(|a| {
                (
                    a.id.clone(),
                    a.name.clone(),
                    a.package_name.clone(),
                    a.is_favorite,
                    a.is_installed(),
                    a.status.clone(),
                )
            })
            .collect();

        if apps.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(
                    RichText::new("No games in library")
                        .size(20.0)
                        .color(Color32::GRAY),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Install games from APK files or the Play Store")
                        .color(Color32::GRAY),
                );
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut selected_idx = None;

            for (idx, (id, name, pkg_name, is_fav, is_installed, status)) in apps.iter().enumerate()
            {
                let is_selected = Some(idx) == self.state.selected_app;
                let bg = if is_selected {
                    Color32::from_rgba_premultiplied(50, 50, 80, 255)
                } else {
                    Color32::from_rgba_premultiplied(30, 30, 40, 255)
                };

                let frame = egui::Frame::default()
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                    .rounding(egui::Rounding::same(8.0))
                    .fill(bg);

                let name_c = name.clone();
                let id_c = id.clone();
                let response = ui
                    .push_id(id, |ui| {
                        frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let status_icon = match status {
                                    gamedock_core::AppStatus::Installed => {
                                        RichText::new("OK").color(Color32::GREEN)
                                    }
                                    _ => RichText::new("--").color(Color32::GRAY),
                                };
                                ui.label(status_icon);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(name).strong().size(16.0));
                                    ui.label(RichText::new(pkg_name).small().color(Color32::GRAY));
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if *is_fav {
                                        ui.label(
                                            RichText::new("*").color(Color32::YELLOW).size(20.0),
                                        );
                                    }

                                    if *is_installed {
                                        if ui.small_button("Play").clicked() {
                                            self.launch_app_by_id(&id_c, &name_c);
                                        }
                                    }
                                });
                            });
                        });
                    })
                    .response;

                if response.interact(egui::Sense::click()).clicked() {
                    selected_idx = Some(idx);
                }

                ui.add_space(4.0);
            }

            if let Some(idx) = selected_idx {
                self.state.selected_app = Some(idx);
            }
        });
    }

    fn render_installed_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Installed Games");
        ui.separator();

        let installed: Vec<(String, String, String)> = self
            .state
            .filtered_apps()
            .into_iter()
            .filter(|a| a.is_installed())
            .map(|a| (a.id.clone(), a.name.clone(), a.version_name.clone()))
            .collect();

        if installed.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(
                    RichText::new("No installed games")
                        .size(20.0)
                        .color(Color32::GRAY),
                );
            });
            return;
        }

        for (id, name, version) in &installed {
            ui.horizontal(|ui| {
                ui.label(RichText::new(name).strong());
                ui.label(RichText::new(version).small().color(Color32::GRAY));
                let id_c = id.clone();
                let n = name.clone();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("Play").clicked() {
                        self.launch_app_by_id(&id_c, &n);
                    }
                    if ui.small_button("Backup").clicked() {
                        self.backup_app(&n);
                    }
                });
            });
            ui.add_space(4.0);
        }
    }

    fn render_favorites_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Favorites");
        ui.separator();
        let favorites: Vec<(String, String)> = self
            .state
            .filtered_apps()
            .into_iter()
            .filter(|a| a.is_favorite)
            .map(|a| (a.id.clone(), a.name.clone()))
            .collect();

        if favorites.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(
                    RichText::new("No favorite games")
                        .size(20.0)
                        .color(Color32::GRAY),
                );
            });
            return;
        }

        for (id, name) in &favorites {
            ui.horizontal(|ui| {
                ui.label(RichText::new("*").color(Color32::YELLOW));
                ui.label(RichText::new(name).strong());
                let id_c = id.clone();
                let n = name.clone();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("Play").clicked() {
                        self.launch_app_by_id(&id_c, &n);
                    }
                });
            });
            ui.add_space(4.0);
        }
    }

    fn render_recent_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recently Played");
        ui.separator();
        let mut recent: Vec<(String, String, Option<chrono::DateTime<chrono::Utc>>)> = self
            .state
            .filtered_apps()
            .into_iter()
            .filter(|a| a.last_played.is_some())
            .map(|a| (a.id.clone(), a.name.clone(), a.last_played))
            .collect();
        recent.sort_by(|a, b| b.2.cmp(&a.2));
        recent.truncate(20);

        if recent.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(
                    RichText::new("No recently played games")
                        .size(20.0)
                        .color(Color32::GRAY),
                );
            });
            return;
        }

        for (id, name, last) in &recent {
            ui.horizontal(|ui| {
                ui.label(RichText::new(">").color(Color32::LIGHT_BLUE));
                ui.label(RichText::new(name).strong());
                if let Some(ref last_played) = last {
                    ui.label(
                        RichText::new(last_played.format("%Y-%m-%d %H:%M").to_string())
                            .small()
                            .color(Color32::GRAY),
                    );
                }
                let id_c = id.clone();
                let n = name.clone();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("Play").clicked() {
                        self.launch_app_by_id(&id_c, &n);
                    }
                });
            });
            ui.add_space(4.0);
        }
    }

    fn render_controllers_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Controller Settings");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Auto-detect controllers:");
            let mut auto_detect = self.state.config.controller.auto_detect;
            if ui.checkbox(&mut auto_detect, "").changed() {
                self.state.config.controller.auto_detect = auto_detect;
                let _ = self.state.config.save();
            }
        });

        ui.add_space(8.0);
        ui.heading(RichText::new("Connected Controllers").size(14.0));
        ui.add_space(4.0);

        if let Some(ref ctrl_mgr) = self.state.controller_manager {
            let rt = tokio::runtime::Handle::current();
            let controllers = tokio::task::block_in_place(|| {
                rt.block_on(async { ctrl_mgr.list_controllers().await })
            });
            if controllers.is_empty() {
                ui.label(
                    RichText::new("No controllers detected. Connect a controller and click Scan.")
                        .color(Color32::GRAY),
                );
            } else {
                for ctrl in &controllers {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(ctrl.display_name()).strong());
                        ui.label(
                            RichText::new(ctrl.device_path().display().to_string())
                                .small()
                                .color(Color32::GRAY),
                        );
                    });
                }
            }
        } else {
            ui.label(RichText::new("Controller manager not initialized.").color(Color32::GRAY));
        }

        ui.add_space(8.0);

        if ui.button("Scan for Controllers").clicked() {
            if let Some(ref ctrl_mgr) = self.state.controller_manager {
                let rt = tokio::runtime::Handle::current();
                let _ = tokio::task::block_in_place(|| {
                    rt.block_on(async { ctrl_mgr.scan_controllers().await })
                });
                self.state
                    .set_notification("Controller scan complete".into());
            }
        }

        ui.add_space(16.0);
        ui.heading(RichText::new("Mapping Profiles").size(14.0));
        ui.add_space(4.0);

        let mut selected_profile = 0usize;
        let profiles = [
            "Default",
            "FPS Profile",
            "Racing Profile",
            "Platformer Profile",
        ];
        for (i, profile) in profiles.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.radio_value(&mut selected_profile, i, *profile).clicked() {
                    self.state
                        .set_notification(format!("Selected profile: {}", profile));
                }
            });
        }

        ui.add_space(8.0);

        if ui.button("Create Default Profiles").clicked() {
            if let Some(ref ctrl_mgr) = self.state.controller_manager {
                let rt = tokio::runtime::Handle::current();
                let _ = tokio::task::block_in_place(|| {
                    rt.block_on(async { ctrl_mgr.create_default_profiles().await })
                });
                self.state
                    .set_notification("Default profiles created!".into());
            }
        }
    }

    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();

        ui.heading(RichText::new("Runtime").size(14.0));
        ui.horizontal(|ui| {
            ui.label("Default runtime:");
            ui.label(RichText::new(&self.state.config.default_runtime).strong());
        });

        ui.add_space(8.0);
        ui.heading(RichText::new("Optimization").size(14.0));
        {
            let mut gamemode = self.state.config.optimizer.gamemode;
            if ui.checkbox(&mut gamemode, "Enable GameMode").changed() {
                self.state.config.optimizer.gamemode = gamemode;
                let _ = self.state.config.save();
            }
        }
        {
            let mut mangohud = self.state.config.optimizer.mangohud;
            if ui.checkbox(&mut mangohud, "Enable MangoHUD").changed() {
                self.state.config.optimizer.mangohud = mangohud;
                let _ = self.state.config.save();
            }
        }
        {
            let mut gpu = self.state.config.optimizer.gpu_optimization;
            if ui.checkbox(&mut gpu, "GPU Optimization").changed() {
                self.state.config.optimizer.gpu_optimization = gpu;
                let _ = self.state.config.save();
            }
        }

        ui.add_space(8.0);
        ui.heading(RichText::new("Google Play Store").size(14.0));
        {
            let mut play_store = self.state.config.play_store.enabled;
            if ui
                .checkbox(&mut play_store, "Enable Play Store support")
                .changed()
            {
                self.state.config.play_store.enabled = play_store;
                let _ = self.state.config.save();
            }
        }
        {
            let mut auto_update = self.state.config.play_store.auto_update_apps;
            if ui.checkbox(&mut auto_update, "Auto-update apps").changed() {
                self.state.config.play_store.auto_update_apps = auto_update;
                let _ = self.state.config.save();
            }
        }

        ui.add_space(8.0);
        ui.heading(RichText::new("UI").size(14.0));
        ui.horizontal(|ui| {
            ui.label("Theme:");
            ui.label(&self.state.config.ui.theme);
        });

        ui.add_space(16.0);
        ui.separator();

        if ui.button("Reset to Defaults").clicked() {
            self.state.config = gamedock_core::AppConfig::default();
            let _ = self.state.config.save();
            self.state
                .set_notification("Settings reset to defaults".into());
        }
    }

    fn render_setup_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.heading(RichText::new("Welcome to GameDock").size(28.0).strong());
            ui.add_space(8.0);
            ui.label(
                RichText::new("Android gaming on Linux, made seamless")
                    .size(16.0)
                    .color(Color32::GRAY),
            );
            ui.add_space(32.0);

            ui.heading(RichText::new("Step 1: Install Waydroid").size(18.0));
            ui.add_space(4.0);
            ui.label("GameDock needs Waydroid to run Android apps.");
            ui.label("It will be installed automatically using your package manager.");
            ui.add_space(8.0);

            ui.heading(RichText::new("Step 2: Choose Google Play Store").size(18.0));
            ui.add_space(4.0);
            ui.label("With Play Store, you can install games directly.");
            ui.label("Without it, you'll need APK files.");
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 160.0);
                ui.checkbox(&mut self.state.setup_gapps, "Include Google Play Store");
            });

            ui.add_space(24.0);

            if self.state.installing_waydroid {
                ui.spinner();
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Installing... This may take a few minutes.")
                        .color(Color32::LIGHT_BLUE),
                );
                ui.label(
                    RichText::new(
                        "A ~1GB Android system image will be downloaded on first launch.",
                    )
                    .small()
                    .color(Color32::GRAY),
                );
            } else {
                let btn_label = if self.state.setup_gapps {
                    "Install with Play Store"
                } else {
                    "Install without Play Store"
                };

                if ui.button(RichText::new(btn_label).size(16.0)).clicked() {
                    self.state.installing_waydroid = true;
                    let runtime_manager = match &self.state.runtime_manager {
                        Some(rm) => rm.clone(),
                        None => return,
                    };
                    let gapps = self.state.setup_gapps;

                    let rt = tokio::runtime::Handle::current();
                    let result = tokio::task::block_in_place(|| {
                        rt.block_on(async {
                            let rm = runtime_manager.clone();
                            rm.initialize().await?;
                            if gapps {
                                rm.init_with_gapps().await
                            } else {
                                rm.init_vanilla().await
                            }
                        })
                    });

                    match result {
                        Ok(()) => {
                            self.state.first_run = false;
                            self.state.installing_waydroid = false;
                            self.state.active_tab = Tab::Library;
                            self.state.set_notification(
                                "Setup complete! You can now install games.".into(),
                            );
                        }
                        Err(e) => {
                            self.state.installing_waydroid = false;
                            self.state.set_notification(format!("Setup failed: {}", e));
                        }
                    }
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new("You can change this later in Settings.")
                        .small()
                        .color(Color32::GRAY),
                );
            }
        });
    }

    fn render_install_dialog(&mut self, ctx: &egui::Context) {
        if !self.state.show_install_dialog {
            return;
        }

        egui::Window::new("Install Package")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Select a package file to install:");
                ui.add_space(8.0);

                if ui.button("Browse APK/XAPK...").clicked() {
                    self.state
                        .set_notification("File dialog would open here".into());
                    self.state.show_install_dialog = false;
                }

                ui.add_space(8.0);
                ui.label("Supported formats: APK, XAPK, APKS, APKM");

                ui.add_space(16.0);
                if ui.button("Cancel").clicked() {
                    self.state.show_install_dialog = false;
                }
            });
    }

    fn launch_app_by_id(&mut self, app_id: &str, app_name: &str) {
        let config = self.state.config.clone();
        let runtime_manager = match &self.state.runtime_manager {
            Some(rm) => rm.clone(),
            None => {
                self.state
                    .set_notification("Runtime not initialized".into());
                return;
            }
        };
        let library = match &self.state.library {
            Some(lib) => lib.clone(),
            None => {
                self.state
                    .set_notification("Library not initialized".into());
                return;
            }
        };

        let event_bus = self.state.event_bus.clone();
        let app_id_owned = app_id.to_string();
        let app_name_owned = app_name.to_string();

        let rt = tokio::runtime::Handle::current();
        let result = tokio::task::block_in_place(|| {
            rt.block_on(async {
                let launcher = gamedock_launcher::AppLauncher::new(
                    config,
                    runtime_manager,
                    library,
                    event_bus,
                );
                launcher
                    .launch_with_optimization(
                        &app_id_owned,
                        self.state.config.optimizer.gamemode,
                        self.state.config.optimizer.mangohud,
                    )
                    .await
            })
        });

        match result {
            Ok(()) => self
                .state
                .set_notification(format!("Launching {}...", app_name_owned)),
            Err(e) => self
                .state
                .set_notification(format!("Failed to launch: {}", e)),
        }
    }

    fn backup_app(&mut self, app_name: &str) {
        self.state
            .set_notification(format!("Backing up {}...", app_name));
    }
}
