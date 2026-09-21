# Builtin Engines when Auth Mode cannot sign

ADR 0001 keeps new vendors on Config-driven Engine profiles and refuses cloud HMAC in the generic adapter. Baidu and Youdao **official** text APIs need request-body-dependent signatures (MD5 / SHA256) that v1 Auth Modes (`none` / `header` / `query` / `bearer` / `basic`) cannot express.

**Decision**: ship such vendors as **Builtin Engines** (host code + flat credentials on `[engine]`), not as Profiles and not as new Auth Modes. First instances: `baidu` (通用翻译 API) and `youdao` (有道智云文本翻译). Unofficial free web scrapes are out of scope for these two.

**Exception rule** (narrow): add a Builtin **only** when the request cannot be expressed with existing Auth Modes—especially request signing. Do **not** add a Builtin merely because a vendor is popular or domestic. Prefer Profiles whenever header/query/bearer/basic suffice (e.g. DeepL).

**Rejected**: per-vendor Auth Modes in the generic adapter (pulls it toward an SDK); Config-driven Profiles that pretend to sign; webpage/unofficial endpoints as the primary Baidu/Youdao path.
