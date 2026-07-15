mod app;
mod state;

use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("gamedock-gui")
        .build()?;

    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("GameDock"),
        ..Default::default()
    };

    eframe::run_native(
        "GameDock",
        options,
        Box::new(|_cc| {
            let mut state = state::AppState::default();
            let _ = state.initialize();
            Ok(Box::new(app::GameDockApp::new(state)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}
