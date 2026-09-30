use gio::prelude::*;
use glib::subclass::prelude::*;
use glib::Properties;
use gtk4::glib;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;

const API_BASE_URL: &str = "https://api.search.brave.com/res/v1/web/search";
const MAX_RESULTS: u32 = 10;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BraveSearchProfile {
    pub name: String,
    pub url: String,
    pub long_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BraveSearchThumbnail {
    pub src: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BraveSearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
    pub age: Option<String>,
    pub page_age: Option<String>,
    pub profile: Option<BraveSearchProfile>,
    pub thumbnail: Option<BraveSearchThumbnail>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveSearchResult>,
    #[serde(rename = "totalCount")]
    #[allow(dead_code)]
    total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BraveQuery {
    original: String,
    show_strict_warning: bool,
    is_navigational: bool,
    local_decision: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: Option<BraveWebResults>,
    #[allow(dead_code)]
    query: Option<BraveQuery>,
}

fn get_api_key() -> Option<String> {
    env::var("BRAVE_SEARCH_API_KEY").ok()
}

fn validate_api_key(api_key: Option<String>) -> Result<String, String> {
    api_key.ok_or_else(|| "Error: BRAVE_SEARCH_API_KEY environment variable not set".to_string())
}

fn decode_html_entities(text: &str) -> String {
    let entity = regex::Regex::new(r"&(#[0-9]+|#[xX][0-9a-fA-F]+|amp|lt|gt|quot|apos);").unwrap();
    entity
        .replace_all(text, |caps: &regex::Captures| {
            let name = &caps[1];
            let decoded = match name {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ if name.starts_with("#x") || name.starts_with("#X") => {
                    u32::from_str_radix(&name[2..], 16)
                        .ok()
                        .and_then(char::from_u32)
                }
                _ => name[1..].parse().ok().and_then(char::from_u32),
            };
            decoded.map_or_else(|| caps[0].to_string(), |c| c.to_string())
        })
        .into_owned()
}

// Snippets come from third-party pages, so everything except <strong> must be escaped.
fn convert_html_to_pango_markup(text: &str) -> String {
    glib::markup_escape_text(&decode_html_entities(text))
        .replace("&lt;strong&gt;", "<span foreground=\"#ff8cc8\">")
        .replace("&lt;/strong&gt;", "</span>")
}

fn parse_brave_search_response(response_text: &str) -> Result<Vec<BraveSearchResult>, String> {
    let response: BraveSearchResponse = serde_json::from_str(response_text)
        .map_err(|e| format!("Failed to parse Brave Search response: {}", e))?;

    let results = response.web.map(|web| web.results).unwrap_or_default();

    // Convert HTML tags to Pango markup
    let converted_results = results
        .into_iter()
        .map(|mut result| {
            result.description = convert_html_to_pango_markup(&result.description);
            result
        })
        .collect();

    Ok(converted_results)
}

pub struct BraveSearchService {
    client: Client,
    history: HashMap<String, Vec<BraveSearchResult>>,
}

impl Default for BraveSearchService {
    fn default() -> Self {
        Self::new()
    }
}

impl BraveSearchService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            history: HashMap::new(),
        }
    }

    pub async fn search(&mut self, query: &str) -> Result<Vec<BraveSearchResult>, String> {
        // Check cache first - if we have this exact query, return cached results immediately
        // This avoids API calls and provides instant results for repeated queries
        if let Some(cached_results) = self.history.get(query) {
            return Ok(cached_results.clone());
        }

        let api_key = validate_api_key(get_api_key())?;

        let url = format!(
            "{}?count={}&q={}",
            API_BASE_URL,
            MAX_RESULTS,
            urlencoding::encode(query)
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("X-Subscription-Token", &api_key)
            .send()
            .await
            .map_err(|e| format!("Brave Search request failed: {}", e))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        if !status.is_success() {
            let error_msg = format!(
                "Brave Search API Error ({}): {}",
                status,
                status.canonical_reason().unwrap_or("Unknown")
            );
            return Err(error_msg);
        }

        // Check if response looks like JSON
        if response_text.trim().is_empty() {
            let error_msg = "Brave Search API returned empty response".to_string();
            return Err(error_msg);
        }

        if !response_text.trim_start().starts_with('{')
            && !response_text.trim_start().starts_with('[')
        {
            let safe_response_preview = if response_text.len() > 200 {
                response_text.chars().take(200).collect::<String>()
            } else {
                response_text.clone()
            };
            let error_msg = format!("Brave Search API returned non-JSON response. This usually indicates an invalid API key or API endpoint issue. Response: {}", safe_response_preview);
            return Err(error_msg);
        }

        match parse_brave_search_response(&response_text) {
            Ok(results) => {
                // Cache the results
                self.history.insert(query.to_string(), results.clone());
                if self.history.len() > 10 {
                    let oldest_key = self.history.keys().next().cloned();
                    if let Some(key) = oldest_key {
                        self.history.remove(&key);
                    }
                }

                Ok(results)
            }
            Err(error) => {
                let enhanced_error = format!("Brave Search parsing error: {}. This usually indicates an invalid API key or malformed response.", error);
                Err(enhanced_error)
            }
        }
    }

    pub fn reset(&mut self) {
        // No need to clear history for now, just reset state
    }
}

