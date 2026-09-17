# 打包与安装（Windows）

## 产物

| 目标 | 命令 | 默认输出目录 |
|---|---|---|
| NSIS（推荐） | `npm run tauri:build:nsis` | `src-tauri/target/release/bundle/nsis/` |
| MSI | `npm run tauri:build:msi` | `src-tauri/target/release/bundle/msi/` |
| 两者 | `npm run tauri:build` | 同上两个目录 |

若设置了 `CARGO_TARGET_DIR`，产物会落在该目录下的 `release/bundle/`。

依赖：Rust、Node、[WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)（安装包会带 bootstrapper）、WiX Toolset（打 MSI 时）、NSIS（Tauri 会按需拉取）。

## 安装模式（NSIS）

- **`installMode: currentUser`**：默认装到当前用户目录（通常为 `%LOCALAPPDATA%\Look Translate\`），**无需管理员**，且可对安装目录旁的 `data/` 正常读写。
- 安装向导可选简体中文 / English。
- 自定义钩子：`src-tauri/windows/hooks.nsh`。

> 不推荐装到 `Program Files`：便携 `data/` 写配置可能因权限失败。若用 MSI 默认路径，请自行确认目录可写，或改用 NSIS 当前用户安装。

## `data/` 位置

运行时解析为：

```text
{executable_dir}/data/config.toml
```

即与 `Look Translate.exe` 同级的 `data/` 目录（与产品决策一致）。开发态一般为：

```text
src-tauri/target/debug/data/
```

首次启动会创建目录并写入默认配置。示例见仓库 `data/config.toml.example`（勿把真实 Key 提交进 Git）。

## 卸载策略

1. 卸载程序会移除安装目录（含 exe、资源）。
2. 若存在 `data/`，NSIS 钩子会弹出确认：
   - **是**：不备份，随安装目录一并删除（配置与 API Key 清除）。
   - **否**：先将 `data/` 备份到 `%LOCALAPPDATA%\Look Translate\data`，再继续卸载；重装后可把该目录复制回新安装目录旁的 `data\`。
3. MSI 卸载一般直接删除安装目录；若需保留配置，请先手动复制 `data/`。

## 签名（可选）

正式分发建议对安装包与 exe 做代码签名。Tauri 可通过环境变量 / `bundle.windows.certificateThumbprint` 等配置；本仓库 MVP 不强制签名。

## 验收清单

- [ ] `npm run tauri:build:nsis` 成功生成 `.exe` 安装包
- [ ] 当前用户安装后，设置页「数据位置」指向安装目录旁 `data/`
- [ ] 填入 Key、保存配置后重启仍有效
- [ ] 卸载时弹出是否删除 `data/`；选否可在 `%LOCALAPPDATA%\Look Translate\data` 找到备份
- [ ] （可选）`npm run tauri:build:msi` 生成 `.msi`
