//! Download and install the recommended ECDICT MDX dictionary into `data/dicts/`.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::{save_to_path, AppConfig, ConfigPaths};

/// Relative path stored in `dictionary.paths` (resolved against `data/`).
pub const RECOMMENDED_DICT_REL_PATH: &str = "dicts/ecdict.mdx";
pub const DICTS_DIR_NAME: &str = "dicts";
pub const RECOMMENDED_MDX_FILE: &str = "ecdict.mdx";

const DOWNLOAD_URL: &str =
    "https://github.com/skywind3000/ECDICT/releases/download/1.0.28/ecdict-mdx-headless-28.zip";
const DOWNLOAD_FILE_NAME: &str = "ecdict-mdx-headless-28.zip";
const DOWNLOAD_TIMEOUT_SECS: u64 = 600;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallRecommendedDictResult {
    /// `already_present` | `installed`
    pub status: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub config: AppConfig,
}

pub fn recommended_mdx_path(data_dir: &Path) -> PathBuf {
    data_dir.join(DICTS_DIR_NAME).join(RECOMMENDED_MDX_FILE)
}

pub async fn install_recommended_dictionary(
    paths: &ConfigPaths,
    mut config: AppConfig,
    follow_system_proxy: bool,
) -> Result<InstallRecommendedDictResult, String> {
    crate::config::ensure_data_dir(paths)?;
    let dest = recommended_mdx_path(&paths.data_dir);
    let absolute_path = dest.to_string_lossy().into_owned();

    let status = if dest.is_file() {
        "already_present".to_string()
    } else {
        download_and_extract(paths, follow_system_proxy, &dest).await?;
        if !dest.is_file() {
            return Err("安装后未找到词典文件".into());
        }
        "installed".to_string()
    };

    config.dictionary.enabled = true;
    ensure_recommended_path_first(&mut config);
    save_to_path(&paths.config_path, &config)?;
    let saved = crate::config::load_from_path(&paths.config_path)?;

    Ok(InstallRecommendedDictResult {
        status,
        relative_path: RECOMMENDED_DICT_REL_PATH.to_string(),
        absolute_path,
        config: saved,
    })
}

fn ensure_recommended_path_first(config: &mut AppConfig) {
    let recommended = RECOMMENDED_DICT_REL_PATH.to_string();
    config
        .dictionary
        .paths
        .retain(|p| p.trim() != recommended && !p.trim().is_empty());
    config.dictionary.paths.insert(0, recommended);
}

async fn download_and_extract(
    paths: &ConfigPaths,
    follow_system_proxy: bool,
    dest_mdx: &Path,
) -> Result<(), String> {
    let tmp_dir = paths.data_dir.join(DICTS_DIR_NAME).join(".tmp");
    fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("创建临时目录失败: {e}"))?;
    let zip_path = tmp_dir.join(DOWNLOAD_FILE_NAME);

    let cleanup = |zip_path: &Path, tmp_dir: &Path| {
        let _ = fs::remove_file(zip_path);
        let _ = fs::remove_dir_all(tmp_dir);
    };

    if let Err(err) = download_zip(&zip_path, follow_system_proxy).await {
        cleanup(&zip_path, &tmp_dir);
        return Err(err);
    }

    if let Err(err) = extract_first_mdx(&zip_path, dest_mdx) {
        cleanup(&zip_path, &tmp_dir);
        let _ = fs::remove_file(dest_mdx);
        return Err(err);
    }

    cleanup(&zip_path, &tmp_dir);
    Ok(())
}

async fn download_zip(zip_path: &Path, follow_system_proxy: bool) -> Result<(), String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .user_agent(concat!("look-translate/", env!("CARGO_PKG_VERSION")));
    if !follow_system_proxy {
        builder = builder.no_proxy();
    }
    let client = builder
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(DOWNLOAD_URL)
        .send()
        .await
        .map_err(|e| format!("下载词典失败: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "下载词典失败: HTTP {}",
            response.status().as_u16()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取下载内容失败: {e}"))?;
    if bytes.len() < 1024 {
        return Err("下载内容过小，可能不是有效的词典压缩包".into());
    }

    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建下载目录失败: {e}"))?;
    }
    let mut file =
        File::create(zip_path).map_err(|e| format!("写入下载文件失败: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("写入下载文件失败: {e}"))?;
    Ok(())
}

fn extract_first_mdx(zip_path: &Path, dest_mdx: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| format!("打开压缩包失败: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("解析压缩包失败: {e}"))?;

    let mut mdx_index = None;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("读取压缩包条目失败: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        let file_name = name.rsplit('/').next().unwrap_or(name.as_str());
        if file_name.to_ascii_lowercase().ends_with(".mdx") {
            mdx_index = Some(i);
            break;
        }
    }

    let index = mdx_index.ok_or_else(|| "压缩包中未找到 .mdx 文件".to_string())?;
    let mut entry = archive
        .by_index(index)
        .map_err(|e| format!("读取 .mdx 条目失败: {e}"))?;

    if let Some(parent) = dest_mdx.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建词典目录失败: {e}"))?;
    }
    // Write via temp then rename so a failed extract does not leave a half file.
    let tmp_out = dest_mdx.with_extension("mdx.partial");
    {
        let mut out =
            File::create(&tmp_out).map_err(|e| format!("创建词典文件失败: {e}"))?;
        io::copy(&mut entry, &mut out).map_err(|e| format!("解压词典失败: {e}"))?;
        out.flush().map_err(|e| format!("写入词典失败: {e}"))?;
    }
    if dest_mdx.exists() {
        fs::remove_file(dest_mdx).map_err(|e| format!("替换旧词典失败: {e}"))?;
    }
    fs::rename(&tmp_out, dest_mdx).map_err(|e| format!("完成词典安装失败: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_path_goes_first() {
        let mut config = AppConfig::default();
        config.dictionary.paths = vec!["D:/other.mdx".into(), RECOMMENDED_DICT_REL_PATH.into()];
        ensure_recommended_path_first(&mut config);
        assert_eq!(
            config.dictionary.paths,
            vec![RECOMMENDED_DICT_REL_PATH.to_string(), "D:/other.mdx".into()]
        );
    }
}
