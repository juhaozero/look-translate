# Look Translate

Windows 划词、输入与 OCR 翻译工具；短词可辅以本地词典；OCR 为独立取词路径。

## 翻译服务

支持 Microsoft、必应网页、Google、Google 网页、Cloudflare、百度、有道和 DeepL。服务由 Rust 内置适配器实现，设置页提供各服务专用的密钥字段。DeepL 使用官方 API，根据密钥选择 Free / Pro 地址，见 [DeepL 配置](docs/deepl.md)。

自定义 HTTP 引擎、Engine Profile 与请求模板已移除；不再解析或执行旧的 [engines.*] 配置。

## 启用服务

engine.actives 是当前并行翻译服务的有序列表；engine.active 保持为首项，以兼容只配置一个服务的旧文件。加载时过滤未知服务，若没有支持的服务则回退到 microsoft_web。

## 翻译方向与入口

英文默认译为中文，中文默认译为英文，其他语言使用用户设置；浮层可手动选择目标语。输入翻译默认 Ctrl+Shift+T 唤起，点击翻译或按 Ctrl+Enter 提交。划词默认 Ctrl+Shift+D，OCR 默认 Ctrl+Shift+S，各快捷键可在设置中修改。
