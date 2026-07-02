# Changelog

All notable changes to this project are documented here.

## [Unreleased]

### Added
- Agent status icons in picker rows (`!`, `✓`, `●`, `○`) for faster scanning.
- `Ctrl-X` closes the selected/open matching workspace without closing the picker; the picker refuses to close its owning workspace.

### Changed
- Empty default picker results now honor agent status priority when `agent_sort = "priority"` or Herdr `agent_panel_sort = "priority"` is active.
- Agent status priority is now: blocked/error/fail, attention/request/wait, done/complete, working/running, idle/unknown.
- Server workspaces now use per-server directories and write `.herdr-server.toml` metadata for `herdr-server-aware`.
- Server SSH commands prefer `autossh` when available and keep SSH alive with explicit keepalive options.
- Picker footer uses compact Ctrl-style key hints.

## [0.2.0] - 2026-06-29

### Added
- Command/JSON plugin integration contract via `[[integrations]]`.
- Herdr success/error notifications for selected actions.
- Agent search by agent name, workspace/session label, cwd/path, status, ids, and aliases.
- Agent shortcut: `@` as a Ctrl-A-style full agent view with configurable sorting.
- Server source from `~/.ssh/config` and manual `[[servers.entries]]`, with `Ctrl-S` filtering and SSH connect inside a server workspace tab.

### Changed

- Agent rows stay tied to the pane start directory while still searching current foreground cwd.
- Herdr Plus logic now lives in a built-in integration adapter.
- Picker entries dispatch through actions instead of hardcoding behavior by source.

## [0.1.2] - 2026-06-28

### Fixed
- Show multiple open workspaces with the same cwd instead of deduping by path.
- Reuse project and directory workspaces by source kind so project and zoxide/root entries do not steal each other.


## [0.1.1] - 2026-06-28

### Added
- Herdr Plus Quick Actions launcher entry.

## [0.1.0] - 2026-06-28

### Added
- Herdr overlay picker plus.
- Sources for open workspaces, Herdr Plus projects, zoxide, configured roots, and agents.
- Configurable source order and search engine.
- Herdr theme-name inheritance with custom token overrides.
- Release and CI workflows for public builds.
