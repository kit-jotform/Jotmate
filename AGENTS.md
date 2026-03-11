# Jotmate — Agent / Contributor Guide

## Architecture in one sentence

`jotmate` is a Rust binary that handles all **data and business logic**; `scripts/tui.sh` is a `gum`-based bash script that handles all **interactive UI**.

---

## The golden rule

> **All UI must live in `scripts/tui.sh`. The Rust binary must never spawn `gum`, open an editor, or render interactive output.**

`gum choose`, `gum input`, `gum confirm`, and `gum style` require a real TTY. When Rust spawns a subprocess, the child loses the TTY and `gum` fails silently. All interactive UI therefore runs in the bash process that owns the terminal.

---

## Responsibility split

| Layer | File(s) | Does |
|---|---|---|
| UI | `scripts/tui.sh` | All `gum` calls, screen layout, cursor/alt-screen control, menus, prompts, confirmations |
| Data | `src/` (Rust) | Config read/write, sync, time tracking, cache, discovery |
| Bridge | hidden `_*` subcommands | Rust exposes thin data commands the shell calls for reads and writes |

---

## Adding a new tool / screen

1. **Add a bash function** in `tui.sh` following this pattern:

```bash
run_mytool() {
    show_tool_header "My Tool" "One-line description"
    show_cursor

    # gum choose / gum input / etc. here
    # call "$JOTMATE_BIN" <subcommand> for data operations

    show_done_screen "My Tool" || return 1
}
```

2. **Add the menu entry** in `show_main_menu` (the `gum choose` block) and a matching `case` branch in `main`.

3. **Add a Rust subcommand** in `src/cli.rs` if the tool needs to read or write data. Wire it up in `src/main.rs`.

---

## Adding a new settings field

1. Add the field to the appropriate struct in `src/config.rs` with a sensible `#[serde(default)]`.
2. Expose it via `src/tui/mod.rs::settings_get()` as a `key=value` line.
3. Add a toggle/set handler (either extend `settings_toggle` for booleans, or add a new `_settings-*` subcommand for other types).
4. Wire the new subcommand in `src/cli.rs` and `src/main.rs`.
5. Add a menu row in the `run_settings` loop in `tui.sh` and a matching `case` branch.

---

## Hidden `_*` subcommands (data bridge)

These are not shown in `--help` but are called by `tui.sh`:

| Command | Purpose |
|---|---|
| `jotmate _icon` | Print the ANSI icon (used by `render_main_logo`) |
| `jotmate _settings-get` | Print all settings as `key=value` lines |
| `jotmate _settings-toggle <field>` | Toggle a boolean field; prints new value |
| `jotmate _settings-add-repo <url> <name>` | Add an upstream repo |
| `jotmate _settings-remove-repo <name>` | Remove an upstream repo by name |
| `jotmate _settings-toggle-repo <name>` | Toggle `enabled` on a repo; prints new value |

All `_*` commands print minimal output (just the new value or nothing) and exit 0 on success, non-zero on error.

---

## tui.sh conventions

- **`show_tool_header "Name" "Description"`** — clears screen, draws small logo, rounded title box, separator. Call this at the top of every tool function.
- **`show_done_screen "Name"`** — shows "DONE!" and waits for Enter/Esc. Returns 1 on Esc (caller should `return 1` to break the main loop).
- **`enter_alt_screen` / `leave_alt_screen`** — called inside `show_tool_header`; don't call manually unless you have a specific reason.
- **`show_cursor` / `hide_cursor`** — call `show_cursor` before any interactive `gum` prompt so the user can see their input; call `hide_cursor` when done.
- **Colors** — use the `C_*` variables (`C_PRIMARY`, `C_SECONDARY`, `C_ACCENT`, `C_SUCCESS`, `C_MUTED`, `C_TEXT`). Don't hardcode hex values inline.
- **`gum choose` items must be plain text** — do not pass pre-styled (ANSI-escaped) strings as items; `gum` will mangle or drop them. Use `[ON ]` / `[OFF]` text badges instead.
- **`|| choice=""`** — always guard `gum choose` with `|| choice=""` so Esc doesn't kill the script under `set -e`.

---

## Project structure

```
scripts/
  tui.sh          — all interactive UI (gum-based)
  run-sync.sh     — embedded sync script (patched at runtime by Rust)
src/
  main.rs         — CLI entry point, dispatches subcommands
  cli.rs          — clap argument definitions (including hidden _* commands)
  config.rs       — Config struct, load/save, UpstreamRepo
  error.rs        — AppError enum
  sync/
    mod.rs        — sync entry point, resolves repo paths
    cache.rs      — repo path cache (JSON)
    discover.rs   — fd-based git repo discovery
    runner.rs     — patches and runs run-sync.sh
  time/           — TimeDoctor integration
  tui/
    mod.rs        — exec_tui_sh(), run_settings() (no-op), _settings-* data handlers
```

---

## What NOT to put in Rust

- `gum` calls of any kind
- `$EDITOR` invocations for config editing
- Any interactive prompt that requires a TTY
- Screen clearing or cursor control outside of the icon output
