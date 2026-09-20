//! Cloudflare Workers translation API (translate-api compatible).
//!
//! Protocol (GET query):
//!   text, source_language, target_language, secret
//! Response JSON:
//!   { "code": 0, "msg": "ok", "text": "..." }
//!
//! See https://github.com/jianchang512/translate-api

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::config::AppConfig;
use crate::lang::normalize_lang_code;

use super::{TranslationRequest, TranslationResult, Translator};

pub struct CloudflareTranslator {
    client: Client,
    endpoint: String,
    secret: Option<String>,
}

impl CloudflareTranslator {
    pub fn from_config(config: &AppConfig, client: Client) -> Result<Self, String> {
        let endpoint = config
            .engine
            .cloudflare_endpoint
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "Cloudflare 翻译未配置 Worker 地址（endpoint）".to_string())?
            .to_string();

        if !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
            return Err("Cloudflare 翻译地址须以 http:// 或 https:// 开头".into());
        }

        let secret = config
            .engine
            .cloudflare_secret
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        Ok(Self {
            client,
            endpoint,
            secret,
        })
    }
}

#[async_trait]
impl Translator for CloudflareTranslator {
    fn id(&self) -> &'static str {
        "cloudflare"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        if req.text.trim().is_empty() {
            return Err("翻译文本为空".into());
        }
        if req.target_lang.trim().is_empty() {
            return Err("目标语言不能为空".into());
        }

        let target = to_m2m_lang(&req.target_lang)?;
        let (source, detected) = resolve_m2m_source(&req.source_lang, req.text.trim());

        let mut url = reqwest::Url::parse(&self.endpoint)
            .map_err(|e| format!("Cloudflare 翻译地址无效: {e}"))?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("text", req.text.trim());
            pairs.append_pair("source_language", &source);
            pairs.append_pair("target_language", &target);
            if let Some(secret) = &self.secret {
                pairs.append_pair("secret", secret);
            }
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Cloudflare 翻译请求失败: {e}"))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取 Cloudflare 翻译响应失败: {e}"))?;

        if !status.is_success() {
            return Err(format!(
                "Cloudflare 翻译 HTTP 错误 ({status})：{}",
                truncate(&raw, 200)
            ));
        }

        let body: ApiResponse = serde_json::from_str(&raw).map_err(|e| {
            format!(
                "解析 Cloudflare 翻译响应失败: {e}; body={}",
                truncate(&raw, 200)
            )
        })?;

        if body.code.unwrap_or(-1) != 0 {
            let msg = body.msg.unwrap_or_else(|| "未知错误".into());
            return Err(format!(
                "Cloudflare 翻译失败（code={}）: {msg}",
                body.code.unwrap_or(-1)
            ));
        }

        let text = body.text.unwrap_or_default().trim().to_string();
        if text.is_empty() {
            return Err("Cloudflare 翻译返回空译文".into());
        }
        if text.starts_with("ERROR") {
            return Err(format!("Cloudflare 翻译模型错误: {text}"));
        }

        Ok(TranslationResult {
            engine: self.id().into(),
            text,
            source_lang: req.source_lang.clone(),
            target_lang: normalize_lang_code(&req.target_lang),
            detected_source_lang: detected,
        })
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    code: Option<i32>,
    msg: Option<String>,
    text: Option<String>,
}

/// Map app language codes to m2m100-style 2-letter codes used by translate-api.
fn to_m2m_lang(code: &str) -> Result<String, String> {
    let normalized = normalize_lang_code(code);
    if normalized.is_empty() {
        return Err("目标语言无效".into());
    }
    let mapped = match normalized.as_str() {
        "zh-CN" | "zh-TW" | "zh" => "zh".to_string(),
        other => {
            let two: String = other.chars().take(2).collect();
            if two.len() < 2 {
                return Err(format!("不支持的语言代码: {other}"));
            }
            two.to_ascii_lowercase()
        }
    };
    Ok(mapped)
}

/// Resolve source language for m2m100. Worker always needs an explicit code;
/// when the app says `auto`, guess from script (CJK / kana / hangul / latin).
fn resolve_m2m_source(code: &str, text: &str) -> (String, Option<String>) {
    let normalized = normalize_lang_code(code);
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("auto") {
        let guessed = guess_source_lang(text).to_string();
        return (guessed.clone(), Some(guessed));
    }
    let mapped = to_m2m_lang(&normalized).unwrap_or_else(|_| guess_source_lang(text).into());
    (mapped, None)
}

/// Lightweight script heuristic — enough for 中↔英 / 日 / 韩划词场景.
fn guess_source_lang(text: &str) -> &'static str {
    let mut cjk = 0usize;
    let mut kana = 0usize;
    let mut hangul = 0usize;
    let mut latin = 0usize;

    for ch in text.chars() {
        match ch {
            '\u{3040}'..='\u{30FF}' | '\u{31F0}'..='\u{31FF}' => kana += 1,
            '\u{AC00}'..='\u{D7AF}' | '\u{1100}'..='\u{11FF}' => hangul += 1,
            '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}' => {
                cjk += 1
            }
            'A'..='Z' | 'a'..='z' => latin += 1,
            _ => {}
        }
    }

    // Kana / Hangul beat Han: Japanese often mixes kanji with kana.
    if kana > 0 {
        return "ja";
    }
    if hangul > 0 {
        return "ko";
    }
    if cjk > 0 && cjk >= latin {
        return "zh";
    }
    "en"
}

fn truncate(s: &str, max: usize) -> String {
    let mut t: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        t.push('…');
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_chinese_variants() {
        assert_eq!(to_m2m_lang("zh-CN").unwrap(), "zh");
        assert_eq!(to_m2m_lang("zh-TW").unwrap(), "zh");
        assert_eq!(to_m2m_lang("en").unwrap(), "en");
        assert_eq!(to_m2m_lang("ja").unwrap(), "ja");
    }

    #[test]
    fn auto_source_guesses_from_script() {
        let (src, detected) = resolve_m2m_source("auto", "你好世界");
        assert_eq!(src, "zh");
        assert_eq!(detected.as_deref(), Some("zh"));

        let (src, _) = resolve_m2m_source("auto", "Hello world");
        assert_eq!(src, "en");

        let (src, _) = resolve_m2m_source("auto", "こんにちは");
        assert_eq!(src, "ja");

        let (src, _) = resolve_m2m_source("auto", "안녕하세요");
        assert_eq!(src, "ko");
    }

    #[test]
    fn explicit_source_is_kept() {
        let (src, detected) = resolve_m2m_source("zh-CN", "Hello");
        assert_eq!(src, "zh");
        assert!(detected.is_none());
    }
}
