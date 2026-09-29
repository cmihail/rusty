use gio::prelude::*;
use glib::subclass::prelude::*;
use glib::Properties;
use gtk4::glib;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;

const MODEL: &str = "gemini-flash-lite-latest";
const API_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Debug, Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
struct SystemInstruction {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct RequestBody {
    contents: Vec<Content>,
    #[serde(rename = "systemInstruction")]
    system_instruction: SystemInstruction,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<ContentResponse>,
}

#[derive(Debug, Deserialize)]
struct ContentResponse {
    parts: Option<Vec<PartResponse>>,
}

#[derive(Debug, Deserialize)]
struct PartResponse {
    text: Option<String>,
}

fn get_api_key() -> Option<String> {
    env::var("PERSONAL_GEMINI_API_KEY").ok()
}

fn validate_api_key(api_key: Option<String>) -> Result<String, String> {
    api_key.ok_or_else(|| "Error: PERSONAL_GEMINI_API_KEY environment variable not set".to_string())
}

fn build_request_body(prompt: &str, history: &HashMap<String, String>) -> RequestBody {
    let mut contents = Vec::new();

    for (user_prompt, model_response) in history {
        contents.push(Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: user_prompt.clone(),
            }],
        });
        contents.push(Content {
            role: "model".to_string(),
            parts: vec![Part {
                text: model_response.clone(),
            }],
        });
    }

    contents.push(Content {
        role: "user".to_string(),
        parts: vec![Part {
            text: prompt.to_string(),
        }],
    });

    let system_text = "You are a helpful assistant that MUST return responses as valid Pango markup text.\nPango markup is used by GTK4 for rich text formatting.\n\nIMPORTANT RULES:\n1. ONLY use these Pango markup tags: <b>, <i>, <u>, <s>, <sub>, <sup>, <small>, <big>, <tt>, <span>\n2. For <span> tags, only use these attributes: foreground, background, font_family, font_size, font_weight, font_style, underline, strikethrough\n3. NEVER use HTML tags like <h1>, <h2>, <p>, <div>, <br>, <ul>, <li>, <code>, etc\n4. NEVER use markdown syntax like # ** __, etc.\n5. Always close all opening tags properly\n6. Use &amp; for &, &lt; for <, &gt; for >, &quot; for \", and avoid complex entities\n7. For line breaks, use actual newlines, not <br> tags\n8. Default text should not be colored. Use colors only for highlighting specific parts of the text.\n9. Use color highlighting to emphasize important content with colors that work well on dark themes\n\nYour response must be valid Pango markup that can be directly used in GTK4 labels.";

    RequestBody {
        contents,
        system_instruction: SystemInstruction {
            parts: vec![Part {
                text: system_text.to_string(),
            }],
        },
    }
}

fn parse_gemini_response(response: &GeminiResponse) -> Result<String, String> {
    if let Some(candidates) = &response.candidates {
        if let Some(candidate) = candidates.first() {
            if let Some(content) = &candidate.content {
                if let Some(parts) = &content.parts {
                    if let Some(part) = parts.first() {
                        if let Some(text) = &part.text {
                            return Ok(text.clone());
                        }
                    }
                }
            }
        }
    }
    Err("Error: No valid content in API response".to_string())
}

