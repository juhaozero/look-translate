# Look Translate

[中文](README.md)

**Windows select-to-translate / on-screen OCR translation utility**

Select text and press a hotkey to translate. Supports parallel engines, optional local MDict dictionaries, and a separate OCR selection flow. Built with [Tauri 2](https://tauri.app/) + React + Rust. Portable config lives next to the executable under `data/`.

[Features](#features) · [Install](#install) · [Usage](#usage) · [Configuration](#configuration) · [Development](#development) · [Build](#build)

![Platform](https://img.shields.io/badge/platform-Windows-0078D4?logo=windows&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-0.0.4-blue)
![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?logo=tauri&logoColor=white)

## Features

- **Select to translate**: select text → global hotkey → popup near the cursor
- **Parallel engines**: enable multiple services at once (Microsoft / Bing web / Google / Cloudflare Workers / custom HTTP); results are shown per engine with individual copy actions
- **OCR translate**: separate hotkey to select a screen region (system OCR or Tesseract.js)
- **Local dictionary**: short words can look up MDict (`.mdx`); optional one-click ECDICT install
- **Portable config**: `data/config.toml` beside the install dir; system proxy, launch at startup, editable hotkeys

## Install

### Release builds (recommended)

1. Open [Releases](https://github.com/juhaozero/look-translate/releases)
2. Download the **NSIS** installer (per-user) or MSI
3. After install the app stays in the tray; first launch creates `data/` next to the executable

> WebView2 is required. The installer can bootstrap it when missing.

### Run from source

```bash
git clone https://github.com/juhaozero/look-translate.git
cd look-translate
npm install
npm run tauri:dev
```

Requirements:

- Windows 10/11
- [Node.js](https://nodejs.org/) (LTS recommended)
- [Rust](https://rustup.rs/) (stable)
- Visual Studio Build Tools with Windows SDK (required for Tauri on Windows)

## Usage

| Action              | Default hotkey        | Notes                                   |
| ------------------- | --------------------- | --------------------------------------- |
| Select to translate | `Ctrl+Shift+D`        | Select text first; editable in Settings |
| OCR translate       | `Ctrl+Shift+S`        | Drag a region, then translate           |
| Close popup         | `Esc` / click outside | —                                       |

Tray:

- **Double-click**: open Settings
- Menu: **Settings** / **Enable hotkeys** / **Quit**

Settings cover languages, hotkeys, multi-engine toggles and keys, dictionary paths, OCR engine, launch at startup, and more. Changes auto-save.

## Configuration

Runtime config file:

```text
<install-dir>/data/config.toml
```

See [`data/config.toml.example`](data/config.toml.example).

Example:

```toml
[general]
target_lang = "zh-CN"
source_lang = "auto"
hotkey_translate = "Ctrl+Shift+D"
hotkey_ocr = "Ctrl+Shift+S"

[engine]
# Run multiple engines in parallel
actives = ["microsoft_web", "google_web"]
active = "microsoft_web"

# Cloudflare Workers (repo work.js):
# actives = ["cloudflare"]
# cloudflare_endpoint = "https://your-worker.workers.dev/"
# cloudflare_secret = "your-secret"   # must match: wrangler secret put SECRET_PASS
# Client uses POST JSON + Authorization: Bearer <secret> (do not hardcode the secret in Worker source)
```

Custom providers (e.g. DeepL) can be added via config-driven `[engines.<id>]` profiles. See [`docs/engine-profiles.md`](docs/engine-profiles.md).

Logs: `data/logs/` (also openable from About).

## Built-in engines

| ID              | Description                          | Key                          |
| --------------- | ------------------------------------ | ---------------------------- |
| `microsoft`     | Azure Translator Text API v3         | Required                     |
| `microsoft_web` | Bing web endpoint (unofficial)       | Not required                 |
| `google`        | Google Cloud Translation API v2      | Required                     |
| `google_web`    | Google gtx web endpoint (unofficial) | Not required                 |
| `cloudflare`    | Cloudflare Workers / translate-api   | Worker URL + optional secret |
| `baidu`         | Baidu Translate open platform        | App ID + secret              |
| `youdao`        | Youdao Zhiyun text translation       | App key + app secret         |
| `[engines.*]`   | Config-driven custom HTTP            | Per profile                  |

Unofficial web endpoints may rate-limit or break; prefer official APIs for production.

## Development

```bash
npm install
npm run tauri:dev      # hot reload
npm run build          # frontend only
cargo test --manifest-path src-tauri/Cargo.toml
```

Windows:

- `popup` — translation popup
- `settings` — settings
- `ocr-select` — OCR region overlay

More: [`docs/architecture.md`](docs/architecture.md) · [`docs/packaging.md`](docs/packaging.md) · [`CONTEXT.md`](CONTEXT.md)

## Build

```bash
npm run tauri:build:nsis   # recommended: NSIS per-user
npm run tauri:build:msi    # MSI
npm run tauri:build        # NSIS + MSI
```

See [`docs/packaging.md`](docs/packaging.md) for artifacts and uninstall behavior.

## Tech stack

- **Shell**: Tauri 2
- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust (capture, hotkeys, translation, dictionary, OCR orchestration)

## License

[MIT](LICENSE) © juhaozero
