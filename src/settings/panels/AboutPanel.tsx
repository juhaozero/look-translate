import type { AboutPanelProps } from "./types";

export function AboutPanel({
  info,
  paths,
  onOpenExternalUrl,
  onOpenLogDir,
  onOpenConfigFile,
  onOpenDataDir,
}: AboutPanelProps) {
  return (
    <section className="settings-about">
      <h2 className="settings-about-name">
        {info?.name ?? "Look Translate"}
      </h2>
      <p className="settings-about-ver">
        {info?.version ?? "…"}
      </p>
      <p className="settings-about-desc">
        {info?.description ?? "Windows 划词翻译小工具"}
      </p>

      <div className="settings-about-links" role="navigation" aria-label="项目链接">
        <button
          type="button"
          className="settings-about-link"
          onClick={() =>
            void onOpenExternalUrl(
              "https://github.com/juhaozero/look-translate",
            )
          }
        >
          GitHub
        </button>
        <button
          type="button"
          className="settings-about-link"
          onClick={() =>
            void onOpenExternalUrl(
              "https://github.com/juhaozero/look-translate/issues",
            )
          }
        >
          问题反馈
        </button>
      </div>

      <div className="settings-about-links" role="navigation" aria-label="本地资源">
        <button
          type="button"
          className="settings-about-link"
          onClick={() => void onOpenLogDir()}
        >
          查看日志
        </button>
        <button
          type="button"
          className="settings-about-link"
          onClick={() => void onOpenConfigFile()}
        >
          查看配置文件
        </button>
        <button
          type="button"
          className="settings-about-link"
          onClick={() => void onOpenDataDir()}
        >
          打开数据目录
        </button>
      </div>

      <div className="settings-about-paths">
        <div className="settings-about-path">
          <span>日志目录</span>
          <code>{paths?.logDir ?? "…"}</code>
        </div>
        <div className="settings-about-path">
          <span>配置文件</span>
          <code>{paths?.configPath ?? "…"}</code>
        </div>
        <div className="settings-about-path">
          <span>数据目录</span>
          <code>{paths?.dataDir ?? "…"}</code>
        </div>
      </div>

      <p className="settings-about-note">
        推荐离线词库来自{" "}
        <button
          type="button"
          className="settings-about-inline-link"
          onClick={() =>
            void onOpenExternalUrl(
              "https://github.com/skywind3000/ECDICT",
            )
          }
        >
          skywind3000/ECDICT
        </button>
        ，按需下载至 data/dicts/
      </p>
    </section>
  );
}
