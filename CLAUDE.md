# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

盯盘小助手 — a Tauri 2 desktop app: an always-on-top floating bar showing live A-share quotes (indices + a user watchlist), sourced from Tencent Finance (`qt.gtimg.cn`). Rust backend, no-framework vanilla HTML/CSS/JS frontend. UI strings are Chinese; the MSI bundle ships zh-CN only.

## Commands

```bash
npm install
npm run dev      # tauri dev — first compile takes 3–4 min
npm run build    # tauri build → MSI in src-tauri/target/release/bundle/
npm run icon     # regenerate icons from a source image
```

There is no test suite, linter, or JS build step configured. The frontend is served as static files (`frontendDist: "../src"`), so edits to `src/*.js|html|css` are picked up on reload; edits to `src-tauri/**` trigger a Rust recompile.

## Architecture

Two windows, defined statically in `src-tauri/tauri.conf.json`:
- **main** — the floating bar (`index.html` / `app.js`). Frameless, transparent, always-on-top, auto-sizes its height to content.
- **settings** — watchlist manager (`settings.html` / `settings.js`). Created hidden; its close button is intercepted to hide (not destroy) so it can be reopened.

Frontend ↔ backend go through Tauri commands only (`window.__TAURI__.core.invoke`, enabled by `withGlobalTauri: true`). The five commands are registered in `src-tauri/src/lib.rs`: `fetch_quotes`, `get_config`, `save_config`, `open_settings`, `close_settings`. Adding a command means updating both the `invoke_handler!` list and the capability grants in `src-tauri/capabilities/default.json`.

Data flow:
- `quote.rs` fetches `http://qt.gtimg.cn/q=<codes>`, decodes the **GBK** response to UTF-8 (`encoding_rs`), and parses each `v_<code>="a~b~c~..."` line by **tilde-separated field index** (name=1, price=3, change=31, change_pct=32, etc.). These magic indices are the Tencent wire format — changing the source means remapping them.
- `store.rs` persists `AppConfig` (watchlist, `hidden_from_bar`, `boss_key`) as JSON in the OS app-data dir. Load never fails — a missing/corrupt file falls back to `Default`.
- `save_config` emits a `config-updated` event; the main window listens and re-polls, so settings changes reflect on the bar without a manual refresh.

Cross-cutting behaviors, all in `lib.rs`:
- **Boss key** — a global shortcut (default `Ctrl+`` `, from config) toggles main-window visibility. `parse_boss_key` falls back to `Ctrl+Shift+K` if the configured string is unparseable.
- **System tray** — right-click menu (show/hide, settings, quit) plus left-click-to-toggle.

## Conventions worth knowing

- The three index codes `["sh000001","sz399001","sz399006"]` are hard-coded in **both** `app.js` and `settings.js` — they're always fetched and shown separately from user stocks. Keep the two copies in sync.
- Codes carry a market prefix: `sh` / `sz` + digits (e.g. `sh600000`). The settings UI builds this from a market dropdown + numeric input.
- `hidden_from_bar` hides a code from the floating bar while keeping it in settings; the watchlist is the source of truth for what's fetched.
- Polling cadence is time-aware (`getInterval` in `app.js`): 3s during A-share trading sessions (weekday 09:15–11:30, 13:00–15:00), 60s otherwise. Settings window polls a flat 3s.
