# Windows installer policy

`installer.nsi` is based on the official Tauri CLI **2.11.4** template
(MIT OR Apache-2.0). Its source URL is recorded at the top of the file.
When upgrading the Tauri CLI, compare against its template before updating this copy.

Local changes:

- Skip the maintenance page for existing NSIS installations and overwrite app files.
- Guard `PageLeaveReinstall` so it cannot invoke an old NSIS uninstaller,
  including during passive installation. Keep the MSI migration flow separate.
- Keep the upstream downgrade restriction when `allowDowngrades` is false.
- Use `hooks.nsh` for informational logging only. Never delete or copy portable
  `data/`, and never use a recursive removal of `$INSTDIR`.

The runtime loads an existing `data/config.toml` without rewriting it. Only a
missing config file is initialized. Build the NSIS package with
`npm run tauri:build:nsis` from the repository root.
