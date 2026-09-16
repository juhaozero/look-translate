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

详见路线图分期实现。脚手架阶段仅保留模块目录与空实现，业务在后续 Phase 落地。

## 配置（`data/config.toml`）

安装目录旁便携 `data/`；密钥只存本地。详见 `docs/product-decisions.md`。