mod imp {
    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::BraveSearch)]
    pub struct BraveSearch {
        #[property(get, set)]
        loading: RefCell<bool>,
        results: RefCell<Vec<BraveSearchResult>>,
        service: RefCell<Option<BraveSearchService>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BraveSearch {
        const NAME: &'static str = "UniversalSearchBraveSearch";
        type Type = super::BraveSearch;
    }

    #[glib::derived_properties]
    impl ObjectImpl for BraveSearch {}

    impl BraveSearch {
        pub fn results(&self) -> Vec<super::BraveSearchResult> {
            self.results.borrow().clone()
        }

        pub fn set_results(&self, results: Vec<super::BraveSearchResult>) {
            self.results.replace(results);
        }

        pub fn take_service(&self) -> Option<BraveSearchService> {
            self.service.borrow_mut().take()
        }

        pub fn set_service(&self, service: BraveSearchService) {
            *self.service.borrow_mut() = Some(service);
        }

        pub fn ensure_service(&self) {
            if self.service.borrow().is_none() {
                *self.service.borrow_mut() = Some(BraveSearchService::new());
            }
        }
    }
}

glib::wrapper! {
    pub struct BraveSearch(ObjectSubclass<imp::BraveSearch>);
}

impl BraveSearch {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn results(&self) -> Vec<BraveSearchResult> {
        self.imp().results()
    }

    pub fn set_results(&self, results: Vec<BraveSearchResult>) {
        self.imp().set_results(results);
        // Note: "results" is not a GObject property, so we don't emit notify signal
    }

    pub fn search(&self, query: String) {
        if query.trim().is_empty() {
            return;
        }

        self.set_loading(true);
        self.set_results(Vec::new());

        // Ensure service exists and take it out
        self.imp().ensure_service();
        let mut service = self.imp().take_service().unwrap_or_default();

        let self_clone = self.clone();

        // Use glib::spawn_future_local to run async code on main thread
        glib::spawn_future_local(async move {
            let result = service.search(&query).await;

            // Put the service back
            self_clone.imp().set_service(service);

            match result {
                Ok(results) => {
                    self_clone.set_results(results);
                }
                Err(_error) => {
                    self_clone.set_results(Vec::new());
                }
            }
            self_clone.set_loading(false);
        });
    }

    pub fn reset(&self) {
        self.set_loading(false);
        self.set_results(Vec::new());
    }
}

impl Default for BraveSearch {
    fn default() -> Self {
        Self::new()
    }
}
