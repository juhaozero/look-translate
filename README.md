# Look Translate

[English](README.en.md)

**Windows 划词翻译 / 屏幕 OCR 翻译小工具**

选中文本按快捷键即可翻译；支持多引擎并行、可选本地 MDict 词典、独立 OCR 框选识别。基于 [Tauri 2](https://tauri.app/) + React + Rust，配置便携，安装目录旁 `data/` 即可带走。

[功能](#功能) · [安装](#安装) · [使用](#使用) · [配置](#配置) · [开发](#开发) · [打包](#打包)

Platform
License
Version
Tauri

## 功能

- **划词翻译**：选中文本 → 全局热键 → 光标附近浮层出译文
- **多引擎并行**：可同时启用多个服务（微软 / 必应网页 / Google / Cloudflare Workers / 自定义 HTTP），结果分块展示、按引擎复制
- **OCR 翻译**：独立热键框选屏幕文字（系统 OCR 或 Tesseract.js）
- **本地词典**：短词可查 MDict（`.mdx`）；支持一键安装 ECDICT 推荐词库
- **便携配置**：`data/config.toml` 与安装目录同级；跟随系统代理、开机自启、热键可改

## 安装

### 发行版（推荐）

1. 打开 [Releases](https://github.com/juhaozero/look-translate/releases)
2. 下载 **NSIS** 安装包（当前用户安装）或 MSI
3. 安装后托盘常驻；首次运行会在可执行文件旁创建 `data/`

> 需要 WebView2 运行时。未安装时，安装包会尝试引导下载。

### 从源码运行

```bash
git clone https://github.com/juhaozero/look-translate.git
cd look-translate
npm install
npm run tauri:dev
```

环境要求：

- Windows 10/11
- [Node.js](https://nodejs.org/)（建议 LTS）
- [Rust](https://rustup.rs/)（stable）
- Visual Studio Build Tools（含 Windows SDK，Tauri Windows 构建需要）

## 使用

| 操作     | 默认快捷键       | 说明                           |
| -------- | ---------------- | ------------------------------ |
| 划词翻译 | `Ctrl+Shift+D`   | 选中文本后按下；可在设置中修改 |
| OCR 翻译 | `Ctrl+Shift+S`   | 框选屏幕区域识别后再翻译       |
| 关闭浮层 | `Esc` / 点击外部 | —                              |

托盘：

- **左键双击**：打开设置
- 菜单：**设置** / **启用热键** / **退出**

设置页可配置：目标语与源语、热键、多引擎开关与 Key、词典路径、OCR 引擎、开机自启等。更改后会自动保存。

## 配置

运行时配置文件：

```text
<安装目录>/data/config.toml
```

示例见 `[data/config.toml.example](data/config.toml.example)`。

常用片段：

```toml
[general]
target_lang = "zh-CN"
source_lang = "auto"
hotkey_translate = "Ctrl+Shift+D"
hotkey_ocr = "Ctrl+Shift+S"

[engine]
# 可并行启用多个引擎
actives = ["microsoft_web", "google_web"]
active = "microsoft_web"

# Cloudflare Workers（translate-api 兼容）示例：
# actives = ["cloudflare"]
# cloudflare_endpoint = "https://your-worker.workers.dev/"
# cloudflare_secret = "your-secret"
```

自定义厂商（如 DeepL）可通过 `[engines.<id>]` 配置驱动接入，说明见 `[docs/engine-profiles.md](docs/engine-profiles.md)`。

日志目录：`data/logs/`（关于页可一键打开）。

## 内置翻译引擎

| ID              | 说明                               | Key                       |
| --------------- | ---------------------------------- | ------------------------- |
| `microsoft`     | Azure Translator Text API v3       | 需要                      |
| `microsoft_web` | 必应网页接口（非官方）             | 不需要                    |
| `google`        | Google Cloud Translation API v2    | 需要                      |
| `google_web`    | Google gtx 网页接口（非官方）      | 不需要                    |
| `cloudflare`    | Cloudflare Workers / translate-api | Worker 地址 + 可选 secret |
| `[engines.*]`   | Config-driven 自定义 HTTP          | 按 Profile                |

非官方网页接口可能限流或失效，生产环境建议使用官方 API。

## 开发

```bash
npm install
npm run tauri:dev      # 开发热重载
npm run build          # 仅前端
cargo test --manifest-path src-tauri/Cargo.toml
```

窗口：

- `popup`：划词 / OCR 译文浮层
- `settings`：设置
- `ocr-select`：OCR 框选层

更多说明：`[docs/architecture.md](docs/architecture.md)` · `[docs/packaging.md](docs/packaging.md)` · `[CONTEXT.md](CONTEXT.md)`

## 打包

```bash
npm run tauri:build:nsis   # 推荐：NSIS 当前用户安装
npm run tauri:build:msi    # MSI
npm run tauri:build        # NSIS + MSI
```

产物与卸载行为见 `[docs/packaging.md](docs/packaging.md)`。

## 技术栈

- **桌面壳**：Tauri 2
- **前端**：React 19 + TypeScript + Vite
- **后端**：Rust（取词、热键、翻译、词典、OCR 编排）

## License

[MIT](LICENSE) © juhaozero