fn escape_markup(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn is_valid_pango_markup(markup: &str) -> bool {
    // Simple validation - just check that basic tags are balanced
    let open_tags = markup.matches('<').count();
    let close_tags = markup.matches('>').count();
    open_tags == close_tags && !markup.contains("&lt;script") && !markup.contains("&lt;style")
}

fn get_valid_pango_portion(markup: &str) -> String {
    let converted = markup
        .replace("<h1>", "<b>")
        .replace("</h1>", "</b>")
        .replace("<h2>", "<b>")
        .replace("</h2>", "</b>")
        .replace("<h3>", "<b>")
        .replace("</h3>", "</b>")
        .replace("<h4>", "<b>")
        .replace("</h4>", "</b>")
        .replace("<h5>", "<b>")
        .replace("</h5>", "</b>")
        .replace("<h6>", "<b>")
        .replace("</h6>", "</b>")
        .replace("<code>", "<tt>")
        .replace("</code>", "</tt>")
        .replace("<strong>", "<b>")
        .replace("</strong>", "</b>")
        .replace("<em>", "<i>")
        .replace("</em>", "</i>")
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<li>", "• ")
        .replace("</li>", "\n")
        .replace("<ul>", "")
        .replace("</ul>", "")
        .replace("<ol>", "")
        .replace("</ol>", "")
        .replace("<p>", "")
        .replace("</p>", "\n")
        .replace("<div>", "")
        .replace("</div>", "")
        .replace("&times;", "×")
        .replace("&nbsp;", " ")
        .replace("&ndash;", "–")
        .replace("&mdash;", "—")
        .replace("&lsquo;", "'")
        .replace("&rsquo;", "'")
        .replace("&ldquo;", "\"")
        .replace("&rdquo;", "\"")
        .replace("&hellip;", "…");

    if is_valid_pango_markup(&converted) {
        converted
    } else {
        escape_markup(markup)
    }
}

pub struct GeminiService {
    client: Client,
    history: HashMap<String, String>,
}

impl Default for GeminiService {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            history: HashMap::new(),
        }
    }

    pub async fn search(&mut self, prompt: &str) -> Result<String, String> {
        // Check history first - if we have this exact prompt, return cached response immediately
        // This avoids API calls and provides instant results for repeated queries
        if let Some(cached_response) = self.history.get(prompt) {
            return Ok(get_valid_pango_portion(cached_response));
        }

        let api_key = validate_api_key(get_api_key())?;
        let request_body = build_request_body(prompt, &self.history);

        let url = format!("{}/{}:generateContent?key={}", API_BASE_URL, MODEL, api_key);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let error_msg = format!(
                "API Error ({}): {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            );
            return Err(error_msg);
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        match parse_gemini_response(&gemini_response) {
            Ok(text) => {
                let valid_markup = get_valid_pango_portion(&text);

                if !text.starts_with("Error:") {
                    // Store in history for both conversation context and caching
                    self.history.insert(prompt.to_string(), text.clone());
                    if self.history.len() > 5 {
                        let oldest_key = self.history.keys().next().cloned();
                        if let Some(key) = oldest_key {
                            self.history.remove(&key);
                        }
                    }
                }

                Ok(valid_markup)
            }
            Err(error) => Err(error),
        }
    }

    pub fn reset(&mut self) {
        // No need to clear history for now, just reset state
    }
}

mod imp {
    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::Gemini)]
    pub struct Gemini {
        #[property(get, set)]
        loading: RefCell<bool>,
        #[property(get, set)]
        result: RefCell<String>,
        service: RefCell<Option<GeminiService>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Gemini {
        const NAME: &'static str = "UniversalSearchGemini";
        type Type = super::Gemini;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Gemini {}

    impl Gemini {
        pub fn take_service(&self) -> Option<GeminiService> {
            self.service.borrow_mut().take()
        }

        pub fn set_service(&self, service: GeminiService) {
            *self.service.borrow_mut() = Some(service);
        }

        pub fn ensure_service(&self) {
            if self.service.borrow().is_none() {
                *self.service.borrow_mut() = Some(GeminiService::new());
            }
        }
    }
}

glib::wrapper! {
    pub struct Gemini(ObjectSubclass<imp::Gemini>);
}

impl Gemini {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn search(&self, prompt: String) {
        self.set_loading(true);
        self.set_result("Thinking...".to_string());

        // Ensure service exists and take it out
        self.imp().ensure_service();
        let mut service = self.imp().take_service().unwrap_or_default();

        let self_clone = self.clone();

        // Use glib::spawn_future_local to run async code on main thread
        glib::spawn_future_local(async move {
            let result = service.search(&prompt).await;

            // Put the service back
            self_clone.imp().set_service(service);

            match result {
                Ok(response) => {
                    self_clone.set_result(response);
                }
                Err(error) => {
                    self_clone.set_result(error);
                }
            }
            self_clone.set_loading(false);
        });
    }

    pub fn reset(&self) {
        self.set_loading(false);
        self.set_result(String::new());
    }
}

impl Default for Gemini {
    fn default() -> Self {
        Self::new()
    }
}
