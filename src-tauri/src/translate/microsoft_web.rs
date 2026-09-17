//! Unofficial Bing / Microsoft web translator (no API key).

use async_trait::async_trait;
use reqwest::header::{CONTENT_TYPE, COOKIE, SET_COOKIE, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;

use crate::lang::{from_microsoft_lang, to_microsoft_lang};

use super::{TranslationRequest, TranslationResult, Translator};

const TRANSLATOR_PAGE: &str = "https://www.bing.com/translator";
const TRANSLATE_ENDPOINT: &str = "https://www.bing.com/ttranslatev3";
const PAGE_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

pub struct MicrosoftWebTranslator {
    client: Client,
}

impl MicrosoftWebTranslator {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Translator for MicrosoftWebTranslator {
    fn id(&self) -> &'static str {
        "microsoft_web"
    }

    async fn translate(&self, req: &TranslationRequest) -> Result<TranslationResult, String> {
        if req.text.trim().is_empty() {
            return Err("翻译文本为空".into());
        }
        if req.target_lang.trim().is_empty() {
            return Err("目标语言不能为空".into());
        }

        let session = fetch_bing_session(&self.client).await?;
        let target = to_microsoft_lang(&req.target_lang);
        let source = {
            let mapped = to_microsoft_lang(&req.source_lang);
            if mapped.is_empty() || mapped.eq_ignore_ascii_case("auto") {
                "auto-detect".into()
            } else {
                mapped
            }
        };

        let mut url = reqwest::Url::parse(TRANSLATE_ENDPOINT).map_err(|e| e.to_string())?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("isVertical", "1");
            pairs.append_pair("IG", &session.ig);
            pairs.append_pair("IID", "translator.5026.1");
        }

        let body = {
            let mut encoder = reqwest::Url::parse("https://lookup.invalid/")
                .map_err(|e| e.to_string())?;
            {
                let mut pairs = encoder.query_pairs_mut();
                pairs.append_pair("fromLang", &source);
                pairs.append_pair("to", &target);
                pairs.append_pair("text", req.text.trim());
                pairs.append_pair("token", &session.token);
                pairs.append_pair("key", &session.key);
                pairs.append_pair("tryFetchingGenderDebiasedTranslations", "true");
            }
            encoder.query().unwrap_or("").to_string()
        };

        let mut request = self
            .client
            .post(url)
            .header(USER_AGENT, PAGE_UA)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body);

        if !session.cookie.is_empty() {
            request = request.header(COOKIE, &session.cookie);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Microsoft 网页翻译请求失败: {e}"))?;

        let status = response.status();
        let raw = response
            .text()
            .await
            .map_err(|e| format!("读取 Microsoft 网页翻译响应失败: {e}"))?;

        if !status.is_success() {
            return Err(format!(
                "Microsoft 网页翻译错误 ({status})：非官方接口可能限流或失效。详情：{}",
                truncate(&raw, 200)
            ));
        }

        let parsed: Vec<BingTranslateItem> = serde_json::from_str(&raw).map_err(|e| {
            format!(
                "解析 Microsoft 网页翻译响应失败: {e}; body={}",
                truncate(&raw, 200)
            )
        })?;

        let item = parsed
            .into_iter()
            .next()
            .ok_or_else(|| "Microsoft 网页翻译返回空结果".to_string())?;

        let translated = item
            .translations
            .into_iter()
            .next()
            .ok_or_else(|| "Microsoft 网页翻译未返回译文".to_string())?;

        if translated.text.trim().is_empty() {
            return Err("Microsoft 网页翻译返回空译文".into());
        }

        Ok(TranslationResult {
            engine: self.id().into(),
            text: translated.text,
            source_lang: req.source_lang.clone(),
            target_lang: translated
                .to
                .map(|code| from_microsoft_lang(&code))
                .unwrap_or_else(|| req.target_lang.clone()),
            detected_source_lang: item
                .detected_language
                .map(|d| from_microsoft_lang(&d.language)),
        })
    }
}

