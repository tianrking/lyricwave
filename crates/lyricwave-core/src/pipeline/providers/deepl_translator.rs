use serde::Deserialize;

use crate::pipeline::Translator;

#[derive(Clone)]
pub struct DeepLTranslatorProvider {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
struct DeepLTranslationItem {
    text: String,
}

#[derive(Debug, Deserialize)]
struct DeepLTranslateResponse {
    translations: Vec<DeepLTranslationItem>,
}

impl Translator for DeepLTranslatorProvider {
    fn name(&self) -> &'static str {
        "deepl"
    }

    fn translate(&self, input: &str, target_lang: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::new();
        let endpoint = format!("{}/v2/translate", self.base_url.trim_end_matches('/'));

        let response = client
            .post(endpoint)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .form(&[("text", input), ("target_lang", target_lang)])
            .send()
            .map_err(|err| format!("DeepL request failed: {err}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(format!("DeepL returned {}: {}", status, body));
        }

        let payload: DeepLTranslateResponse = response
            .json()
            .map_err(|err| format!("failed to parse DeepL response: {err}"))?;

        payload
            .translations
            .first()
            .map(|t| t.text.clone())
            .ok_or_else(|| "DeepL response missing translations".to_string())
    }
}
