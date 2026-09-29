pub mod apps;
pub mod brave_search;
pub mod calculator;
pub mod favorites;
pub mod files;
pub mod gemini;
pub mod history;
pub mod recent_usage;

use apps::{AppSearch, Application as AppEntry};
use brave_search::{BraveSearch, BraveSearchResult};
use calculator::{calculate, launch_calculator};
use favorites::Favorites;
use files::{FileSearchResult, FileSearcher, FileType};
use gdk4::prelude::*;
use gemini::Gemini;
use glib::Propagation;
use gtk4::{prelude::*, *};
use history::History;
use recent_usage::{RecentApp, RecentFile, RecentUsage};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

const DEFAULT_APP_DISPLAY_LIMIT: usize = 50;

struct SearchWindow {
    window: ApplicationWindow,
    entry: Entry,
    results_box: Box,
    #[allow(dead_code)]
    scrolled_window: ScrolledWindow,
    app_search: Rc<RefCell<AppSearch>>,
    file_searcher: FileSearcher,
    gemini: Gemini,
    brave_search: BraveSearch,
    history: Rc<RefCell<History>>,
    history_index: Rc<RefCell<usize>>,
    is_navigating: Rc<RefCell<bool>>,
    recent_usage: Rc<RefCell<RecentUsage>>,
    favorites: Rc<RefCell<Favorites>>,
    last_ai_query: Rc<RefCell<String>>,
    result_buttons: Rc<RefCell<Vec<Button>>>,
}

impl SearchWindow {
    fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(700)
            .title("Universal Search")
            .resizable(true)
            .css_classes(vec!["UniversalSearch"])
            .build();

        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .css_classes(vec!["Centered"])
            .build();

        // Search entry
        let entry = Entry::builder()
            .placeholder_text("Search apps, files, do math, or ask AI...")
            .margin_bottom(5)
            .build();

        entry.set_icon_from_icon_name(EntryIconPosition::Primary, Some("window-close-symbolic"));
        entry.set_icon_from_icon_name(EntryIconPosition::Secondary, Some("system-search-symbolic"));

        // Separator
        let separator = Separator::builder()
            .orientation(Orientation::Horizontal)
            .build();

        // Results container with scrolling
        let results_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();

        // Scrolled window for results - expands to fill available space
        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .hexpand(true)
            .child(&results_box)
            .build();

        main_box.append(&entry);
        main_box.append(&separator);
        main_box.append(&scrolled_window);

        window.set_child(Some(&main_box));

        let app_search = Rc::new(RefCell::new(AppSearch::new()));
        let file_searcher = FileSearcher::new();
        let gemini = Gemini::new();
        let brave_search = BraveSearch::new();
        let history = Rc::new(RefCell::new(History::new()));
        let history_index = Rc::new(RefCell::new(history.borrow().get_entries().len()));
        let is_navigating = Rc::new(RefCell::new(false));
        let recent_usage = Rc::new(RefCell::new(RecentUsage::new()));
        let favorites = Rc::new(RefCell::new(Favorites::new()));
        let last_ai_query = Rc::new(RefCell::new(String::new()));
        let result_buttons = Rc::new(RefCell::new(Vec::new()));

        let mut search_window = Self {
            window,
            entry,
            results_box,
            scrolled_window,
            app_search,
            file_searcher,
            gemini,
            brave_search,
            history,
            history_index,
            is_navigating,
            recent_usage,
            favorites,
            last_ai_query,
            result_buttons,
        };

        search_window.setup_signals();
        search_window.update_results("");

