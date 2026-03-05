#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════╗
# ║  JOTMATE :: tui.sh                                          ║
# ║  Gum-based interactive TUI                                   ║
# ╚══════════════════════════════════════════════════════════════╝

set -euo pipefail

# ── Resolve jotmate binary (passed as first arg) ──────────────
JOTMATE_BIN="${1:-jotmate}"

# ── Colors ────────────────────────────────────────────────────
C_PRIMARY="#7C3AED"
C_SECONDARY="#06B6D4"
C_SUCCESS="#10B981"
C_MUTED="#6B7280"
C_ACCENT="#F472B6"
C_TEXT="#E5E7EB"

RST='\033[0m'
BOLD='\033[1m'
GREEN='\033[38;5;82m'
PURPLE='\033[38;5;141m'
CYAN='\033[38;5;87m'
GRAY='\033[38;5;245m'
WHITE='\033[38;5;255m'
RED='\033[38;5;196m'

# ── Terminal Dimensions ───────────────────────────────────────
term_width()  { tput cols  2>/dev/null || echo 80; }
term_height() { tput lines 2>/dev/null || echo 24; }

# ── Alternate Screen ──────────────────────────────────────────
enter_alt_screen() { tput smcup 2>/dev/null || true; }
leave_alt_screen() { tput rmcup 2>/dev/null || true; }

clear_screen() { printf '\033[2J\033[H'; }
hide_cursor()  { printf '\033[?25l'; }
show_cursor()  { printf '\033[?25h'; }

# ── Logo ──────────────────────────────────────────────────────
JOTMATE_LOGO='     ██╗ ██████╗ ████████╗███╗   ███╗ █████╗ ████████╗███████╗
     ██║██╔═══██╗╚══██╔══╝████╗ ████║██╔══██╗╚══██╔══╝██╔════╝
     ██║██║   ██║   ██║   ██╔████╔██║███████║   ██║   █████╗
██   ██║██║   ██║   ██║   ██║╚██╔╝██║██╔══██║   ██║   ██╔══╝
╚█████╔╝╚██████╔╝   ██║   ██║ ╚═╝ ██║██║  ██║   ██║   ███████╗
 ╚════╝  ╚═════╝    ╚═╝   ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚══════╝'

JOTMATE_LOGO_SMALL='╦╔═╗╔╦╗╔╦╗╔═╗╔╦╗╔═╗
║║ ║ ║ ║║║╠═╣ ║ ║╣
╚╝╚═╝ ╩ ╩ ╩╩ ╩ ╩ ╚═╝'

