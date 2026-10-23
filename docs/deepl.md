# DeepL 翻译服务

在「设置 → 服务设置」展开「DeepL 翻译」，填写 DeepL API Key，然后启用该服务。支持与其他服务并行翻译，以及划词、OCR 和输入翻译。

使用 DeepL API 套餐提供的密钥。以 `:fx` 结尾的 Free 密钥使用 `https://api-free.deepl.com/v2/translate`，其他密钥使用 Pro 地址 `https://api.deepl.com/v2/translate`。请求由 Rust 后端发送，通过 `Authorization: DeepL-Auth-Key …` 鉴权，自动源语言时省略 `source_lang`。

简体、繁体目标语言分别映射到 `ZH-HANS`、`ZH-HANT`；中文源语言使用 `ZH`。密钥保存于本地便携配置，请勿提交到仓库。

```toml
[engine]
actives = ["deepl"]
active = "deepl"
deepl_api_key = "your-api-key:fx"
```

自定义 HTTP 引擎已移除。旧 `[engines.*]` 配置会被忽略，保存时不再写出；旧启用列表会保留仍支持的服务。如果旧列表仅包含已移除的服务，则改用无需密钥的必应网页翻译。之前通过自定义 Profile 配置 DeepL 的用户，需要将 API Key 填入服务设置或上述 `deepl_api_key` 字段。

官方参考：[鉴权与 Free/Pro 地址](https://developers.deepl.com/docs/getting-started/auth)、[文本翻译请求](https://developers.deepl.com/api-reference/translate/request-translation)。
