# Look Translate

Windows 划词翻译小工具（Tauri 2 + React）。

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
