import { IconDict } from "../icons";
import { ToggleRow } from "../ui";
import { describeDictPath, movePathToFront } from "../configUtils";
import type { DictionaryPanelProps } from "./types";

export function DictionaryPanel({
  config,
  setConfig,
  paths,
  installingDict,
  loading,
  onInstallRecommendedDict,
  onPickDictionaryFile,
  onPickDictionaryFolder,
}: DictionaryPanelProps) {
  return (
    <>
      <section className="settings-panel">
        <ToggleRow
          label="启用离线词库"
          checked={config.dictionary.enabled}
          onChange={(checked) =>
            setConfig({
              ...config,
              dictionary: {
                ...config.dictionary,
                enabled: checked,
              },
            })
          }
        />
        <p className="settings-panel-foot">
          仅短词查询：≤30 字符且 ≤3 个词，且无换行。只查列表第一本。
        </p>
      </section>
      <section className="settings-panel settings-panel-stack">
        <label className="settings-stack-label">推荐词典</label>
        <p className="settings-panel-foot">
          ECDICT 简明英汉增强版（mdx，无音标）。安装到{" "}
          <code>
            {paths?.recommendedDictRelative ?? "dicts/ecdict.mdx"}
          </code>
          （相对 data/）。约 70MB，需联网从 GitHub Release 下载。
        </p>
        <div className="settings-inline-actions">
          <button
            type="button"
            className="settings-btn settings-btn-primary"
            disabled={installingDict || loading}
            onClick={() => void onInstallRecommendedDict()}
          >
            {installingDict
              ? "下载安装中…"
              : paths?.recommendedDictPresent
                ? "重新绑定推荐词典"
                : "安装 ECDICT 推荐词典"}
          </button>
        </div>
        {paths?.recommendedDictPresent ? (
          <p className="settings-panel-foot">
            已检测到：<code>{paths.recommendedDictPath}</code>
          </p>
        ) : null}
      </section>
      <section className="settings-panel settings-panel-stack">
        <div className="settings-stack-head">
          <span className="settings-stack-label" id="dict-paths-label">
            词典路径
          </span>
          {config.dictionary.paths.length > 0 ? (
            <span className="settings-stack-meta">
              {config.dictionary.paths.length} 项 · 仅查第一本
            </span>
          ) : null}
        </div>

        {config.dictionary.paths.length === 0 ? (
          <div
            className="dict-path-empty"
            role="status"
            aria-labelledby="dict-paths-label"
          >
            <IconDict className="dict-path-empty-icon" />
            <p>尚未添加词典</p>
            <span>选择 .mdx 文件或文件夹，或安装上方推荐词典</span>
          </div>
        ) : (
          <ul
            className="dict-path-list"
            aria-labelledby="dict-paths-label"
          >
            {config.dictionary.paths.map((path, index) => {
              const kind = describeDictPath(path);
              return (
                <li
                  key={path}
                  className={
                    index === 0
                      ? "dict-path-item is-primary"
                      : "dict-path-item"
                  }
                >
                  <div className="dict-path-item-main">
                    <div className="dict-path-badges">
                      {index === 0 ? (
                        <span className="dict-path-badge is-primary">
                          优先
                        </span>
                      ) : null}
                      <span className="dict-path-badge">{kind.badge}</span>
                    </div>
                    <div className="dict-path-text">
                      <span className="dict-path-name" title={path}>
                        {kind.name}
                      </span>
                      <code className="dict-path-full" title={path}>
                        {path}
                      </code>
                    </div>
                  </div>
                  <div className="dict-path-item-actions">
                    {index > 0 ? (
                      <button
                        type="button"
                        className="settings-btn settings-btn-ghost dict-path-action"
                        onClick={() =>
                          setConfig({
                            ...config,
                            dictionary: {
                              ...config.dictionary,
                              paths: movePathToFront(
                                config.dictionary.paths,
                                path,
                              ),
                            },
                          })
                        }
                      >
                        设为优先
                      </button>
                    ) : null}
                    <button
                      type="button"
                      className="settings-btn settings-btn-ghost dict-path-action"
                      aria-label={`移除 ${kind.name}`}
                      onClick={() =>
                        setConfig({
                          ...config,
                          dictionary: {
                            ...config.dictionary,
                            paths: config.dictionary.paths.filter(
                              (p) => p !== path,
                            ),
                          },
                        })
                      }
                    >
                      移除
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        )}

        <div className="settings-inline-actions">
          <button
            type="button"
            className="settings-btn"
            onClick={() => void onPickDictionaryFile()}
          >
            选择 .mdx
          </button>
          <button
            type="button"
            className="settings-btn"
            onClick={() => void onPickDictionaryFolder()}
          >
            选择文件夹
          </button>
          {config.dictionary.paths.length > 0 ? (
            <button
              type="button"
              className="settings-btn settings-btn-ghost"
              onClick={() =>
                setConfig({
                  ...config,
                  dictionary: { ...config.dictionary, paths: [] },
                })
              }
            >
              全部清空
            </button>
          ) : null}
        </div>
        <p className="settings-panel-foot">
          相对路径相对安装目录旁的 <code>data/</code>
          。释义纯文本，不加载 <code>.mdd</code>。
        </p>
      </section>
    </>
  );
}
