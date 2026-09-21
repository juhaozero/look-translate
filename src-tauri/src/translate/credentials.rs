//! Runtime view of Builtin Engine credentials.
//!
//! TOML still stores flat fields on [`crate::config::EngineConfig`]; adapters
//! read through this normalized shape so credential access stays in one place.

use crate::config::EngineConfig;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuiltinCredentials {
    pub microsoft_api_key: Option<String>,
    pub microsoft_region: Option<String>,
    pub google_api_key: Option<String>,
    pub cloudflare_endpoint: Option<String>,
    pub cloudflare_secret: Option<String>,
    pub baidu_app_id: Option<String>,
    pub baidu_secret: Option<String>,
    pub youdao_app_key: Option<String>,
    pub youdao_app_secret: Option<String>,
}

impl BuiltinCredentials {
    pub fn from_engine_config(engine: &EngineConfig) -> Self {
        Self {
            microsoft_api_key: trim_opt(engine.microsoft_api_key.as_deref()),
            microsoft_region: trim_opt(engine.microsoft_region.as_deref()),
            google_api_key: trim_opt(engine.google_api_key.as_deref()),
            cloudflare_endpoint: trim_opt(engine.cloudflare_endpoint.as_deref()),
            cloudflare_secret: trim_opt(engine.cloudflare_secret.as_deref()),
            baidu_app_id: trim_opt(engine.baidu_app_id.as_deref()),
            baidu_secret: trim_opt(engine.baidu_secret.as_deref()),
            youdao_app_key: trim_opt(engine.youdao_app_key.as_deref()),
            youdao_app_secret: trim_opt(engine.youdao_app_secret.as_deref()),
        }
    }
}

fn trim_opt(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_drops_empty() {
        let mut engine = EngineConfig::default();
        engine.microsoft_api_key = Some("  key  ".into());
        engine.google_api_key = Some("   ".into());
        engine.cloudflare_endpoint = Some("https://x.workers.dev".into());
        engine.baidu_app_id = Some("  2015  ".into());
        engine.youdao_app_secret = Some("".into());

        let creds = BuiltinCredentials::from_engine_config(&engine);
        assert_eq!(creds.microsoft_api_key.as_deref(), Some("key"));
        assert!(creds.google_api_key.is_none());
        assert_eq!(
            creds.cloudflare_endpoint.as_deref(),
            Some("https://x.workers.dev")
        );
        assert_eq!(creds.baidu_app_id.as_deref(), Some("2015"));
        assert!(creds.youdao_app_secret.is_none());
    }
}
