# Config-driven engines over plugins

To add translation vendors without shipping a new host build each time, we extend via **Config-driven Engine** profiles in `config.toml`, not a plugin runtime or marketplace.

Builtin Engines (Microsoft, Google, Cloudflare, etc.) stay for开箱即用. New vendors default to named Engine Profiles (`engines.<id>`); we do not add a Builtin module per vendor. `active` selects one engine; Builtin ids win over same-named profiles.

Profiles are edited only in TOML. The settings list shows them read-only (optional `label`, shared placeholder icon) so users can switch. v1 Auth Modes: `none` / `header` / `query` / `bearer` / `basic` — no cloud HMAC (e.g. Tencent TC3). Request Template + Response Extract cover practical REST (placeholders, `extra`, lang map, JSON paths). First documented example: DeepL.

**Rejected**: plugin packages (too heavy for this app), TOML-only invisible profiles (poor discoverability), cloud signatures in v1 (pulls the generic adapter toward a vendor SDK), single global `http` slot (awkward when juggling multiple customs).
