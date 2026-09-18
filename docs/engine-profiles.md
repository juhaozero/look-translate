# 自定义翻译引擎（Config-driven）

通过 `data/config.toml` 的 `[engines.<id>]` 接入任意 REST 翻译 API，无需改代码。  
决策见 [ADR 0001](./adr/0001-config-driven-engines.md)。

## 快速流程

1. 设置 → 服务设置 → **添加通用模板**（写入 `[engines.custom]`）
2. **打开配置文件**，按下方字段说明填写
3. **重新加载配置**，打开该引擎开关

也可直接复制本文「完整示例」到 `config.toml`，把 `custom` 改成任意 id（勿与内置引擎同名：`microsoft` / `google` / `cloudflare` 等）。

## 字段说明

| 字段 | 含义 |
|------|------|
| `label` | 设置页显示名（可选，默认用 id） |
| `method` | `GET` 或 `POST` |
| `url` | 接口地址，可用占位符 |
| `auth` | `none` / `header` / `query` / `bearer` / `basic` |
| `auth_header` / `auth_value` | `auth=header` 时：头名与值模板 |
| `token` | `auth=bearer` 时的 token（可 `{{extra.api_key}}`） |
| `username` / `password` | `auth=basic` |
| `auth_query_key` / `auth_query_value` | `auth=query` |
| `headers` / `query` / `body` | 额外头、查询、正文；值为字符串模板 |
| `body_type` | `json` / `form` / `none` |
| `extra` | 自由键值，模板里用 `{{extra.xxx}}` |
| `lang_map` | 应用语言码 → 厂商语言码；映射成空字符串的字段会从 body/query 省略 |
| `text_path` | JSON 点路径取译文，如 `translations.0.text` |
| `error_path` | 可选，错误信息路径 |

**占位符**：`{{text}}`、`{{source_lang}}`、`{{target_lang}}`、`{{extra.<键>}}`。

## 通用骨架（与设置页「添加通用模板」相同）

```toml
[engines.custom]
label = "自定义 HTTP"
method = "POST"
url = "https://example.com/v1/translate"
auth = "bearer"
token = "{{extra.api_key}}"
body_type = "json"
text_path = "data.text"
error_path = "error.message"

[engines.custom.body]
text = "{{text}}"
source_lang = "{{source_lang}}"
target_lang = "{{target_lang}}"

[engines.custom.extra]
api_key = "your-api-key"

[engines.custom.lang_map]
zh-CN = "zh-CN"
zh-TW = "zh-TW"
en = "en"
ja = "ja"
ko = "ko"
fr = "fr"
de = "de"
es = "es"
auto = "auto"
```

启用：

```toml
[engine]
active = "custom"
```

## 完整示例：DeepL Free API

把下面整段贴进 `config.toml`（可把 id 从 `deepl` 改成别的）。密钥只放本机，勿提交仓库。

```toml
[engine]
active = "deepl"

[engines.deepl]
label = "DeepL"
method = "POST"
url = "https://api-free.deepl.com/v2/translate"
auth = "header"
auth_header = "Authorization"
auth_value = "DeepL-Auth-Key {{extra.api_key}}"
body_type = "form"
text_path = "translations.0.text"
error_path = "message"

[engines.deepl.body]
text = "{{text}}"
target_lang = "{{target_lang}}"
source_lang = "{{source_lang}}"

[engines.deepl.extra]
api_key = "your-deepl-auth-key"

[engines.deepl.lang_map]
zh-CN = "ZH"
zh-TW = "ZH"
en = "EN"
ja = "JA"
ko = "KO"
fr = "FR"
de = "DE"
es = "ES"
auto = ""
```

说明：`auto = ""` 时不会发送空的 `source_lang`，由 DeepL 自动检测。正式版 API 主机为 `https://api.deepl.com/v2/translate`。

## 多厂商

复制 `[engines.custom]` 块，改成不同 id（如 `engines.libre`），分别填写后用 `active` 切换。设置页会列出所有非内置 id 的 Profile。
