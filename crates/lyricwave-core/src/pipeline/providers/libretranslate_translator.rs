use serde::{Deserialize, Serialize};

use crate::pipeline::Translator;

#[derive(Clone)]
pub struct LibreTranslateProvider {
    pub base_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct LibreTranslateRequest<'a> {
    q: &'a str,
    source: &'a str,
    target: &'a str,
    format: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct LibreTranslateResponse {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

impl Translator for LibreTranslateProvider {
    fn name(&self) -> &'static str {
        "libretranslate"
    }

    fn translate(&self, input: &str, target_lang: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::new();
        let endpoint = format!("{}/translate", self.base_url.trim_end_matches('/'));

        let request = LibreTranslateRequest {
            q: input,
            source: "auto",
            target: target_lang,
            format: "text",
            api_key: self.api_key.as_deref(),
        };

        let response = client
            .post(endpoint)
            .json(&request)
            .send()
            .map_err(|err| format!("LibreTranslate request failed: {err}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(format!("LibreTranslate returned {}: {}", status, body));
        }

        let payload: LibreTranslateResponse = response
            .json()
            .map_err(|err| format!("failed to parse LibreTranslate response: {err}"))?;

        Ok(payload.translated_text)
    }
}
