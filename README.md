# Look Translate

Windows 划词翻译小工具（Tauri 2 + React）。

## 文档

| 文档 | 内容 |
|---|---|
| [docs/product-decisions.md](docs/product-decisions.md) | 已锁定产品决策 |
| [docs/architecture.md](docs/architecture.md) | 整体架构与模块 |
| [docs/roadmap.md](docs/roadmap.md) | 分期与 MVP 验收 |
| [docs/packaging.md](docs/packaging.md) | Windows 安装包、`data/` 卸载策略、GitHub 发版 |

## 开发

```bash
npm install
npm run tauri:dev
```

托盘常驻：左键双击打开设置；菜单含「设置 / 启用热键 / 退出」。浮层窗口 `popup`、设置窗口 `settings`。

## 打包

```bash
npm run tauri:build:nsis   # 推荐：当前用户安装
npm run tauri:build:msi    # 可选 MSI
npm run tauri:build        # NSIS + MSI
```

配置目录为安装（或 exe）旁的 `data/`。卸载与 **GitHub 打 tag 发版**（draft Release + changelog）见 [docs/packaging.md](docs/packaging.md#github-发版)。

## 当前进度

Phase 1 第 10 步已完成：NSIS/MSI 打包配置与 `data/` 卸载说明。Phase 1 MVP 任务清单已全部落地，可按路线图验收。
