use gamedock_core::{AppConfig, AppInfo, Result};
use std::fmt;

pub struct DesktopEntry {
    pub name: String,
    pub comment: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: String,
    pub terminal: bool,
    pub startup_notify: bool,
    pub type_field: String,
}

impl DesktopEntry {
    pub fn from_app_info(app: &AppInfo, _config: &AppConfig) -> Result<Self> {
        let exec = format!(
            "gamedock-cli launch --app-id {}",
            app.id
        );

        let categories = app.categories.iter()
            .map(|c| match c {
                gamedock_core::Category::Action => "Game;ActionGame",
                gamedock_core::Category::Adventure => "Game;AdventureGame",
                gamedock_core::Category::Arcade => "Game;ArcadeGame",
                gamedock_core::Category::Puzzle => "Game;LogicGame",
                gamedock_core::Category::Racing => "Game;RacingGame",
                gamedock_core::Category::RolePlaying => "Game;RolePlaying",
                gamedock_core::Category::Simulation => "Game;Simulation",
                gamedock_core::Category::Sports => "Game;SportsGame",
                gamedock_core::Category::Strategy => "Game;StrategyGame",
                _ => "Game",
            })
            .next()
            .unwrap_or("Game")
            .to_string();

        let icon = app.icon_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "gamedock".to_string());

        Ok(Self {
            name: app.name.clone(),
            comment: Some(app.description.clone()),
            exec,
            icon: Some(icon),
            categories,
            terminal: false,
            startup_notify: true,
            type_field: "Application".to_string(),
        })
    }
}

impl fmt::Display for DesktopEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Desktop Entry]")?;
        writeln!(f, "Type={}", self.type_field)?;
        writeln!(f, "Name={}", self.name)?;
        if let Some(ref comment) = self.comment {
            writeln!(f, "Comment={}", comment)?;
        }
        writeln!(f, "Exec={}", self.exec)?;
        if let Some(ref icon) = self.icon {
            writeln!(f, "Icon={}", icon)?;
        }
        writeln!(f, "Categories={}", self.categories)?;
        writeln!(f, "Terminal={}", self.terminal)?;
        writeln!(f, "StartupNotify={}", self.startup_notify)?;
        writeln!(f, "Keywords=android;game;gamedock;")?;
        writeln!(f, "MimeType=x-scheme-handler/gamedock;")?;
        Ok(())
    }
}
