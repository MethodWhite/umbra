use axum::Json;
use serde::Deserialize;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
];

fn random_user_agent() -> &'static str {
    use rand::Rng;
    USER_AGENTS[rand::thread_rng().gen_range(0..USER_AGENTS.len())]
}

#[derive(Deserialize)]
pub struct SearchAndTrainBody {
    pub query: String,
    pub max_pages: Option<usize>,
}

#[derive(Deserialize)]
pub struct VisitBody {
    pub url: String,
    pub mode: Option<String>,
    pub extract: Option<bool>,
}

#[derive(Deserialize)]
pub struct CollectBody {
    pub targets: Option<Vec<String>>,
    pub extract_tables_and_lists: Option<bool>,
}

#[derive(Deserialize)]
pub struct SettingsBody {
    pub human_mode: Option<bool>,
}

async fn fetch_url(url: &str, extract: bool) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(url)
        .header("User-Agent", random_user_agent())
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    let headers = resp.headers().clone();
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/html")
        .to_string();
    let body = resp.text().await.map_err(|e| format!("Read failed: {}", e))?;

    let title = extract_title(&body);
    let text_content = if extract {
        extract_text(&body, &content_type)
    } else {
        String::new()
    };
    let word_count = text_content.split_whitespace().count();

    Ok(serde_json::json!({
        "url": url,
        "title": title,
        "status": status.as_u16(),
        "content_type": content_type,
        "text_content": text_content,
        "word_count": word_count,
        "body_preview": body.chars().take(500).collect::<String>(),
    }))
}

fn extract_title(html: &str) -> String {
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html[start + 7..].find("</title>") {
            return html[start + 7..start + 7 + end].to_string();
        }
    }
    String::new()
}

fn extract_text(html: &str, _content_type: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        if in_script {
            if c == '<' {
                let peek: String = chars.clone().take(7).collect();
                if peek.to_lowercase().starts_with("/script") {
                    in_script = false;
                }
            }
            continue;
        }
        if in_style {
            if c == '<' {
                let peek: String = chars.clone().take(6).collect();
                if peek.to_lowercase().starts_with("/style") {
                    in_style = false;
                }
            }
            continue;
        }
        if c == '<' {
            in_tag = true;
            let peek: String = chars.clone().take(6).collect();
            if peek.to_lowercase().starts_with("script") { in_script = true; }
            if peek.to_lowercase().starts_with("style") { in_style = true; }
            continue;
        }
        if c == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag {
            result.push(c);
        }
    }

    let mut cleaned = String::new();
    let mut prev_space = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !prev_space {
                cleaned.push(' ');
                prev_space = true;
            }
        } else {
            cleaned.push(c);
            prev_space = false;
        }
    }

    cleaned.trim().to_string()
}

pub async fn search_and_train(
    Json(body): Json<SearchAndTrainBody>,
) -> Json<serde_json::Value> {
    let query = &body.query;
    let search_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding(query));

    match fetch_url(&search_url, true).await {
        Ok(result) => {
            Json(serde_json::json!({
                "status": "ok",
                "result": {
                    "query": query,
                    "pages_fetched": 1,
                    "snippet": result.get("text_content").and_then(|v| v.as_str()).unwrap_or("").chars().take(2000).collect::<String>(),
                },
            }))
        }
        Err(e) => Json(serde_json::json!({"status": "error", "error": e})),
    }
}

pub async fn visit(
    Json(body): Json<VisitBody>,
) -> Json<serde_json::Value> {
    let extract = body.extract.unwrap_or(false);
    match fetch_url(&body.url, extract).await {
        Ok(content) => {
            Json(serde_json::json!({
                "status": "ok",
                "content": content,
            }))
        }
        Err(e) => Json(serde_json::json!({"status": "error", "error": e})),
    }
}

pub async fn status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "human_mode": false,
        "buffer_size": 0,
        "estimated_tokens": 0,
        "browser_active": false,
    }))
}

pub async fn collect(
    Json(_body): Json<CollectBody>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "result": {
            "collected": 0,
            "message": "Browser endpoints use HTTP fetch (no Playwright). For full browser automation, run the Python server alongside.",
        },
    }))
}

pub async fn settings() -> Json<serde_json::Value> {
    Json(serde_json::json!({"human_mode": false}))
}

pub async fn settings_update(
    Json(body): Json<SettingsBody>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "human_mode": body.human_mode.unwrap_or(false),
    }))
}

fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
