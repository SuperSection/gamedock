use gamedock_core::{AppInfo, Category};
use regex::Regex;

pub struct SearchEngine;

impl SearchEngine {
    pub fn filter_apps(apps: &[AppInfo], query: &str) -> Vec<AppInfo> {
        if query.is_empty() {
            return apps.to_vec();
        }

        let pattern = format!("(?i){}", regex::escape(query));
        let re = Regex::new(&pattern).ok();

        apps.iter()
            .filter(|app| {
                if let Some(ref re) = re {
                    re.is_match(&app.name)
                        || re.is_match(&app.package_name)
                        || re.is_match(&app.author)
                        || re.is_match(&app.description)
                } else {
                    app.name.to_lowercase().contains(&query.to_lowercase())
                        || app
                            .package_name
                            .to_lowercase()
                            .contains(&query.to_lowercase())
                        || app.author.to_lowercase().contains(&query.to_lowercase())
                }
            })
            .cloned()
            .collect()
    }

    pub fn filter_by_category(apps: &[AppInfo], category: &Category) -> Vec<AppInfo> {
        apps.iter()
            .filter(|app| app.categories.contains(category))
            .cloned()
            .collect()
    }

    pub fn filter_installed(apps: &[AppInfo]) -> Vec<AppInfo> {
        apps.iter()
            .filter(|app| app.is_installed())
            .cloned()
            .collect()
    }

    pub fn filter_favorites(apps: &[AppInfo]) -> Vec<AppInfo> {
        apps.iter().filter(|app| app.is_favorite).cloned().collect()
    }

    pub fn sort_by_name(apps: &mut Vec<AppInfo>) {
        apps.sort_by(|a, b| a.name.cmp(&b.name));
    }

    pub fn sort_by_last_played(apps: &mut Vec<AppInfo>) {
        apps.sort_by(|a, b| {
            b.last_played
                .cmp(&a.last_played)
                .then_with(|| a.name.cmp(&b.name))
        });
    }

    pub fn sort_by_play_time(apps: &mut Vec<AppInfo>) {
        apps.sort_by(|a, b| {
            b.play_time_seconds
                .cmp(&a.play_time_seconds)
                .then_with(|| a.name.cmp(&b.name))
        });
    }

    pub fn sort_by_rating(apps: &mut Vec<AppInfo>) {
        apps.sort_by(|a, b| {
            b.rating
                .partial_cmp(&a.rating)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.name.cmp(&b.name))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gamedock_core::{AppStatus, Category};

    fn make_test_apps() -> Vec<AppInfo> {
        let mut apps = Vec::new();

        let mut app1 = AppInfo::new("com.game.one", "Alpha Game", "1.0", 1);
        app1.status = AppStatus::Installed;
        app1.is_favorite = true;
        app1.categories = vec![Category::Action];
        app1.rating = Some(4.5);
        apps.push(app1);

        let mut app2 = AppInfo::new("com.game.two", "Beta Game", "2.0", 2);
        app2.status = AppStatus::Installed;
        app2.categories = vec![Category::Puzzle];
        app2.rating = Some(3.8);
        apps.push(app2);

        let mut app3 = AppInfo::new("com.game.three", "Gamma Game", "3.0", 3);
        app3.status = AppStatus::NotInstalled;
        app3.categories = vec![Category::Action];
        app3.rating = Some(4.2);
        apps.push(app3);

        apps
    }

    #[test]
    fn test_filter_by_name() {
        let apps = make_test_apps();
        let results = SearchEngine::filter_apps(&apps, "Alpha");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alpha Game");
    }

    #[test]
    fn test_filter_by_package_name() {
        let apps = make_test_apps();
        let results = SearchEngine::filter_apps(&apps, "com.game.two");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].package_name, "com.game.two");
    }

    #[test]
    fn test_filter_empty_query() {
        let apps = make_test_apps();
        let results = SearchEngine::filter_apps(&apps, "");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_filter_case_insensitive() {
        let apps = make_test_apps();
        let results = SearchEngine::filter_apps(&apps, "alpha");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_filter_by_category() {
        let apps = make_test_apps();
        let results = SearchEngine::filter_by_category(&apps, &Category::Action);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_installed() {
        let apps = make_test_apps();
        let results = SearchEngine::filter_installed(&apps);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_favorites() {
        let apps = make_test_apps();
        let results = SearchEngine::filter_favorites(&apps);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alpha Game");
    }

    #[test]
    fn test_sort_by_name() {
        let mut apps = make_test_apps();
        SearchEngine::sort_by_name(&mut apps);
        assert_eq!(apps[0].name, "Alpha Game");
        assert_eq!(apps[1].name, "Beta Game");
        assert_eq!(apps[2].name, "Gamma Game");
    }

    #[test]
    fn test_sort_by_rating() {
        let mut apps = make_test_apps();
        SearchEngine::sort_by_rating(&mut apps);
        assert_eq!(apps[0].name, "Alpha Game");
        assert_eq!(apps[1].name, "Gamma Game");
        assert_eq!(apps[2].name, "Beta Game");
    }

    #[test]
    fn test_no_match() {
        let apps = make_test_apps();
        let results = SearchEngine::filter_apps(&apps, "nonexistent");
        assert!(results.is_empty());
    }
}