        search_window
    }

    fn setup_signals(&mut self) {
        let window = self.window.clone();
        let results_box = self.results_box.clone();
        let app_search = self.app_search.clone();
        let file_searcher = self.file_searcher.clone();
        let gemini = self.gemini.clone();
        let brave_search = self.brave_search.clone();
        let history = self.history.clone();
        let history_index = self.history_index.clone();
        let is_navigating = self.is_navigating.clone();
        let recent_usage = self.recent_usage.clone();
        let favorites = self.favorites.clone();
        let last_ai_query = self.last_ai_query.clone();
        let result_buttons = self.result_buttons.clone();

        // Search on text change
        let results_box_clone = results_box.clone();
        let app_search_clone = app_search.clone();
        let file_searcher_clone = file_searcher.clone();
        let gemini_clone = gemini.clone();
        let brave_search_clone = brave_search.clone();
        let recent_usage_clone = recent_usage.clone();
        let favorites_clone = favorites.clone();
        let window_clone = window.clone();
        let history_clone_for_changed = history.clone();
        let history_index_clone_for_changed = history_index.clone();
        let is_navigating_clone = is_navigating.clone();
        let last_ai_query_clone = last_ai_query.clone();
        let result_buttons_clone = result_buttons.clone();
        self.entry.connect_changed(move |entry| {
            if !*is_navigating_clone.borrow() {
                let text = entry.text().to_string();
                Self::update_search_results(
                    &results_box_clone,
                    &app_search_clone,
                    &file_searcher_clone,
                    &gemini_clone,
                    &brave_search_clone,
                    &recent_usage_clone,
                    &favorites_clone,
                    &text,
                    &window_clone,
                    &last_ai_query_clone,
                    &result_buttons_clone,
                );
                *history_index_clone_for_changed.borrow_mut() =
                    history_clone_for_changed.borrow().get_entries().len();
            }
        });

        // Handle enter key - trigger AI search
        let history_clone = history.clone();
        let gemini_clone_for_activate = gemini.clone();
        let brave_search_clone_for_activate = brave_search.clone();
        let last_ai_query_clone_for_activate = last_ai_query.clone();
        self.entry.connect_activate(move |entry| {
            let text = entry.text().to_string();
            if !text.trim().is_empty() {
                history_clone.borrow_mut().add(&text);
                gemini_clone_for_activate.search(text.clone());
                brave_search_clone_for_activate.search(text.clone());
                *last_ai_query_clone_for_activate.borrow_mut() = text;
            }
        });

        // Handle icon clicks
        let gemini_clone_for_icon = gemini.clone();
        let brave_search_clone_for_icon = brave_search.clone();
        let last_ai_query_clone_for_icon = last_ai_query.clone();
        self.entry
            .connect_icon_press(move |entry, icon_pos| match icon_pos {
                EntryIconPosition::Primary => {
                    entry.set_text("");
                    entry.grab_focus();
                }
                EntryIconPosition::Secondary => {
                    let text = entry.text().to_string();
                    if !text.trim().is_empty() {
                        gemini_clone_for_icon.search(text.clone());
                        brave_search_clone_for_icon.search(text.clone());
                        *last_ai_query_clone_for_icon.borrow_mut() = text;
                    }
                }
                _ => {}
            });

        // Handle escape key to close and Ctrl+number for result selection
        let window_key_controller = EventControllerKey::new();
        let window_clone = window.clone();
        let result_buttons_clone = self.result_buttons.clone();
        window_key_controller.connect_key_pressed(move |_, key, _, modifiers| {
            if key == gdk4::Key::Escape {
                window_clone.close();
                return Propagation::Stop;
            }

            // Check for Ctrl+number (0-9)
            if modifiers.contains(gdk4::ModifierType::CONTROL_MASK) {
                let button_index = match key {
                    gdk4::Key::_0 => Some(0),
                    gdk4::Key::_1 => Some(1),
                    gdk4::Key::_2 => Some(2),
                    gdk4::Key::_3 => Some(3),
                    gdk4::Key::_4 => Some(4),
                    gdk4::Key::_5 => Some(5),
                    gdk4::Key::_6 => Some(6),
                    gdk4::Key::_7 => Some(7),
                    gdk4::Key::_8 => Some(8),
                    gdk4::Key::_9 => Some(9),
                    _ => None,
                };

                if let Some(index) = button_index {
                    let buttons = result_buttons_clone.borrow();
                    if let Some(button) = buttons.get(index) {
                        button.emit_clicked();
                        return Propagation::Stop;
                    }
                }
            }

            Propagation::Proceed
        });
        self.window.add_controller(window_key_controller);

        // Handle entry key presses
        let entry_key_controller = EventControllerKey::new();
        let history_clone = history.clone();
        let entry_clone = self.entry.clone();
        entry_key_controller.connect_key_pressed(move |_, key, _, _| {
            let mut key_handled = false;
            match key {
                gdk4::Key::Up => {
                    *is_navigating.borrow_mut() = true;
                    let mut index = history_index.borrow_mut();
                    let history_entries = history_clone.borrow().get_entries().clone();
                    if *index > 0 {
                        *index -= 1;
                        entry_clone.set_text(&history_entries[*index]);
                        key_handled = true;
                    }
                    *is_navigating.borrow_mut() = false;
                }
                gdk4::Key::Down => {
                    *is_navigating.borrow_mut() = true;
                    let mut index = history_index.borrow_mut();
                    let history_entries = history_clone.borrow().get_entries().clone();
                    if !history_entries.is_empty() && *index < history_entries.len() - 1 {
                        *index += 1;
                        entry_clone.set_text(&history_entries[*index]);
                        key_handled = true;
                    }
                    *is_navigating.borrow_mut() = false;
                }
                _ => {}
            }

            if key_handled {
                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        });
        self.entry.add_controller(entry_key_controller);

        // Focus entry when window is shown
        let entry_clone = self.entry.clone();
        self.window.connect_show(move |_| {
            entry_clone.grab_focus();
        });

        // Refresh UI when Gemini result changes
        let results_box_for_gemini = results_box.clone();
        let app_search_for_gemini = app_search.clone();
        let file_searcher_for_gemini = file_searcher.clone();
        let gemini_for_refresh = gemini.clone();
        let brave_search_for_gemini = brave_search.clone();
        let recent_usage_for_gemini = recent_usage.clone();
        let favorites_for_gemini = favorites.clone();
        let window_for_gemini = window.clone();
        let entry_for_gemini = self.entry.clone();
        let last_ai_query_for_gemini = last_ai_query.clone();
        let result_buttons_for_gemini = result_buttons.clone();
        gemini.connect_notify_local(Some("result"), move |_obj, _| {
            let current_text = entry_for_gemini.text().to_string();
            if !current_text.is_empty() {
                Self::update_search_results(
                    &results_box_for_gemini,
                    &app_search_for_gemini,
                    &file_searcher_for_gemini,
                    &gemini_for_refresh,
                    &brave_search_for_gemini,
                    &recent_usage_for_gemini,
                    &favorites_for_gemini,
                    &current_text,
                    &window_for_gemini,
                    &last_ai_query_for_gemini,
                    &result_buttons_for_gemini,
                );
            }
        });

        // Refresh UI when Brave Search results change
        let results_box_for_brave = results_box.clone();
        let app_search_for_brave = app_search.clone();
        let file_searcher_for_brave = file_searcher.clone();
        let gemini_for_brave = gemini.clone();
        let brave_search_for_refresh = brave_search.clone();
        let recent_usage_for_brave = recent_usage.clone();
        let favorites_for_brave = favorites.clone();
        let window_for_brave = window.clone();
        let entry_for_brave = self.entry.clone();
        let last_ai_query_for_brave = last_ai_query.clone();
        let result_buttons_for_brave = result_buttons.clone();

        // Listen to loading state changes
        brave_search.connect_notify_local(Some("loading"), {
            let results_box_for_brave = results_box_for_brave.clone();
            let app_search_for_brave = app_search_for_brave.clone();
            let file_searcher_for_brave = file_searcher_for_brave.clone();
            let gemini_for_brave = gemini_for_brave.clone();
            let brave_search_for_refresh = brave_search_for_refresh.clone();
            let recent_usage_for_brave = recent_usage_for_brave.clone();
            let favorites_for_brave = favorites_for_brave.clone();
            let window_for_brave = window_for_brave.clone();
            let entry_for_brave = entry_for_brave.clone();
            let last_ai_query_for_brave = last_ai_query_for_brave.clone();
            let result_buttons_for_brave = result_buttons_for_brave.clone();
            move |_obj, _| {
                let current_text = entry_for_brave.text().to_string();
                if !current_text.is_empty() {
                    Self::update_search_results(
                        &results_box_for_brave,
                        &app_search_for_brave,
                        &file_searcher_for_brave,
                        &gemini_for_brave,
                        &brave_search_for_refresh,
                        &recent_usage_for_brave,
                        &favorites_for_brave,
                        &current_text,
                        &window_for_brave,
                        &last_ai_query_for_brave,
                        &result_buttons_for_brave,
                    );
                }
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn update_search_results(
        results_box: &Box,
        app_search: &Rc<RefCell<AppSearch>>,
        file_searcher: &FileSearcher,
        gemini: &Gemini,
        brave_search: &BraveSearch,
        recent_usage: &Rc<RefCell<RecentUsage>>,
        favorites: &Rc<RefCell<Favorites>>,
        query: &str,
        window: &ApplicationWindow,
        last_ai_query: &Rc<RefCell<String>>,
        result_buttons: &Rc<RefCell<Vec<Button>>>,
    ) {
        // Clear previous results
        while let Some(child) = results_box.first_child() {
            results_box.remove(&child);
        }

        // Clear previous button references
        result_buttons.borrow_mut().clear();

        // Track the current result index for numbering
        let mut result_index = 0;

        if query.is_empty() {
            // Show recent apps and files when no query and reset AI services
            gemini.reset();
            brave_search.reset();
            *last_ai_query.borrow_mut() = String::new();

            let recent = recent_usage.borrow();
            let recent_apps = recent.get_recent_apps();
            let recent_files = recent.get_recent_files();

            // Get favorite apps
            let favorite_desktop_ids: HashSet<String> =
                favorites.borrow().get_favorites().into_iter().collect();

            // If no recent items, show favorites first (alphabetically), then other apps up to limit
            if recent_apps.is_empty() && recent_files.is_empty() {
                let all_apps = app_search.borrow().get_all_apps(usize::MAX);

                // Separate favorite and non-favorite apps
                let mut favorite_apps: Vec<AppEntry> = Vec::new();
                let mut other_apps: Vec<AppEntry> = Vec::new();

                for app in all_apps {
                    if favorite_desktop_ids.contains(&app.desktop_id) {
                        favorite_apps.push(app);
                    } else {
                        other_apps.push(app);
                    }
                }

                // Sort favorite apps alphabetically
                favorite_apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                // Calculate total apps to show: at least all favorites, but up to the limit
                let favorite_count = favorite_apps.len();
                let total_to_show = favorite_count.max(DEFAULT_APP_DISPLAY_LIMIT);
                let other_apps_to_show = total_to_show.saturating_sub(favorite_count);

                // Display favorite apps first
                for app in favorite_apps {
                    let index = if result_index < 10 {
                        Some(result_index)
                    } else {
                        None
                    };
                    let (widget, button) = Self::create_result_button(
                        &app,
                        window,
                        app_search,
                        recent_usage,
                        favorites,
                        index,
                    );
                    if result_index < 10 {
                        result_buttons.borrow_mut().push(button);
                    }
                    results_box.append(&widget);
                    result_index += 1;
                }

                // Then display other apps up to the calculated limit
                for app in other_apps.iter().take(other_apps_to_show) {
                    let index = if result_index < 10 {
                        Some(result_index)
                    } else {
                        None
                    };
                    let (widget, button) = Self::create_result_button(
                        app,
                        window,
                        app_search,
                        recent_usage,
                        favorites,
                        index,
                    );
                    if result_index < 10 {
                        result_buttons.borrow_mut().push(button);
                    }
                    results_box.append(&widget);
                    result_index += 1;
                }
            } else {
                // Show favorite apps first (alphabetically) when there are recent items
                let all_apps = app_search.borrow().get_all_apps(usize::MAX);
                let mut favorite_apps: Vec<AppEntry> = all_apps
                    .into_iter()
                    .filter(|app| favorite_desktop_ids.contains(&app.desktop_id))
                    .collect();

                // Sort favorite apps alphabetically
                favorite_apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                // Display favorite apps first
                for app in favorite_apps {
                    let index = if result_index < 10 {
                        Some(result_index)
                    } else {
                        None
                    };
                    let (widget, button) = Self::create_result_button(
                        &app,
                        window,
                        app_search,
                        recent_usage,
                        favorites,
                        index,
                    );
                    if result_index < 10 {
                        result_buttons.borrow_mut().push(button);
                    }
                    results_box.append(&widget);
                    result_index += 1;
                }

                // Then show recent apps
                for recent_app in &recent_apps {
                    // Skip if already shown as favorite
                    if favorite_desktop_ids.contains(&recent_app.desktop_id) {
                        continue;
                    }
                    let index = if result_index < 10 {
                        Some(result_index)
                    } else {
                        None
                    };
                    let (widget, button) = Self::create_recent_app_button(
                        recent_app,
                        window,
                        recent_usage,
                        favorites,
                        index,
                    );
                    if result_index < 10 {
                        result_buttons.borrow_mut().push(button);
                    }
                    results_box.append(&widget);
                    result_index += 1;
                }

                // Show recent files after apps
                for recent_file in &recent_files {
                    let index = if result_index < 10 {
                        Some(result_index)
                    } else {
                        None
                    };
                    let (widget, button) =
                        Self::create_recent_file_button(recent_file, window, recent_usage, index);
                    if result_index < 10 {
                        result_buttons.borrow_mut().push(button);
                    }
                    results_box.append(&widget.upcast::<Widget>());
                    result_index += 1;
                }
            }
            return;
        }

        // Check if AI searches are active and query matches last AI query
        let last_query = last_ai_query.borrow().clone();
        let query_matches_ai_search = query == last_query;
        let has_ai_results = query_matches_ai_search
            && (gemini.loading()
                || !gemini.result().is_empty()
                || brave_search.loading()
                || !brave_search.results().is_empty());

        // Get all app results
        let app_results = app_search.borrow().fuzzy_query(query);

        // Try to calculate math result and search files if query is long enough
        if query.len() >= 3 {
            // Add calculator result if available
            if let Some(result) = calculate(query) {
                let index = if result_index < 10 {
                    Some(result_index)
                } else {
                    None
                };
                let (widget, button) =
                    Self::create_calculator_button(query, &result, window, index);
                if result_index < 10 {
                    result_buttons.borrow_mut().push(button);
                }
                results_box.append(&widget);
                result_index += 1;
            }

            // Search files
            if let Ok(file_results) = file_searcher.search(query) {
                // Add app results (limit to 2 if AI results present)
                let app_limit = if has_ai_results { 2 } else { app_results.len() };
                for app in app_results.iter().take(app_limit) {
                    let index = if result_index < 10 {
                        Some(result_index)
                    } else {
                        None
                    };
                    let (widget, button) = Self::create_result_button(
                        app,
                        window,
                        app_search,
                        recent_usage,
                        favorites,
                        index,
                    );
                    if result_index < 10 {
                        result_buttons.borrow_mut().push(button);
                    }
                    results_box.append(&widget);
                    result_index += 1;
                }

                // Add file results (limit to 2 if AI results present)
                let file_limit = if has_ai_results {
                    2
                } else {
                    file_results.len()
                };
                for file_result in file_results.iter().take(file_limit) {
                    let index = if result_index < 10 {
                        Some(result_index)
                    } else {
                        None
                    };
                    let (widget, button) =
                        Self::create_file_result_button(file_result, window, recent_usage, index);
                    if result_index < 10 {
                        result_buttons.borrow_mut().push(button);
                    }
                    results_box.append(&widget.upcast::<Widget>());
                    result_index += 1;
                }
            } else {
                // If file search fails, just show app results
                let app_limit = if has_ai_results { 2 } else { app_results.len() };
                for app in app_results.iter().take(app_limit) {
                    let index = if result_index < 10 {
                        Some(result_index)
                    } else {
                        None
                    };
                    let (widget, button) = Self::create_result_button(
                        app,
                        window,
                        app_search,
                        recent_usage,
                        favorites,
                        index,
                    );
                    if result_index < 10 {
                        result_buttons.borrow_mut().push(button);
                    }
                    results_box.append(&widget);
                    result_index += 1;
                }
            }
        } else {
            // For short queries, only show app results
            let app_limit = if has_ai_results { 2 } else { app_results.len() };
            for app in app_results.iter().take(app_limit) {
                let index = if result_index < 10 {
                    Some(result_index)
                } else {
                    None
                };
                let (widget, button) = Self::create_result_button(
                    app,
                    window,
                    app_search,
                    recent_usage,
                    favorites,
                    index,
                );
                if result_index < 10 {
                    result_buttons.borrow_mut().push(button);
                }
                results_box.append(&widget);
                result_index += 1;
            }
        }

        // Add Gemini result if available and query matches last AI search (after apps and files)
        if query_matches_ai_search && (gemini.loading() || !gemini.result().is_empty()) {
            let gemini_button = Self::create_gemini_result_button(gemini);
            results_box.append(&gemini_button);
        }

        // Add Brave search results if available and query matches last AI search (after apps and files)
        if query_matches_ai_search && (brave_search.loading() || !brave_search.results().is_empty())
        {
            let brave_results_widget = Self::create_brave_results_widget(brave_search, window);
            results_box.append(&brave_results_widget);
        }
    }

    fn create_result_button(
        app: &AppEntry,
        window: &ApplicationWindow,
        app_search: &Rc<RefCell<AppSearch>>,
        recent_usage: &Rc<RefCell<RecentUsage>>,
        favorites: &Rc<RefCell<Favorites>>,
        index: Option<usize>,
    ) -> (Box, Button) {
        // Create outer container box
        let outer_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .hexpand(true)
            .margin_end(8)
            .build();

        // Create button content box with proper sizing
        let button_content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        // Icon with fixed size to avoid layout conflicts
        let icon = Image::builder()
            .icon_name(&app.icon_name)
            .pixel_size(16)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        // Label with proper sizing constraints
        let label = Label::builder()
            .label(&app.name)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::End)
            .single_line_mode(true)
            .build();

        button_content.append(&icon);
        button_content.append(&label);

        // Add number label inside button if index is 0-9
        if let Some(idx) = index {
            if idx < 10 {
                let number_label = Label::builder()
                    .label(idx.to_string())
                    .css_classes(vec!["ResultNumber"])
                    .valign(Align::Center)
                    .build();
                button_content.append(&number_label);
            }
        }

        let button = Button::builder()
            .child(&button_content)
            .tooltip_text(&app.name)
            .hexpand(true)
            .height_request(16)
            .build();

        let app_clone = app.clone();
        let window_clone = window.clone();
        let app_search_clone = app_search.clone();
        let recent_usage_clone = recent_usage.clone();

        button.connect_clicked(move |_| {
            if let Err(e) = app_clone.launch() {
                eprintln!("Failed to launch {}: {}", app_clone.name, e);
            } else {
                app_search_clone
                    .borrow_mut()
                    .increment_frequency(&app_clone.desktop_id);
                // Track recent usage
                recent_usage_clone.borrow_mut().add_recent_app(
                    &app_clone.desktop_id,
                    &app_clone.name,
                    &app_clone.icon_name,
                );
            }
            window_clone.set_visible(false);
        });

        // Create favorite icon button
        let is_favorited = favorites.borrow().is_favorited(&app.desktop_id);
        let fav_icon_name = if is_favorited {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        };

        let fav_icon = Image::builder()
            .icon_name(fav_icon_name)
            .pixel_size(16)
            .build();

        let fav_button = Button::builder()
            .child(&fav_icon)
            .tooltip_text(if is_favorited {
                "Remove from favorites"
            } else {
                "Add to favorites"
            })
            .valign(Align::Center)
            .height_request(16)
            .build();

        let desktop_id = app.desktop_id.clone();
        let favorites_clone = favorites.clone();
        let fav_icon_clone = fav_icon.clone();
        fav_button.connect_clicked(move |btn| {
            let is_now_favorited = favorites_clone.borrow_mut().toggle(&desktop_id);
            let new_icon = if is_now_favorited {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            };
            fav_icon_clone.set_icon_name(Some(new_icon));
            btn.set_tooltip_text(Some(if is_now_favorited {
                "Remove from favorites"
            } else {
                "Add to favorites"
            }));
        });

        outer_box.append(&button);
        outer_box.append(&fav_button);

        (outer_box, button)
    }

    fn create_recent_app_button(
        recent_app: &RecentApp,
        window: &ApplicationWindow,
        recent_usage: &Rc<RefCell<RecentUsage>>,
        favorites: &Rc<RefCell<Favorites>>,
        index: Option<usize>,
    ) -> (Box, Button) {
        // Create outer container box
        let outer_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .hexpand(true)
            .margin_end(8)
            .build();

        // Create button content box with proper sizing
        let button_content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        // Icon with fixed size to avoid layout conflicts
        let icon = Image::builder()
            .icon_name(&recent_app.icon_name)
            .pixel_size(16)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        // Label with proper sizing constraints
        let label = Label::builder()
            .label(&recent_app.name)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::End)
            .single_line_mode(true)
            .build();

        button_content.append(&icon);
        button_content.append(&label);

        // Add number label inside button if index is 0-9
        if let Some(idx) = index {
            if idx < 10 {
                let number_label = Label::builder()
                    .label(idx.to_string())
                    .css_classes(vec!["ResultNumber"])
                    .valign(Align::Center)
                    .build();
                button_content.append(&number_label);
            }
        }

        let button = Button::builder()
            .child(&button_content)
            .tooltip_text(&recent_app.name)
            .hexpand(true)
            .height_request(16)
            .build();

        let desktop_id = recent_app.desktop_id.clone();
        let name = recent_app.name.clone();
        let icon_name = recent_app.icon_name.clone();
        let window_clone = window.clone();
        let recent_usage_clone = recent_usage.clone();

        button.connect_clicked(move |_| {
            // Try to launch the app by desktop_id
            if let Some(app_info) = gio::DesktopAppInfo::new(&desktop_id) {
                if let Err(e) = app_info.launch(&[], None::<&gio::AppLaunchContext>) {
                    eprintln!("Failed to launch {}: {}", name, e);
                } else {
                    // Update recent usage (move to front)
                    recent_usage_clone
                        .borrow_mut()
                        .add_recent_app(&desktop_id, &name, &icon_name);
                }
            } else {
                eprintln!("Failed to find desktop app: {}", desktop_id);
            }
            window_clone.set_visible(false);
        });

        // Create favorite icon button
        let is_favorited = favorites.borrow().is_favorited(&recent_app.desktop_id);
        let fav_icon_name = if is_favorited {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        };

        let fav_icon = Image::builder()
            .icon_name(fav_icon_name)
            .pixel_size(16)
            .build();

        let fav_button = Button::builder()
            .child(&fav_icon)
            .tooltip_text(if is_favorited {
                "Remove from favorites"
            } else {
                "Add to favorites"
            })
            .valign(Align::Center)
            .height_request(16)
            .build();

        let desktop_id_for_fav = recent_app.desktop_id.clone();
        let favorites_clone = favorites.clone();
        let fav_icon_clone = fav_icon.clone();
        fav_button.connect_clicked(move |btn| {
            let is_now_favorited = favorites_clone.borrow_mut().toggle(&desktop_id_for_fav);
            let new_icon = if is_now_favorited {
                "starred-symbolic"
            } else {
                "non-starred-symbolic"
            };
            fav_icon_clone.set_icon_name(Some(new_icon));
            btn.set_tooltip_text(Some(if is_now_favorited {
                "Remove from favorites"
            } else {
                "Add to favorites"
            }));
        });

        outer_box.append(&button);
        outer_box.append(&fav_button);

        (outer_box, button)
    }

    fn create_recent_file_button(
        recent_file: &RecentFile,
        window: &ApplicationWindow,
        recent_usage: &Rc<RefCell<RecentUsage>>,
        index: Option<usize>,
    ) -> (Box, Button) {
        // Create outer container box
        let outer_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .hexpand(true)
            .margin_end(8)
            .build();

        // Create button content box with proper sizing
        let button_content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        // Choose icon based on file type (simplified)
        let icon_name = if std::path::Path::new(&recent_file.path).is_dir() {
            "folder"
        } else {
            "text-x-generic"
        };

        let icon = Image::builder()
            .icon_name(icon_name)
            .pixel_size(16)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        // Show full path in label
        let label = Label::builder()
            .label(&recent_file.path)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::Start) // Ellipsize from start for file paths
            .single_line_mode(true)
            .build();

        button_content.append(&icon);
        button_content.append(&label);

        // Add number label inside button if index is 0-9
        if let Some(idx) = index {
            if idx < 10 {
                let number_label = Label::builder()
                    .label(idx.to_string())
                    .css_classes(vec!["ResultNumber"])
                    .valign(Align::Center)
                    .build();
                button_content.append(&number_label);
            }
        }

        let button = Button::builder()
            .child(&button_content)
            .tooltip_text(&recent_file.path)
            .hexpand(true)
            .height_request(16)
            .build();

        let path = recent_file.path.clone();
        let window_clone = window.clone();
        let recent_usage_clone = recent_usage.clone();

        button.connect_clicked(move |_| {
            // Open the file/directory using default application
            if let Err(e) = Self::open_path(&path) {
                eprintln!("Failed to open {}: {}", path, e);
            } else {
                // Update recent usage (move to front)
                recent_usage_clone.borrow_mut().add_recent_file(&path);
            }
            window_clone.set_visible(false);
        });

        // Create folder open button
        let folder_icon = Image::builder()
            .icon_name("folder-open-symbolic")
            .pixel_size(16)
            .build();

        let folder_button = Button::builder()
            .child(&folder_icon)
            .tooltip_text("Open containing folder")
            .valign(Align::Center)
            .height_request(16)
            .build();

        let path_for_folder = recent_file.path.clone();
        let window_clone_for_folder = window.clone();
        folder_button.connect_clicked(move |_| {
            // Get parent directory
            let path_obj = std::path::Path::new(&path_for_folder);
            let dir_to_open = if path_obj.is_dir() {
                path_for_folder.clone()
            } else {
                path_obj
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| path_for_folder.clone())
            };

            if let Err(e) = Self::open_path(&dir_to_open) {
                eprintln!("Failed to open directory {}: {}", dir_to_open, e);
            }
            window_clone_for_folder.set_visible(false);
        });

        outer_box.append(&button);
        outer_box.append(&folder_button);

        (outer_box, button)
    }

    fn create_calculator_button(
        expression: &str,
        result: &str,
        window: &ApplicationWindow,
        index: Option<usize>,
    ) -> (Widget, Button) {
        // Create button content box similar to app buttons
        let button_content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        // Calculator icon
        let icon = Image::builder()
            .icon_name("org.gnome.Calculator")
            .pixel_size(16)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        // Result label formatted as "= result"
        let result_text = format!("= {}", result);
        let label = Label::builder()
            .label(&result_text)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::End)
            .single_line_mode(true)
            .build();

        button_content.append(&icon);
        button_content.append(&label);

        // Add number label inside button if index is 0-9
        if let Some(idx) = index {
            if idx < 10 {
                let number_label = Label::builder()
                    .label(idx.to_string())
                    .css_classes(vec!["ResultNumber"])
                    .valign(Align::Center)
                    .build();
                button_content.append(&number_label);
            }
        }

        let button = Button::builder()
            .child(&button_content)
            .tooltip_text(format!("Click to open calculator with: {}", expression))
            .hexpand(true)
            .height_request(32)
            .build();

        let expression_clone = expression.to_string();
        let window_clone = window.clone();

        button.connect_clicked(move |_| {
            if let Err(e) = launch_calculator(&expression_clone) {
                eprintln!("Failed to launch calculator: {}", e);
            }
            window_clone.set_visible(false);
        });

        (button.clone().upcast(), button)
    }

    fn create_gemini_result_button(gemini: &Gemini) -> Widget {
        // Main container with AiResult class
        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .css_classes(vec!["AiResult"])
            .build();

        // Header with icon and title
        let header_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        // Spinner for loading state - visibility controlled by loading state
        let spinner = Spinner::new();
        spinner.set_visible(gemini.loading());
        if gemini.loading() {
            spinner.start();
        }

        // Gemini icon
        let icon = Image::builder()
            .icon_name("applications-internet")
            .pixel_size(16)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        // Title label
        let title_label = Label::builder()
            .label("<b>Gemini</b>")
            .use_markup(true)
            .xalign(0.0)
            .hexpand(true)
            .build();

        header_box.append(&spinner);
        header_box.append(&icon);
        header_box.append(&title_label);

        // Result label (no longer in scrolled window)
        let result_label = Label::builder()
            .label(gemini.result())
            .use_markup(true)
            .wrap(true)
            .xalign(0.0)
            .selectable(true)
            .build();

        main_box.append(&header_box);
        main_box.append(&result_label);

        // Connect to property changes to update UI
        let spinner_clone = spinner.clone();
        let result_label_clone = result_label.clone();
        gemini.connect_notify_local(Some("loading"), move |obj, _| {
            let loading = obj.loading();
            spinner_clone.set_visible(loading);
            if loading {
                spinner_clone.start();
            } else {
                spinner_clone.stop();
            }
        });

        gemini.connect_notify_local(Some("result"), move |obj, _| {
            result_label_clone.set_markup(&obj.result());
        });

        main_box.upcast()
    }

    fn create_brave_results_widget(
        brave_search: &BraveSearch,
        window: &ApplicationWindow,
    ) -> Widget {
        if brave_search.loading() {
            // Loading state
            let loading_box = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(8)
                .css_classes(vec!["BraveResultsLoading"])
                .build();

            let header_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .build();

            let spinner = Spinner::new();
            spinner.start();

            let icon = Image::builder()
                .icon_name("applications-internet")
                .pixel_size(16)
                .build();

            let title_label = Label::builder()
                .label("Web results")
                .xalign(0.0)
                .hexpand(true)
                .build();

            header_box.append(&spinner);
            header_box.append(&icon);
            header_box.append(&title_label);

            let waiting_label = Label::builder()
                .label("Waiting for results...")
                .xalign(0.0)
                .build();

            loading_box.append(&header_box);
            loading_box.append(&waiting_label);

            loading_box.upcast()
        } else {
            // Results state
            let results = brave_search.results();

            let results_box = Box::builder()
                .orientation(Orientation::Vertical)
                .spacing(8)
                .build();

            for result in results {
                let result_widget = Self::create_brave_search_result_entry(&result, window);
                results_box.append(&result_widget);
            }

            results_box.upcast()
        }
    }

    /// Accent color for markup text, resolved from the active GTK theme.
    ///
    /// `accent_color` is the token themes tune for accent *text*, while
    /// `theme_selected_bg_color` is a fill and can be too dark to read at this
    /// size, so the lighter of the two wins. Pango only accepts `#rrggbb`, not
    /// the `rgb()` form `RGBA::to_string` produces.
    fn accent_text_color(widget: &impl IsA<Widget>) -> String {
        let style = widget.as_ref().style_context();
        let luma = |c: &gdk4::RGBA| 0.2126 * c.red() + 0.7152 * c.green() + 0.0722 * c.blue();

        ["accent_color", "theme_selected_bg_color"]
            .iter()
            .filter_map(|name| style.lookup_color(name))
            .max_by(|a, b| luma(a).total_cmp(&luma(b)))
            .map(|c| {
                let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                format!(
                    "#{:02x}{:02x}{:02x}",
                    channel(c.red()),
                    channel(c.green()),
                    channel(c.blue())
                )
            })
            .unwrap_or_else(|| "#74c0fc".to_string())
    }

    fn create_brave_search_result_entry(
        result: &BraveSearchResult,
        window: &ApplicationWindow,
    ) -> Widget {
        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .css_classes(vec!["BraveSearchResultEntry"])
            .build();

        // Header with icon and title
        let header_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .tooltip_text(&result.url)
            .build();

        let icon = Image::builder()
            .icon_name("applications-internet")
            .pixel_size(16)
            .build();

        let content_box = Box::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .build();

        // Profile name if available
        if let Some(profile) = &result.profile {
            let profile_label = Label::builder()
                .label(&profile.name)
                .xalign(0.0)
                .max_width_chars(40)
                .single_line_mode(true)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            content_box.append(&profile_label);
        }

        // Title with highlighting
        let title_label = Label::builder()
            .use_markup(true)
            .wrap(true)
            .xalign(0.0)
            .max_width_chars(40)
            .single_line_mode(true)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        title_label.set_label(&format!(
            "<span foreground=\"{}\">{}</span>",
            Self::accent_text_color(&title_label),
            glib::markup_escape_text(&result.title)
        ));
        content_box.append(&title_label);

        header_box.append(&icon);
        header_box.append(&content_box);

        // Description
        let description_label = Label::builder()
            .label(&result.description)
            .wrap(true)
            .use_markup(true)
            .xalign(0.0)
            .selectable(true)
            .build();

        main_box.append(&header_box);
        main_box.append(&description_label);

        // Make it clickable
        let url = result.url.clone();
        let window_clone = window.clone();
        let gesture_click = gtk4::GestureClick::new();
        gesture_click.connect_released(move |_, _, _, _| {
            let _ = Self::open_url(&url, &window_clone);
            window_clone.set_visible(false);
        });
        main_box.add_controller(gesture_click);

        main_box.upcast()
    }

    fn create_file_result_button(
        file_result: &FileSearchResult,
        window: &ApplicationWindow,
        recent_usage: &Rc<RefCell<RecentUsage>>,
        index: Option<usize>,
    ) -> (Box, Button) {
        // Create outer container box
        let outer_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .hexpand(true)
            .margin_end(8)
            .build();

        // Create button content box similar to app buttons
        let button_content = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        // Choose icon based on file type
        let icon_name = match file_result.file_type {
            FileType::Directory => "folder",
            FileType::File => "text-x-generic",
        };

        let icon = Image::builder()
            .icon_name(icon_name)
            .pixel_size(16)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        // Use the full path as the label
        let label = Label::builder()
            .label(&file_result.path)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::Start) // Ellipsize from start for file paths
            .single_line_mode(true)
            .build();

        button_content.append(&icon);
        button_content.append(&label);

        // Add number label inside button if index is 0-9
        if let Some(idx) = index {
            if idx < 10 {
                let number_label = Label::builder()
                    .label(idx.to_string())
                    .css_classes(vec!["ResultNumber"])
                    .valign(Align::Center)
                    .build();
                button_content.append(&number_label);
            }
        }

        let button = Button::builder()
            .child(&button_content)
            .tooltip_text(&file_result.path)
            .hexpand(true)
            .height_request(16)
            .build();

        let file_path = file_result.path.clone();
        let file_type = file_result.file_type.clone();
        let window_clone = window.clone();
        let recent_usage_clone = recent_usage.clone();

        button.connect_clicked(move |_| {
            // Open the file/directory using default application
            let path_to_open = match file_type {
                FileType::Directory => file_path.clone(),
                FileType::File => file_path.clone(),
            };

            if let Err(e) = Self::open_path(&path_to_open) {
                eprintln!("Failed to open {}: {}", path_to_open, e);
            } else {
                // Track recent usage
                recent_usage_clone
                    .borrow_mut()
                    .add_recent_file(&path_to_open);
            }
            window_clone.set_visible(false);
        });

        // Create folder open button
        let folder_icon = Image::builder()
            .icon_name("folder-open-symbolic")
            .pixel_size(16)
            .build();

        let folder_button = Button::builder()
            .child(&folder_icon)
            .tooltip_text("Open containing folder")
            .valign(Align::Center)
            .height_request(16)
            .build();

        let file_path_for_folder = file_result.path.clone();
        let file_type_for_folder = file_result.file_type.clone();
        let window_clone_for_folder = window.clone();
        folder_button.connect_clicked(move |_| {
            // Get parent directory
            let path_obj = std::path::Path::new(&file_path_for_folder);
            let dir_to_open = match file_type_for_folder {
                FileType::Directory => file_path_for_folder.clone(),
                FileType::File => path_obj
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| file_path_for_folder.clone()),
            };

            if let Err(e) = Self::open_path(&dir_to_open) {
                eprintln!("Failed to open directory {}: {}", dir_to_open, e);
            }
            window_clone_for_folder.set_visible(false);
        });

        outer_box.append(&button);
        outer_box.append(&folder_button);

        (outer_box, button)
    }

    fn open_path(path: &str) -> Result<(), std::io::Error> {
        // Use xdg-open to open files/directories with default applications
        std::process::Command::new("xdg-open").arg(path).spawn()?;
        Ok(())
    }

    fn open_url(url: &str, _window: &ApplicationWindow) -> Result<(), std::io::Error> {
        // Use xdg-open to open URLs with default browser
        std::process::Command::new("xdg-open").arg(url).spawn()?;
        Ok(())
    }

    fn update_results(&self, query: &str) {
        Self::update_search_results(
            &self.results_box,
            &self.app_search,
            &self.file_searcher,
            &self.gemini,
            &self.brave_search,
            &self.recent_usage,
            &self.favorites,
            query,
            &self.window,
            &self.last_ai_query,
            &self.result_buttons,
        );
    }

    fn show(&self) {
        self.window.present();
        self.entry.grab_focus();
    }

    #[allow(dead_code)]
    fn close(&self) {
        self.window.close();
    }
}

#[tokio::main]
async fn main() {
    // Create application with unique ID for single instance behavior
    // The REPLACE flag allows the new instance to replace an existing one if needed
    let app = gtk4::Application::builder()
        .application_id("com.github.cmihail.universal-search")
        .flags(gio::ApplicationFlags::REPLACE)
        .build();

    app.connect_startup(|_| {
        // Load CSS from embedded content
        let provider = CssProvider::new();
        let css_data = include_str!("style.css");
        provider.load_from_data(css_data);

        gtk4::style_context_add_provider_for_display(
            &gdk4::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(|app| {
        // Single instance behavior: check if there's already an active window
        if let Some(window) = app.active_window() {
            // Toggle visibility: hide if visible, show if hidden
            if window.is_visible() {
                window.set_visible(false);
            } else {
                window.present();
            }
        } else {
            // No existing window, create a new one
            let search_window = SearchWindow::new(app);
            search_window.show();
        }

        // Set up global shortcut handler (if needed)
        // This would typically be handled by the window manager or desktop environment
    });

    app.run();
}
