/**
 * Look Translate / translate-api 兼容
 * GET ?text=&source_language=&target_language=&secret=
 * Response: { code, msg, text }
 *
 * Dashboard Variables 配置 SECRET_PASS，并绑定 Workers AI
 */
const LANG = {
    zh: "简体中文汉字",
    en: "English",
    ja: "日本語",
    ko: "한국어",
    fr: "Français",
    de: "Deutsch",
    es: "Español",
  };
  /** 常见套话前缀，整段匹配后剥掉 */
  const CHATTER = [
    /^自动检测语言后[，,].*?(?:翻译[：:]\s*)?/u,
    /^(?:好的|当然|没问题)[，,！!]?\s*/u,
    /^(?:以下是|这是|翻译结果|译文)[：:]\s*/u,
    /^(?:Here is|Here's|Sure[,!]?\s+here(?:'s| is))\s+(?:the\s+)?(?:translation|result)[:：]?\s*/i,
    /^Translation[:：]\s*/i,
  ];
  export default {
    async fetch(request, env) {
      const url = new URL(request.url);
      const text = (url.searchParams.get("text") || "").trim();
      const target = normalize(url.searchParams.get("target_language") || "zh");
      const secret = url.searchParams.get("secret");
      if (secret !== SECRET_PASS) {
        return Response.json({ code: 1, msg: "无权访问" }, { status: 401 });
      }
      if (!text) {
        return Response.json({ code: 1, msg: "缺少 text 参数" }, { status: 400 });
      }
      const to = LANG[target] || target;
      try {
        // 短 prompt + few-shot，比长规则更不容易跑偏
        const response = await env.AI.run("@cf/meta/llama-3.2-3b-instruct", {
          messages: [
            {
              role: "system",
              content:
                `Translate to ${to}. Reply with ONLY the translation. No preamble. Chinese → 汉字 not pinyin.`,
            },
            { role: "user", content: "hello" },
            { role: "assistant", content: target === "zh" ? "你好" : "hello" },
            { role: "user", content: text },
          ],
          max_tokens: 512,
        });
        let out = String(
          response?.response ||
            response?.choices?.[0]?.message?.content ||
            "",
        ).trim();
        out = stripChat(out);
        if (!out) {
          return Response.json({
            code: 2,
            msg: "翻译模型未返回有效结果",
            text: "",
          });
        }
        return Response.json({ code: 0, msg: "ok", text: out });
      } catch (e) {
        return Response.json(
          { code: 3, msg: "服务器内部错误: " + (e?.message || String(e)) },
          { status: 500 },
        );
      }
    },
  };
  function normalize(raw) {
    const s = String(raw).trim().toLowerCase();
    if (s.startsWith("zh")) return "zh";
    return s.slice(0, 2);
  }
  function stripChat(s) {
    let t = s.trim();
    // 去掉包裹引号
    if (
      (t.startsWith('"') && t.endsWith('"')) ||
      (t.startsWith("'") && t.endsWith("'")) ||
      (t.startsWith("「") && t.endsWith("」"))
    ) {
      t = t.slice(1, -1).trim();
    }
    for (const re of CHATTER) {
      t = t.replace(re, "").trim();
    }
    // 多段时：若第一段是套话、后面还有内容，取最后一段非空
    const parts = t.split(/\n+/).map((p) => p.trim()).filter(Boolean);
    if (parts.length > 1) {
      const last = parts[parts.length - 1];
      const firstIsMeta =
        /检测|翻译[：:]|以下是|Here is/i.test(parts[0]) &&
        !/检测|翻译[：:]|以下是|Here is/i.test(last);
      if (firstIsMeta) t = last;
    }
    return t.trim();
  }