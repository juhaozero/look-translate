/**
 * Look Translate — Cloudflare Worker (m2m100)
 *
 * 部署前设置密钥（不要写进源码）：
 *   npx wrangler secret put SECRET_PASS
 *
 * 调用协议：
 *   POST /
 *   Authorization: Bearer <SECRET_PASS>
 *   Content-Type: application/json
 *   { "text": "...", "source_language": "en", "target_language": "zh" }
 *
 * 也可用请求头 X-Api-Key: <SECRET_PASS> 代替 Bearer。
 */
export default {
  async fetch(request, env) {
    if (request.method !== "POST") {
      return Response.json({ code: 1, msg: "仅支持 POST" }, { status: 405 });
    }

    const expected = String(env.SECRET_PASS ?? "").trim();
    if (!expected) {
      return Response.json(
        { code: 1, msg: "未配置 SECRET_PASS（请用 wrangler secret put）" },
        { status: 503 },
      );
    }

    const secret = extractSecret(request);
    if (!secret || secret !== expected) {
      return Response.json({ code: 1, msg: "无权访问" }, { status: 401 });
    }

    let text;
    let sourceRaw = "en";
    let targetRaw = "zh";

    try {
      const parsed = await readPostBody(request);
      text = parsed.text;
      sourceRaw = parsed.source_language || "en";
      targetRaw = parsed.target_language || "zh";
    } catch (error) {
      return Response.json(
        { code: 1, msg: "请求体无效: " + (error?.message || String(error)) },
        { status: 400 },
      );
    }

    if (!text || !String(text).trim()) {
      return Response.json({ code: 1, msg: "缺少 text 参数" }, { status: 400 });
    }

    const source_lang = normalizeLang(sourceRaw);
    const target_lang = normalizeLang(targetRaw);

    try {
      const response = await env.AI.run("@cf/meta/m2m100-1.2b", {
        text: String(text).trim(),
        source_lang,
        target_lang,
      });

      const translated = String(response?.translated_text ?? "").trim();
      if (!translated) {
        return Response.json({
          code: 2,
          msg: "翻译模型未返回有效结果",
          text: "",
        });
      }
      if (translated.toUpperCase().startsWith("ERROR")) {
        return Response.json({ code: 2, msg: "ok", text: translated });
      }

      return Response.json({ code: 0, msg: "ok", text: translated });
    } catch (error) {
      console.error(error);
      return Response.json(
        { code: 3, msg: "服务器内部错误: " + (error?.message || String(error)) },
        { status: 500 },
      );
    }
  },
};

/** 仅接受 Authorization: Bearer 或 X-Api-Key。 */
function extractSecret(request) {
  const auth = request.headers.get("Authorization") || "";
  const bearer = auth.match(/^Bearer\s+(.+)$/i);
  if (bearer) {
    return bearer[1].trim();
  }

  const apiKey =
    request.headers.get("X-Api-Key") || request.headers.get("x-api-key");
  if (apiKey) {
    return apiKey.trim();
  }

  return "";
}

async function readPostBody(request) {
  const contentType = (request.headers.get("Content-Type") || "").toLowerCase();
  if (contentType.includes("application/json")) {
    const body = await request.json();
    return {
      text: body?.text,
      source_language: body?.source_language ?? body?.sourceLanguage,
      target_language: body?.target_language ?? body?.targetLanguage,
    };
  }

  if (
    contentType.includes("application/x-www-form-urlencoded") ||
    contentType.includes("multipart/form-data")
  ) {
    const form = await request.formData();
    return {
      text: form.get("text"),
      source_language: form.get("source_language") || form.get("sourceLanguage"),
      target_language: form.get("target_language") || form.get("targetLanguage"),
    };
  }

  const body = await request.json();
  return {
    text: body?.text,
    source_language: body?.source_language ?? body?.sourceLanguage,
    target_language: body?.target_language ?? body?.targetLanguage,
  };
}

function normalizeLang(raw) {
  const s = String(raw || "")
    .trim()
    .toLowerCase();
  if (!s) return "en";
  if (s.startsWith("zh")) return "zh";
  return s.slice(0, 2);
}
