# 整体架构

## 一句话

Windows 托盘常驻的划词翻译小工具：全局快捷键取词 → Rust 编排翻译/词库 → React 浮层展示。

## 逻辑架构

```
┌─────────────────────────────────────────────────────────┐
│  UI (React + Tauri WebView)                             │
│  ├── popup/       结果浮层（译文、词库、复制、改语言重译） │
│  ├── ocr-select/  OCR 框选层（拖拽选区）                   │
│  └── settings/    设置页（引擎、Key、热键、词典、目标语） │
└──────────────────────────┬──────────────────────────────┘
                           │ invoke / events
┌──────────────────────────▼──────────────────────────────┐
│  App Core (Rust)                                        │
│  ├── hotkey          全局快捷键注册/开关                  │
│  ├── capture         剪贴板取词（划词热键）              │
│  ├── ocr             截屏 OCR（独立热键，互不兜底）        │
│  ├── translate       Translator trait + 多引擎           │
│  ├── dictionary      MDict provider（短词才查）          │
│  ├── cache           内存 LRU（100）                     │
│  ├── config          读写 data/config.toml               │
│  └── popup_ctrl      定位/显示/隐藏浮层窗口               │
└─────────────────────────────────────────────────────────┘
         │                │                 │
         ▼                ▼                 ▼
   系统剪贴板      微软等 HTTP API      本地 .mdx 文件
   系统代理         (reqwest)           (引用用户路径)
```

## 推荐仓库结构

```
look-translate/
├── docs/
│   ├── product-decisions.md
│   ├── architecture.md
│   ├── packaging.md
│   └── roadmap.md
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   ├── capture/
│   │   ├── translate/
│   │   ├── dictionary/
│   │   ├── cache/
│   │   ├── config/
│   │   └── hotkey/
│   ├── windows/
│   │   └── hooks.nsh      # NSIS 卸载时 data/ 备份询问
│   ├── capabilities/
│   └── tauri.conf.json
├── src/
│   ├── main.tsx
│   ├── popup/
│   ├── settings/
│   └── shared/
├── data/
├── LICENSE
└── package.json
```

## 核心模块职责

详见路线图分期实现。已落地模块见下方。

## 配置（`data/config.toml`）

- **路径**：`{executable_dir}/data/config.toml`（开发时即 `src-tauri/target/debug/data/`）
- **启动**：目录不存在则创建；文件不存在则写入默认配置
- **命令**：`get_app_paths` / `get_config` / `save_config`
- **字段**：`general`（语言/热键/代理）、`engine`（active + microsoft_* + google_api_key）、`dictionary`（enabled + paths）
- 示例见仓库根目录 `data/config.toml.example`；密钥勿提交
- **安装/卸载**：见 `docs/packaging.md`（NSIS 当前用户安装；卸载可备份 `data/`）

## 热键

- 插件：`tauri-plugin-global-shortcut`
- 默认：`Ctrl+Shift+D`；`hotkey_enabled` 总开关；保存配置或托盘「启用热键」后重新注册
- 触发时机：热键 **Released**（避免 Ctrl/Shift 未松开时干扰 Ctrl+C）
- 命令：`get_hotkey_status`

## 剪贴板取词

1. 备份剪贴板文本（非文本内容可能无法完整还原）
2. 写入临时 marker → 模拟 **Ctrl+C**（使用 `Key::C` / VK_C，禁止 Unicode 假按键）→ 轮询至内容变化或超时
3. 还原剪贴板 → 校验非空 / 最大 8000 字
4. 结果写入 `CaptureState`，事件 `capture-updated`，并显示 `popup`
5. 命令：`get_last_capture`
6. **不**在失败时自动改走 OCR

## OCR（独立热键）

- 默认热键：`Ctrl+Shift+S`（可改）；与划词热键不可相同
- 热键先截当前显示器快照，再打开透明 `ocr-select` 框选层；预标指针附近约 480×160 建议区
- 拖拽重选 / Enter 确认 / Esc 取消 → 从快照裁剪（避免截到遮罩）
- 引擎（`[ocr] engine`，设置页「服务设置」可切换）：
  - `system`（默认）：`Windows.Media.Ocr`
  - `tesseract`：前端 [Tesseract.js](https://github.com/naptha/tesseract.js/) WASM（`eng+chi_sim`），命令 `submit_ocr_text`
- 命令：`get_ocr_region_hint` / `confirm_ocr_region` / `cancel_ocr_select` / `submit_ocr_text`
- `source=ocr`；与剪贴板路径互不兜底
- 系统 OCR 需已安装语言包；Tesseract 首次使用会下载模型

## 翻译

- 抽象：`Translator` trait；入口 `translate_with_config`
- 引擎：`microsoft`（Azure Translator Text API v3）；`microsoft_web`（非官方 Bing 网页，无 Key）；`google`（Cloud Translation API v2 + Key）；`google_web`（非官方 gtx，无 Key）；`self_hosted`（自建 translate-api / Cloudflare Worker）
- 语言：设置侧统一 Google 风格（`zh-CN` / `zh-TW`）；读配置兼容旧 `zh-Hans` / `zh-Hant`；微软系引擎内反向映射；`source_lang=auto` 时不传 `from`（自建引擎则按 `en`）；默认 `target_lang=zh-CN`
- 代理：`follow_system_proxy=true` 时走 reqwest system-proxy；否则 `no_proxy()`
- 配置：`microsoft_api_key` / `microsoft_region`；`google_api_key`（仅官方 Google）；`self_hosted_endpoint` / `self_hosted_secret`
- 流水线：取词成功后异步翻译；事件 `translation-updated`（loading/ok/error）
- 命令：`get_last_translation` / `translate_text`

## 浮层

- 打开时定位到光标附近（监视器内钳制）
- Esc / 关闭按钮 / 失焦（点击外部）隐藏；刚打开 250ms 内忽略失焦防闪烁
- 目标语下拉可即时重译；一键复制译文（`copy_text`）
- 加载态 / 错误重试

## 翻译缓存

- 内存 LRU，容量 **100**（进程内，不落盘）
- Key：`engine + source_lang + target_lang + text`
- 命中：跳过网络请求，浮层标注「缓存」；仅缓存成功结果

## 词典（MDict）

- 仅短词：≤30 字符 **且** ≤3 token，且无换行
- 配置 `dictionary.paths` 只查**第一本**；可为 `.mdx` 文件或含 `.mdx` 的目录
- **相对路径**相对安装目录旁 `data/`（推荐内置路径：`dicts/ecdict.mdx`）
- 设置页可一键下载 ECDICT（`ecdict-mdx-headless-28.zip`）到 `data/dicts/ecdict.mdx`，**不进安装包**
- `mdict-rs` 解析；释义 HTML → 纯文本；不加载 `.mdd`
- 翻译失败时若短词命中词库，仍展示词典区
- 保存配置后使已打开词典失效并按需重开

## 设置页

- 分区：语言 / 引擎 / 热键 / 词典 / 数据位置
- 语言与引擎用下拉；Microsoft Key/Region 条件展示
- 词典：安装推荐 ECDICT、选择 `.mdx` / 文件夹（`tauri-plugin-dialog`）
- 未保存标记、重新加载、保存校验与热键注册状态提示
- 数据位置展示便携路径，并提示 NSIS 卸载备份策略

## 打包

- 目标：`nsis` + `msi`（`tauri.conf.json` → `bundle.targets`）
- NSIS：`installMode=currentUser`，钩子 `windows/hooks.nsh`
- 文档：`docs/packaging.md`
