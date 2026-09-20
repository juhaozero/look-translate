import { OCR_ENGINES, resolveActives } from "../../shared/options";
import { EngineIcon, IconEdit, OcrEngineIcon } from "../../shared/EngineIcon";
import type { ServicePanelProps } from "./types";

export function ServicePanel({
  config,
  setConfig,
  paths,
  engineList,
  editingEngine,
  profileBusy,
  onEngineToggle,
  onSelectOcrEngine,
  onOcrEngineToggle,
  onToggleEngineEditor,
  onOpenConfigFile,
  onReloadConfigFromDisk,
  onAddCustomTemplate,
}: ServicePanelProps) {
  return (
    <>
      <section className="service-list-shell" aria-label="翻译服务">
        <h2 className="service-section-title">翻译服务</h2>
        <p className="service-section-hint">
          可同时开启多个服务，划词时并行翻译并在浮层分别展示。
        </p>
        <ul className="service-list">
          {engineList.map((engine) => {
            const active = resolveActives(config.engine).includes(
              engine.value,
            );
            const expanded = editingEngine === engine.value;
            return (
              <li
                key={engine.value}
                className={
                  active
                    ? "service-card is-active"
                    : "service-card"
                }
              >
                <div className="service-card-row">
                  <button
                    type="button"
                    className="service-card-main"
                    onClick={() =>
                      onEngineToggle(engine.value, !active)
                    }
                  >
                    <EngineIcon
                      engine={engine.value}
                      className="service-engine-icon"
                    />
                    <span className="service-engine-text">
                      <span className="service-engine-name">
                        {engine.label}
                      </span>
                      <span className="service-engine-sub">
                        {engine.subtitle}
                      </span>
                    </span>
                  </button>
                  <div className="service-card-actions">
                    <button
                      type="button"
                      className={
                        active
                          ? "settings-switch is-on"
                          : "settings-switch"
                      }
                      role="switch"
                      aria-checked={active}
                      aria-label={
                        active
                          ? `关闭 ${engine.label}`
                          : `启用 ${engine.label}`
                      }
                      onClick={() =>
                        onEngineToggle(engine.value, !active)
                      }
                    >
                      <span className="settings-switch-thumb" />
                    </button>
                    {engine.configurable ? (
                      <button
                        type="button"
                        className={
                          expanded
                            ? "service-icon-btn is-active"
                            : "service-icon-btn"
                        }
                        aria-expanded={expanded}
                        aria-label={`配置 ${engine.label}`}
                        onClick={() => onToggleEngineEditor(engine.value)}
                      >
                        <IconEdit className="service-icon-btn-svg" />
                      </button>
                    ) : (
                      <span
                        className="service-icon-btn is-spacer"
                        aria-hidden="true"
                      />
                    )}
                  </div>
                </div>
                {expanded && engine.kind === "profile" ? (
                  <div className="service-card-editor">
                    <p className="service-card-editor-hint">
                      {engine.hint}
                    </p>
                    <div className="service-profile-actions">
                      <button
                        type="button"
                        className="settings-btn settings-btn-ghost"
                        onClick={() => void onOpenConfigFile()}
                      >
                        打开配置文件
                      </button>
                      <button
                        type="button"
                        className="settings-btn settings-btn-ghost"
                        onClick={() => void onReloadConfigFromDisk()}
                      >
                        重新加载配置
                      </button>
                    </div>
                  </div>
                ) : null}
                {expanded && engine.value === "microsoft" ? (
                  <div className="service-card-editor">
                    <p className="service-card-editor-hint">
                      {engine.hint}
                    </p>
                    <label className="service-field">
                      <span>API Key</span>
                      <input
                        type="password"
                        autoComplete="off"
                        className="settings-input"
                        placeholder="Azure Translator 订阅密钥"
                        value={config.engine.microsoft_api_key ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              microsoft_api_key: event.target.value,
                            },
                          })
                        }
                      />
                    </label>
                    <label className="service-field">
                      <span>Region</span>
                      <input
                        className="settings-input"
                        placeholder="如 eastasia；全球资源可留空"
                        value={config.engine.microsoft_region ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              microsoft_region: event.target.value,
                            },
                          })
                        }
                      />
                    </label>
                  </div>
                ) : null}
                {expanded && engine.value === "google" ? (
                  <div className="service-card-editor">
                    <p className="service-card-editor-hint">
                      {engine.hint}
                    </p>
                    <label className="service-field">
                      <span>API Key</span>
                      <input
                        type="password"
                        autoComplete="off"
                        className="settings-input"
                        placeholder="Google Cloud Translation API Key"
                        value={config.engine.google_api_key ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              google_api_key: event.target.value,
                            },
                          })
                        }
                      />
                    </label>
                  </div>
                ) : null}
                {expanded && engine.value === "cloudflare" ? (
                  <div className="service-card-editor">
                    <label className="service-field">
                      <span>Worker 地址</span>
                      <input
                        type="url"
                        autoComplete="off"
                        className="settings-input"
                        placeholder="https://xxx.workers.dev/"
                        value={config.engine.cloudflare_endpoint ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              cloudflare_endpoint: event.target.value,
                            },
                          })
                        }
                      />
                    </label>
                    <label className="service-field">
                      <span>访问密钥</span>
                      <input
                        type="password"
                        autoComplete="off"
                        className="settings-input"
                        placeholder="与 Worker 的 SECRET_PASS 一致（Authorization Bearer）"
                        value={config.engine.cloudflare_secret ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              cloudflare_secret: event.target.value,
                            },
                          })
                        }
                      />
                    </label>
                  </div>
                ) : null}
              </li>
            );
          })}
        </ul>
        <p className="settings-panel-foot service-list-foot">
          同一时间仅一个翻译引擎生效。打开开关即切换当前服务；关闭当前无效。
        </p>
        <div className="service-custom-engines">
          <p className="service-custom-engines-title">自定义引擎（config.toml）</p>
          <p className="service-custom-engines-desc">
            在{" "}
            <code>{paths?.configPath ?? "data/config.toml"}</code>{" "}
            编写 <code>[engines.*]</code>
            。可插入通用 HTTP 骨架，再按文档改成 DeepL 等任意厂商。
          </p>
          <div className="service-profile-actions">
            <button
              type="button"
              className="settings-btn settings-btn-ghost"
              onClick={() => void onOpenConfigFile()}
            >
              打开配置文件
            </button>
            <button
              type="button"
              className="settings-btn settings-btn-ghost"
              onClick={() => void onReloadConfigFromDisk()}
            >
              重新加载配置
            </button>
            {/* <button
              type="button"
              className="settings-btn settings-btn-ghost"
              onClick={() => void openEngineProfilesDoc()}
            >
              填写说明
            </button> */}
            <button
              type="button"
              className="settings-btn"
              disabled={profileBusy}
              onClick={() => void onAddCustomTemplate()}
            >
              {profileBusy ? "处理中…" : "添加通用模板"}
            </button>
          </div>
        </div>
      </section>

      <section className="service-list-shell" aria-label="OCR 识别">
        <h2 className="service-section-title">OCR 识别</h2>
        <ul className="service-list">
          {OCR_ENGINES.map((engine) => {
            const active = config.ocr.engine === engine.value;
            return (
              <li
                key={engine.value}
                className={
                  active ? "service-card is-active" : "service-card"
                }
              >
                <div className="service-card-row">
                  <button
                    type="button"
                    className="service-card-main"
                    onClick={() => onSelectOcrEngine(engine.value)}
                  >
                    <OcrEngineIcon
                      engine={engine.value}
                      className="service-engine-icon"
                    />
                    <span className="service-engine-text">
                      <span className="service-engine-name">
                        {engine.label}
                      </span>
                      <span className="service-engine-sub">
                        {engine.subtitle}
                      </span>
                    </span>
                  </button>
                  <div className="service-card-actions">
                    <button
                      type="button"
                      className={
                        active
                          ? "settings-switch is-on"
                          : "settings-switch"
                      }
                      role="switch"
                      aria-checked={active}
                      aria-label={`将 ${engine.label} 设为当前 OCR`}
                      onClick={() =>
                        onOcrEngineToggle(engine.value, !active)
                      }
                    >
                      <span className="settings-switch-thumb" />
                    </button>
                    <span
                      className="service-icon-btn is-spacer"
                      aria-hidden="true"
                    />
                  </div>
                </div>
                {active ? (
                  <div className="service-card-editor">
                    <p className="service-card-editor-hint">
                      {engine.hint}
                    </p>
                  </div>
                ) : null}
              </li>
            );
          })}
        </ul>
        <p className="settings-panel-foot service-list-foot">
          OCR 与翻译引擎独立配置。Tesseract.js 首次识别会下载 eng+chi_sim
          模型（见{" "}
          <a
            href="https://github.com/naptha/tesseract.js/"
            target="_blank"
            rel="noreferrer"
          >
            tesseract.js
          </a>
          ）。
        </p>
      </section>
    </>
  );
}
