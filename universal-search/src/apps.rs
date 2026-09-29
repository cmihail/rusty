use gio::{prelude::*, DesktopAppInfo};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Application {
    pub name: String,
    pub icon_name: String,
    pub desktop_id: String,
    pub app_info: DesktopAppInfo,
    pub score: f64,
}

impl Application {
    pub fn from_app_info(app_info: DesktopAppInfo) -> Self {
        let name = app_info.name().to_string();
        let icon_name = app_info
            .icon()
            .and_then(|icon| icon.to_string())
            .unwrap_or_else(|| "application-x-executable".into())
            .to_string();
        let desktop_id = app_info.id().unwrap_or_default().to_string();

        Self {
            name,
            icon_name,
            desktop_id,
            app_info,
            score: 0.0,
        }
    }

    pub fn launch(&self) -> Result<(), glib::Error> {
        self.app_info
            .launch(&[], Option::<&gio::AppLaunchContext>::None)
    }

    fn fuzzy_match_score(&self, query: &str) -> f64 {
        if query.is_empty() {
            return 1.0;
        }

        let name_lower = self.name.to_lowercase();
        let query_lower = query.to_lowercase();

        // Exact match gets highest score
        if name_lower == query_lower {
            return 100.0;
        }

        // Starts with match gets high score
        if name_lower.starts_with(&query_lower) {
            return 90.0;
        }

        // Contains match gets medium score
        if name_lower.contains(&query_lower) {
            return 70.0;
        }

        // Fuzzy match algorithm
        fuzzy_match_string(&query_lower, &name_lower)
    }
}

impl PartialEq for Application {
    fn eq(&self, other: &Self) -> bool {
        self.desktop_id == other.desktop_id
    }
}

impl Eq for Application {}

impl PartialOrd for Application {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Application {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .score
            .partial_cmp(&self.score)
            .unwrap_or(Ordering::Equal)
    }
}

pub struct AppSearch {
    applications: Vec<Application>,
    frequents: HashMap<String, u32>,
}

impl Default for AppSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl AppSearch {
    pub fn new() -> Self {
        let mut apps = Self {
            applications: Vec::new(),
            frequents: HashMap::new(),
        };
        apps.reload();
        apps
    }

    pub fn reload(&mut self) {
        self.applications.clear();

        let app_infos = gio::AppInfo::all();
        for app_info in app_infos {
            if let Some(desktop_app_info) = app_info.downcast_ref::<DesktopAppInfo>() {
                if desktop_app_info.should_show() {
                    let app = Application::from_app_info(desktop_app_info.clone());
                    self.applications.push(app);
                }
            }
        }
    }

    pub fn get_all_apps(&self, limit: usize) -> Vec<Application> {
        let mut apps: Vec<Application> = self.applications.iter().take(limit).cloned().collect();
        for app in &mut apps {
            app.score = 1.0;
        }
        apps
    }

    pub fn fuzzy_query(&self, query: &str) -> Vec<Application> {
        let mut results: Vec<Application> = self
            .applications
            .iter()
            .filter_map(|app| {
                let score = app.fuzzy_match_score(query);
                if score > 0.5 {
                    let mut app_clone = app.clone();
                    app_clone.score = score;
                    Some(app_clone)
                } else {
                    None
                }
            })
            .collect();

        results.sort();
        results
    }

    pub fn increment_frequency(&mut self, desktop_id: &str) {
        *self.frequents.entry(desktop_id.to_string()).or_insert(0) += 1;
    }
}

// Simple fuzzy matching algorithm
fn fuzzy_match_string(pattern: &str, text: &str) -> f64 {
    if pattern.is_empty() {
        return 60.0;
    }

    if text.len() < pattern.len() {
        return 0.0;
    }

    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    let mut score = 0.0;
    let mut pattern_index = 0;
    let mut consecutive_matches = 0;

    for (text_index, &text_char) in text_chars.iter().enumerate() {
        if pattern_index < pattern_chars.len() && text_char == pattern_chars[pattern_index] {
            score += 10.0;

            // Bonus for consecutive matches
            if text_index > 0 && consecutive_matches > 0 {
                score += 5.0;
            }

            // Bonus for matches at word boundaries
            if text_index == 0
                || text_chars[text_index - 1] == ' '
                || text_chars[text_index - 1] == '-'
            {
                score += 10.0;
            }

            consecutive_matches += 1;
            pattern_index += 1;
        } else {
            consecutive_matches = 0;
        }
    }

    if pattern_index == pattern_chars.len() {
        // All pattern chars matched, apply completion bonus
        score += 20.0;

        // Length ratio bonus (shorter is better for same match)
        let length_ratio = pattern.len() as f64 / text.len() as f64;
        score += length_ratio * 10.0;

        score
    } else {
        0.0
    }
}
