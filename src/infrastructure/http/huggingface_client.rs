const HF_API_BASE: &str = "https://huggingface.co/api/models";

pub struct HuggingFaceClient;

impl HuggingFaceClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn search_models(
        &self,
        pipeline_tag: &str,
        query: &str,
        language: &str,
        limit: usize,
    ) -> Result<Vec<serde_json::Value>, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let limit_str = limit.to_string();
        let mut params: Vec<(&str, &str)> = Vec::new();
        params.push(("sort", "downloads"));
        params.push(("direction", "-1"));
        params.push(("limit", &limit_str));
        if !query.is_empty() { params.push(("search", query)); }
        if !pipeline_tag.is_empty() { params.push(("pipeline_tag", pipeline_tag)); }

        let resp = client
            .get(HF_API_BASE)
            .query(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status() != 200 {
            return Ok(vec![]);
        }

        let data: Vec<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;
        let lang_code = lang_to_code(language).to_lowercase();

        let results: Vec<serde_json::Value> = data.into_iter()
            .filter(|item| {
                if lang_code.is_empty() || lang_code == "en" { return true; }
                let tags = item.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|t| t.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default();
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                tags.iter().any(|t| t.to_lowercase() == lang_code) || id.contains(&lang_code)
            })
            .take(limit)
            .map(|item| {
                serde_json::json!({
                    "id": item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    "pipeline": item.get("pipeline_tag").and_then(|v| v.as_str()).unwrap_or(""),
                    "downloads": item.get("downloads").and_then(|v| v.as_i64()).unwrap_or(0),
                    "likes": item.get("likes").and_then(|v| v.as_i64()).unwrap_or(0),
                })
            })
            .collect();

        Ok(results)
    }
}

const LANG_MAP: &[(&str, &str)] = &[
    ("en-US", "en"), ("en-GB", "en"), ("es-ES", "es"), ("es-MX", "es"),
    ("zh-CN", "zh"), ("zh-TW", "zh"), ("fr-FR", "fr"), ("de-DE", "de"),
    ("ja-JP", "ja"), ("ko-KR", "ko"), ("pt-BR", "pt"), ("ru-RU", "ru"),
    ("ar-SA", "ar"), ("it-IT", "it"), ("nl-NL", "nl"), ("pl-PL", "pl"),
    ("sv-SE", "sv"), ("tr-TR", "tr"), ("vi-VN", "vi"), ("th-TH", "th"),
    ("id-ID", "id"), ("ms-MY", "ms"), ("ro-RO", "ro"), ("uk-UA", "uk"),
    ("ca-ES", "ca"), ("fi-FI", "fi"), ("cs-CZ", "cs"), ("el-GR", "el"),
    ("he-IL", "he"), ("hi-IN", "hi"), ("hu-HU", "hu"), ("no-NO", "no"),
    ("sk-SK", "sk"), ("da-DK", "da"), ("bg-BG", "bg"), ("hr-HR", "hr"),
    ("lt-LT", "lt"), ("sl-SI", "sl"), ("et-EE", "et"), ("lv-LV", "lv"),
    ("sr-RS", "sr"), ("mk-MK", "mk"), ("sq-AL", "sq"),
];

fn lang_to_code(language: &str) -> &str {
    for (lang, code) in LANG_MAP {
        if *lang == language { return code; }
    }
    "en"
}
