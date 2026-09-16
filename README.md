# look-translate

Windows 划词翻译小工具（Tauri 2 + React）。

## 文档

| 文档 | 内容 |
|---|---|
| [docs/product-decisions.md](docs/product-decisions.md) | 已锁定产品决策 |
| [docs/architecture.md](docs/architecture.md) | 整体架构与模块 |
| [docs/roadmap.md](docs/roadmap.md) | 分期与 MVP 验收 |

## 开发

```bash
npm install
npm run tauri dev
```

默认托盘常驻：菜单含「设置 / 退出」。浮层窗口 `popup`、设置窗口 `settings`。

## 当前进度

Phase 1 第 3 步已完成：全局热键。下一步：剪贴板取词。