struct BingSession {
    key: String,
    token: String,
    ig: String,
    cookie: String,
}

async fn fetch_bing_session(client: &Client) -> Result<BingSession, String> {
    let response = client
        .get(TRANSLATOR_PAGE)
        .header(USER_AGENT, PAGE_UA)
        .send()
        .await
        .map_err(|e| format!("打开 Bing 翻译页失败: {e}"))?;

    let cookie = collect_cookies(&response);
    let status = response.status();
    let html = response
        .text()
        .await
        .map_err(|e| format!("读取 Bing 翻译页失败: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "打开 Bing 翻译页失败 ({status})：{}",
            truncate(&html, 160)
        ));
    }

    let (key, token) = parse_abuse_prevention(&html)?;
    let ig = parse_ig(&html)?;

    Ok(BingSession {
        key,
        token,
        ig,
        cookie,
    })
}

fn collect_cookies(response: &reqwest::Response) -> String {
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|raw| raw.split(';').next())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

/// `params_AbusePreventionHelper = [1712345678901,"token…",3600000];`
fn parse_abuse_prevention(html: &str) -> Result<(String, String), String> {
    let marker = "params_AbusePreventionHelper";
    let start = html
        .find(marker)
        .ok_or_else(|| "无法从 Bing 页面提取 token（页面结构可能已变更）".to_string())?;
    let after = &html[start + marker.len()..];
    let bracket = after
        .find('[')
        .ok_or_else(|| "无法解析 Bing AbusePrevention 参数".to_string())?;
    let list = &after[bracket + 1..];
    let end = list
        .find(']')
        .ok_or_else(|| "无法解析 Bing AbusePrevention 参数".to_string())?;
    let inner = &list[..end];

    let mut parts = inner.splitn(3, ',');
    let key = parts
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Bing AbusePrevention key 为空".to_string())?
        .to_string();
    let token_raw = parts
        .next()
        .map(str::trim)
        .ok_or_else(|| "Bing AbusePrevention token 缺失".to_string())?;
    let token = token_raw
        .trim_matches('"')
        .trim()
        .to_string();
    if token.is_empty() {
        return Err("Bing AbusePrevention token 为空".into());
    }
    Ok((key, token))
}

fn parse_ig(html: &str) -> Result<String, String> {
    // IG:"ABCDEF..." or IG: "ABCDEF..."
    for marker in ["IG:\"", "IG: \""] {
        if let Some(start) = html.find(marker) {
            let rest = &html[start + marker.len()..];
            if let Some(end) = rest.find('"') {
                let ig = rest[..end].trim();
                if !ig.is_empty() {
                    return Ok(ig.to_string());
                }
            }
        }
    }
    Err("无法从 Bing 页面提取 IG（页面结构可能已变更）".into())
}

fn truncate(s: &str, max: usize) -> String {
    let mut iter = s.chars();
    let truncated: String = iter.by_ref().take(max).collect();
    if iter.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

#[derive(Debug, Deserialize)]
struct BingTranslateItem {
    #[serde(rename = "detectedLanguage")]
    detected_language: Option<BingDetectedLanguage>,
    translations: Vec<BingTranslation>,
}

#[derive(Debug, Deserialize)]
struct BingDetectedLanguage {
    language: String,
}

#[derive(Debug, Deserialize)]
struct BingTranslation {
    text: String,
    to: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_abuse_prevention_helper() {
        let html = r#"var params_AbusePreventionHelper = [1712345678901,"abcTokenXYZ",3600000];"#;
        let (key, token) = parse_abuse_prevention(html).unwrap();
        assert_eq!(key, "1712345678901");
        assert_eq!(token, "abcTokenXYZ");
    }

    #[test]
    fn parses_ig() {
        let html = r#"...,IG:"A1B2C3D4E5",IID:"translator.5026""#;
        assert_eq!(parse_ig(html).unwrap(), "A1B2C3D4E5");
    }
}