render_main_logo() {
    local tw="$1"

    # Capture logo first (slower gum call), then icon — both before any output
    local logo_lines=()
    while IFS= read -r line; do
        logo_lines+=("$line")
    done < <(gum style \
        --foreground "$C_TEXT" \
        --bold \
        "$JOTMATE_LOGO")

    local icon_lines=()
    while IFS= read -r line; do
        icon_lines+=("$line")
    done < <("$JOTMATE_BIN" _icon 2>/dev/null \
        | sed 's/\x1b\[?25[lh]//g' \
        | grep -v '^$')

    # icon has 7 lines, logo has 6 — align logo to start at icon line 1 (1 line up)
    local logo_offset=1
    local icon_count=${#icon_lines[@]}
    local logo_count=${#logo_lines[@]}
    local max=$(( icon_count > (logo_count + logo_offset) ? icon_count : (logo_count + logo_offset) ))

    local icon_col_width=16
    local logo_vis_width
    logo_vis_width="$(printf '%s' "${logo_lines[0]:-}" | sed 's/\x1b\[[0-9;]*m//g' | wc -c | tr -d ' ')"
    local combined_width=$(( icon_col_width + logo_vis_width ))
    local pad=$(( (tw - combined_width) / 2 ))
    [[ $pad -lt 0 ]] && pad=0
    local indent
    printf -v indent '%*s' "$pad" ''

    # Build output into a variable first, then print atomically
    local output=''
    local i
    for (( i=0; i<max; i++ )); do
        local icon_part="${icon_lines[$i]:-}"
        local logo_idx=$(( i - logo_offset ))
        local logo_part=''
        [[ $logo_idx -ge 0 && $logo_idx -lt $logo_count ]] && logo_part="${logo_lines[$logo_idx]}"
        local visible
        visible="$(printf '%s' "$icon_part" | sed 's/\x1b\[[0-9;]*m//g')"
        local vis_len=${#visible}
        local spaces=$(( icon_col_width - vis_len ))
        [[ $spaces -lt 0 ]] && spaces=0
        local pad_str
        printf -v pad_str '%*s' "$spaces" ''
        output+="$(printf '%s%s%s%s\n' "$indent" "$icon_part" "$pad_str" "$logo_part")"$'\n'
    done
    printf '%s' "$output"
}

render_small_logo() {
    local tw="$1"
    gum style \
        --foreground "$C_PRIMARY" \
        --bold \
        --align center \
        --width "$tw" \
        "$JOTMATE_LOGO_SMALL"
}

# ── Context bar ───────────────────────────────────────────────
_build_context_line() {
    local version
    version="$("$JOTMATE_BIN" --version 2>/dev/null | awk '{print $NF}' || echo "?")"
    echo "$(date '+%H:%M')  |  v${version}"
}

# ── Tool header ───────────────────────────────────────────────
show_tool_header() {
    local tool_name="$1"
    local tool_desc="${2:-}"
    local tw
    tw="$(term_width)"

    enter_alt_screen
    clear_screen
    echo ""
    render_small_logo "$tw"
    echo ""

    gum style \
        --foreground "$C_ACCENT" \
        --bold \
        --border rounded \
        --border-foreground "$C_PRIMARY" \
        --padding "0 4" \
        --align center \
        --width 50 \
        --margin "0 $(( (tw - 54) / 2 > 0 ? (tw - 54) / 2 : 0 ))" \
        "$tool_name"

    if [[ -n "$tool_desc" ]]; then
        gum style \
            --foreground "$C_MUTED" \
            --italic \
            --align center \
            --width "$tw" \
            "$tool_desc"
    fi

    echo ""
    gum style \
        --foreground "$C_MUTED" \
        --align center \
        --width "$tw" \
        "─────────────────────────────────────────────────"
    echo ""
}

# ── Done screen ───────────────────────────────────────────────
show_done_screen() {
    local tool_name="$1"
    local tw
    tw="$(term_width)"

    show_tool_header "$tool_name" "Completed"
    echo ""
    gum style \
        --foreground "$C_SUCCESS" \
        --bold \
        --align center \
        --width "$tw" \
        "DONE!"
    echo ""
    gum style \
        --foreground "$C_MUTED" \
        --align center \
        --width "$tw" \
        "Enter: main menu   ·   Esc: exit jotmate"

    local key=""
    IFS= read -rsn1 key
    [[ "$key" == $'\e' ]] && return 1
    return 0
}

# ── Graceful exit ─────────────────────────────────────────────
tui_exit() {
    show_cursor
    leave_alt_screen
    echo ""
    echo -e "  ${GRAY}See you later, engineer. Ship it!${RST}"
    echo ""
}

# ── Main menu ─────────────────────────────────────────────────
show_main_menu() {
    local tw
    tw="$(term_width)"

    enter_alt_screen
    clear_screen

    echo ""
    render_main_logo "$tw"

    gum style \
        --foreground "$C_MUTED" \
        --italic \
        --align center \
        --width "$tw" \
        "The lazy engineer's Swiss Army knife"

    echo ""

    gum style \
        --foreground "$C_MUTED" \
        --align center \
        --width "$tw" \
        "$(_build_context_line)"

    echo ""

    gum style \
        --foreground "$C_MUTED" \
        --align center \
        --width "$tw" \
        "─────────────────────────────────────────────────"

    echo ""

    local choice
    choice="$(gum choose \
        --header "    SELECT TOOL  (↑↓ navigate · Enter select · Esc exit)" \
        --header.foreground "$C_SECONDARY" \
        --header.bold \
        --cursor "  ▸ " \
        --cursor.foreground "$C_PRIMARY" \
        --selected.foreground "$C_ACCENT" \
        --selected.bold \
        --height 8 \
        "Sync              ─  Sync repos to upstream" \
        "Time Doctor       ─  Track your work hours" \
        "Settings          ─  Configure jotmate" \
        "Exit")" || choice="Exit"

    MAIN_MENU_CHOICE="$choice"
}

# ── Run Sync ──────────────────────────────────────────────────
run_sync() {
    local tw
    tw="$(term_width)"
    show_tool_header "Sync" "Sync repos to upstream"

    "$JOTMATE_BIN" sync

    show_done_screen "Sync" || return 1
}

# ── Run Time ──────────────────────────────────────────────────
run_time() {
    local tw
    tw="$(term_width)"
    show_tool_header "Time Doctor" "Track your work hours"

    "$JOTMATE_BIN" time

    show_done_screen "Time Doctor" || return 1
}

# ── Run Settings ─────────────────────────────────────────────
run_settings() {
    leave_alt_screen
    show_cursor
    "$JOTMATE_BIN" settings
    hide_cursor
}

# ── Main loop ─────────────────────────────────────────────────
main() {
    hide_cursor
    trap 'tui_exit' EXIT

    while true; do
        show_main_menu
        local choice="${MAIN_MENU_CHOICE:-}"
        case "$choice" in
            "Sync"*)
                run_sync || break
                ;;
            "Time Doctor"*)
                run_time || break
                ;;
            "Settings"*)
                run_settings
                ;;
            "Exit"|"")
                break
                ;;
        esac
    done
}

main
