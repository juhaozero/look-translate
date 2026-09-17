//! Self-hosted translation API (Cloudflare Workers / translate-api compatible).
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

pub struct SelfHostedTranslator {
    client: Client,
    endpoint: String,
    secret: Option<String>,
}

impl SelfHostedTranslator {
    pub fn from_config(config: &AppConfig, client: Client) -> Result<Self, String> {
        let endpoint = config
            .engine
            .self_hosted_endpoint
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "自建翻译未配置接口地址（endpoint）".to_string())?
            .to_string();

        if !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
            return Err("自建翻译地址须以 http:// 或 https:// 开头".into());
        }

        let secret = config
            .engine
            .self_hosted_secret
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
impl Translator for SelfHostedTranslator {
    fn id(&self) -> &'static str {
        "self_hosted"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        if req.text.trim().is_empty() {
            return Err("翻译文本为空".into());
        }
        if req.target_lang.trim().is_empty() {
            return Err("目标语言不能为空".into());
        }

        let target = to_m2m_lang(&req.target_lang)?;
        let source = to_m2m_source(&req.source_lang);

        let mut url = reqwest::Url::parse(&self.endpoint)
            .map_err(|e| format!("自建翻译地址无效: {e}"))?;
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
            .map_err(|e| format!("自建翻译请求失败: {e}"))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取自建翻译响应失败: {e}"))?;

        if !status.is_success() {
            return Err(format!(
                "自建翻译 HTTP 错误 ({status})：{}",
                truncate(&raw, 200)
            ));
        }

        let body: ApiResponse = serde_json::from_str(&raw).map_err(|e| {
            format!("解析自建翻译响应失败: {e}; body={}", truncate(&raw, 200))
        })?;

        if body.code.unwrap_or(-1) != 0 {
            let msg = body.msg.unwrap_or_else(|| "未知错误".into());
            return Err(format!("自建翻译失败（code={}）: {msg}", body.code.unwrap_or(-1)));
        }

        let text = body
            .text
            .unwrap_or_default()
            .trim()
            .to_string();
        if text.is_empty() {
            return Err("自建翻译返回空译文".into());
        }
        if text.starts_with("ERROR") {
            return Err(format!("自建翻译模型错误: {text}"));
        }

        Ok(TranslationResult {
            engine: self.id().into(),
            text,
            source_lang: req.source_lang.clone(),
            target_lang: normalize_lang_code(&req.target_lang),
            detected_source_lang: None,
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

fn to_m2m_source(code: &str) -> String {
    let normalized = normalize_lang_code(code);
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("auto") {
        // Worker requires a source_lang; default to English for auto.
        return "en".into();
    }
    to_m2m_lang(&normalized).unwrap_or_else(|_| "en".into())
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
    fn auto_source_defaults_to_en() {
        assert_eq!(to_m2m_source("auto"), "en");
        assert_eq!(to_m2m_source(""), "en");
    }
}
