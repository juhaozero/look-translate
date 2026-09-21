# Config-driven engines over plugins

To add translation vendors without shipping a new host build each time, we extend via **Config-driven Engine** profiles in `config.toml`, not a plugin runtime or marketplace.

Builtin Engines (Microsoft, Google, Cloudflare, etc.) stay for开箱即用. New vendors default to named Engine Profiles (`engines.<id>`); we do not add a Builtin module per vendor. `active` selects one engine; Builtin ids win over same-named profiles.

Profiles are edited in TOML. Settings lists them read-only for switching, and offers **open config / reload / insert generic HTTP template / open fill-in docs**. Vendor-specific samples (e.g. DeepL) live in `docs/engine-profiles.md`, not as a forced UI preset. v1 Auth Modes: `none` / `header` / `query` / `bearer` / `basic` — no cloud HMAC (e.g. Tencent TC3).

**Rejected**: plugin packages (too heavy for this app), TOML-only invisible profiles (poor discoverability), cloud signatures in v1 (pulls the generic adapter toward a vendor SDK), single global `http` slot (awkward when juggling multiple customs).

**Qualified by** [0002](./0002-builtin-for-signed-apis.md): when a vendor’s official API cannot be expressed with v1 Auth Modes (request signing), that vendor may ship as a Builtin Engine instead of a Profile.
