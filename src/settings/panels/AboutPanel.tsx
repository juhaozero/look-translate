import type { AboutPanelProps } from "./types";
import "./about.css";

export function AboutPanel({ info, paths, onOpenExternalUrl, onOpenLogDir, onOpenConfigFile, onOpenDataDir }: AboutPanelProps) {
  const resources = [
    { label: "日志目录", path: paths?.logDir, action: onOpenLogDir },
    { label: "配置文件", path: paths?.configPath, action: onOpenConfigFile },
    { label: "数据目录", path: paths?.dataDir, action: onOpenDataDir },
  ];

  return (
    <section className="about-page" aria-label="关于 Look Translate">
      <div className="about-cover">
        <div className="about-intro">
          <span className="about-eyebrow">文字之间，自有连接</span>
          <h2>{info?.name ?? "Look Translate"}</h2>
          <p className="about-tagline">重建巴别塔。</p>
          <p className="about-description">{info?.description ?? "Windows 划词翻译小工具"}</p>
          <span className="about-version"><span>VERSION</span>{info?.version ? `v${info.version}` : "读取中…"}</span>
        </div>
        <div className="about-wordmark" aria-hidden="true">
          <div className="about-wordmark-caption">ENGLISH → 中文</div>
          <span className="about-word-en">look</span>
          <span className="about-word-zh">看见</span>
          <span className="about-wordmark-note">让理解，近一点。</span>
        </div>
      </div>
      <div className="about-methods" aria-label="翻译方式">
        <span>划词即译</span><i aria-hidden="true" /><span>截图识别</span><i aria-hidden="true" /><span>输入自动翻译</span>
      </div>
      <nav className="about-project-links" aria-label="项目链接">
        <button type="button" onClick={() => void onOpenExternalUrl("https://github.com/juhaozero/look-translate")}>
          <span className="about-link-index" aria-hidden="true">01</span>
          <span><strong>项目主页</strong><small>在 GitHub 查看源码与版本</small></span>
          <span className="about-link-arrow" aria-hidden="true">↗</span>
        </button>
        <button type="button" onClick={() => void onOpenExternalUrl("https://github.com/juhaozero/look-translate/issues")}>
          <span className="about-link-index" aria-hidden="true">02</span>
          <span><strong>一起改进</strong><small>反馈问题，或分享你的想法</small></span>
          <span className="about-link-arrow" aria-hidden="true">↗</span>
        </button>
      </nav>
      <details className="about-resources">
        <summary><span>本地资源</span><span className="about-resource-hint">日志、配置与数据</span></summary>
        <div className="about-resource-list">
          {resources.map((resource) => (
            <div className="about-resource-row" key={resource.label}>
              <div><span>{resource.label}</span><code>{resource.path ?? "正在读取路径…"}</code></div>
              <button type="button" onClick={() => void resource.action()} aria-label={`打开${resource.label}`}>打开</button>
            </div>
          ))}
        </div>
      </details>
      <footer className="about-footer">
        <span>致谢与词库</span>
        <p>离线词库来自 <button type="button" onClick={() => void onOpenExternalUrl("https://github.com/skywind3000/ECDICT")}>skywind3000 / ECDICT</button>，可在词库设置中按需下载。</p>
      </footer>
    </section>
  );
}
