# Look Translate

Windows 划词翻译：选中文本后经可配置引擎出译文；短词可辅以本地词典；OCR 为独立路径。

## Language

**Builtin Engine**:
宿主内置、代码实现的翻译引擎（如 Microsoft、Google、Cloudflare）。
_Avoid_: 插件引擎、官方引擎（易与厂商「官方 API」混淆）

**Config-driven Engine**:
由配置描述请求/鉴权/响应抽取的通用引擎；加新厂商以改配置为主，不以改宿主代码为主。
_Avoid_: 插件、脚本引擎、自建网关（那是另一条扩展路径）

**Engine Active**:
当前唯一生效的翻译引擎标识（同一时间只用一个）。若匹配 Builtin Engine 则走内置实现；否则在命名的 Config-driven 配置中查找。与 Builtin 同名的 Engine Profile 被忽略（Builtin 优先）。
_Avoid_: 默认引擎（易与出厂默认混淆）

**Engine Profile**:
一份以 id 命名的 Config-driven 配置（如 `engines.deepl`）；用户通过把 Engine Active 设为该 id 来启用。设置页只读展示并可切换；展示名用可选 `label`（缺省为 id）；图标用统一「自定义」占位，不支持自定义图标文件。编辑仍在 `config.toml`。
_Avoid_: 插件、扩展包、http 单槽

**Auth Mode**:
Config-driven Engine 请求鉴权的种类。第一版支持：`none`、`header`、`query`、`bearer`、`basic`。不含云厂商签名（如腾讯 TC3）。
_Avoid_: API Key（太宽泛，应落到具体 Auth Mode）

**Request Template**:
Engine Profile 中描述 HTTP 方法、URL、Header、Query、Body 的配置；可用占位符（如 `{{text}}`、`{{source_lang}}`、`{{target_lang}}`、`{{extra.*}}`）和语言映射。
_Avoid_: 脚本、插件逻辑

**Response Extract**:
从 JSON 响应中取出译文（及可选错误信息）的路径配置。
_Avoid_: 正则扒全文（第一版不以之为正路）
