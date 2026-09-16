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
- 触发时机：热键 **Released**（避免 Ctrl/Shift 未松开时干扰 Ctrl+C）
- 命令：`get_hotkey_status`

## 剪贴板取词

1. 备份剪贴板文本（非文本内容可能无法完整还原）
2. 写入临时 marker → 模拟 Ctrl+C → 轮询至内容变化或超时
3. 还原剪贴板 → 校验非空 / 最大 8000 字
4. 结果写入 `CaptureState`，事件 `capture-updated`，并显示 `popup`
5. 命令：`get_last_capture`

## 翻译

- 抽象：`Translator` trait；入口 `translate_with_config`
- 引擎：`microsoft`（Azure Translator Text API v3）
- 语言：`source_lang=auto` 时不传 `from`；默认 `target_lang=zh-Hans`
- 代理：`follow_system_proxy=true` 时走 reqwest system-proxy；否则 `no_proxy()`
- 配置：`microsoft_api_key`；区域资源可填 `microsoft_region`
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
