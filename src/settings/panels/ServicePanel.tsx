import { OCR_ENGINES, resolveActives } from "../../shared/options";
import { EngineIcon, IconEdit, OcrEngineIcon } from "../../shared/EngineIcon";
import type { ServicePanelProps } from "./types";

export function ServicePanel({
  config,
  setConfig,
  engineList,
  editingEngine,
  onEngineToggle,
  onSelectOcrEngine,
  onOcrEngineToggle,
  onToggleEngineEditor,
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
                {expanded && engine.value === "deepl" ? (
                  <div className="service-card-editor">
                    <p className="service-card-editor-hint">{engine.hint}</p>
                    <label className="service-field">
                      <span>API Key</span>
                      <input
                        type="password"
                        autoComplete="off"
                        className="settings-input"
                        placeholder="DeepL API Free / Pro 密钥"
                        value={config.engine.deepl_api_key ?? ""}
                        onChange={(event) => setConfig({
                          ...config,
                          engine: { ...config.engine, deepl_api_key: event.target.value },
                        })}
                      />
                    </label>
                    <p className="service-card-editor-hint">
                      以 :fx 结尾的密钥使用 Free 接口，其他密钥使用 Pro 接口。需要 DeepL API 套餐的密钥。
                    </p>
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
                {expanded && engine.value === "baidu" ? (
                  <div className="service-card-editor">
                    <p className="service-card-editor-hint">
                      {engine.hint}
                    </p>
                    <label className="service-field">
                      <span>App ID</span>
                      <input
                        className="settings-input"
                        autoComplete="off"
                        placeholder="百度翻译开放平台 App ID"
                        value={config.engine.baidu_app_id ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              baidu_app_id: event.target.value,
                            },
                          })
                        }
                      />
                    </label>
                    <label className="service-field">
                      <span>密钥</span>
                      <input
                        type="password"
                        autoComplete="off"
                        className="settings-input"
                        placeholder="百度翻译开放平台密钥"
                        value={config.engine.baidu_secret ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              baidu_secret: event.target.value,
                            },
                          })
                        }
                      />
                    </label>
                  </div>
                ) : null}
                {expanded && engine.value === "youdao" ? (
                  <div className="service-card-editor">
                    <p className="service-card-editor-hint">
                      {engine.hint}
                    </p>
                    <label className="service-field">
                      <span>应用 ID</span>
                      <input
                        className="settings-input"
                        autoComplete="off"
                        placeholder="有道智云应用 ID（appKey）"
                        value={config.engine.youdao_app_key ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              youdao_app_key: event.target.value,
                            },
                          })
                        }
                      />
                    </label>
                    <label className="service-field">
                      <span>应用密钥</span>
                      <input
                        type="password"
                        autoComplete="off"
                        className="settings-input"
                        placeholder="有道智云应用密钥"
                        value={config.engine.youdao_app_secret ?? ""}
                        onChange={(event) =>
                          setConfig({
                            ...config,
                            engine: {
                              ...config.engine,
                              youdao_app_secret: event.target.value,
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
          可同时开启多个翻译服务，请至少保留一个服务。
        </p>
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
