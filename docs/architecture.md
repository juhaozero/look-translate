# 整体架构

## 一句话

Windows 托盘常驻的划词翻译小工具：全局快捷键取词 → Rust 编排翻译/词库 → React 浮层展示。

## 逻辑架构

```
┌─────────────────────────────────────────────────────────┐
│  UI (React + Tauri WebView)                             │
│  ├── popup/     结果浮层（译文、词库、复制、改语言重译）   │
│  └── settings/  设置页（引擎、Key、热键、词典、目标语）   │
└──────────────────────────┬──────────────────────────────┘
                           │ invoke / events
┌──────────────────────────▼──────────────────────────────┐
│  App Core (Rust)                                        │
│  ├── hotkey          全局快捷键注册/开关                  │
│  ├── capture         剪贴板取词（Phase2: OCR）           │
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

详见路线图分期实现。已落地模块见下方；其余目录仍为骨架。

## 配置（`data/config.toml`）

- **路径**：`{executable_dir}/data/config.toml`（开发时即 `src-tauri/target/debug/data/`）
- **启动**：目录不存在则创建；文件不存在则写入默认配置
- **命令**：`get_app_paths` / `get_config` / `save_config`
- **字段**：`general`（语言/热键/代理）、`engine`（active + microsoft_api_key）、`dictionary`（enabled + paths）
- 示例见仓库根目录 `data/config.toml.example`；密钥勿提交

## 热键

- 插件：`tauri-plugin-global-shortcut`
- 默认：`Ctrl+Shift+D`；`hotkey_enabled` 总开关；保存配置或托盘「启用热键」后重新注册
- 当前触发：打开 `popup` 浮层（取词/翻译在后续步骤接入）
- 命令：`get_hotkey_status`
