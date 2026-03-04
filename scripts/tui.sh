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
GRAY='\033[38;5;245m'

# ── Terminal Dimensions ───────────────────────────────────────
term_width()  { tput cols  2>/dev/null || echo 80; }
term_height() { tput lines 2>/dev/null || echo 24; }

# ── Alternate Screen ──────────────────────────────────────────
enter_alt_screen() { tput smcup 2>/dev/null || true; }
leave_alt_screen() { tput rmcup 2>/dev/null || true; }

clear_screen() { printf '\033[2J\033[H'; }
hide_cursor()  { printf '\033[?25l'; }
show_cursor()  { printf '\033[?25h'; }

# ── Logo text ─────────────────────────────────────────────────
JOTMATE_LOGO='     ██╗ ██████╗ ████████╗███╗   ███╗ █████╗ ████████╗███████╗
     ██║██╔═══██╗╚══██╔══╝████╗ ████║██╔══██╗╚══██╔══╝██╔════╝
     ██║██║   ██║   ██║   ██╔████╔██║███████║   ██║   █████╗
██   ██║██║   ██║   ██║   ██║╚██╔╝██║██╔══██║   ██║   ██╔══╝
╚█████╔╝╚██████╔╝   ██║   ██║ ╚═╝ ██║██║  ██║   ██║   ███████╗
 ╚════╝  ╚═════╝    ╚═╝   ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚══════╝'

JOTMATE_LOGO_SMALL='╦╔═╗╔╦╗╔╦╗╔═╗╔╦╗╔═╗
║║ ║ ║ ║║║╠═╣ ║ ║╣
╚╝╚═╝ ╩ ╩ ╩╩ ╩ ╩ ╚═╝'

# ── Icon (embedded ANSI art, 7 lines × 14 visible chars) ──────
# Each line is the raw ANSI string for one row of the icon.
ICON_LINE_0=$'\033[0m\033[38;5;102;48;5;232m\u2597\033[38;5;244;48;5;16m\u2585\033[38;5;243m\u2585\033[38;5;244m\u2583     \033[38;5;235m\u2595\033[38;5;243;48;5;232m\u2583\033[48;5;16m\u2585\u2585\033[38;5;246m\u2596\033[0m'
ICON_LINE_1=$'\033[38;5;16;48;5;244m\u258d\033[38;5;233;48;5;173m\u258e\033[38;5;173;48;5;167m\u259e\033[48;5;237m\u2586\033[38;5;179;48;5;241m\u2583\033[38;5;16;48;5;137m\u2594\033[38;5;179;48;5;243m\u2584\u2584\033[38;5;16;48;5;102m\u2594\033[38;5;179;48;5;242m\u2583\033[38;5;173;48;5;238m\u2586\033[38;5;167;48;5;173m\u2583\033[38;5;173;48;5;233m\u258b\033[38;5;145;48;5;16m\u258c\033[0m'
ICON_LINE_2=$'\033[38;5;16;48;5;244m\u258d\033[38;5;237;48;5;173m\u258c\033[38;5;179m\u2582\033[38;5;180m\u2584\033[38;5;179;48;5;180m\u258c\033[48;5;150m\u258a\033[38;5;180;48;5;179m\u258e\033[38;5;179;48;5;150m\u259e\033[38;5;180;48;5;215m\u258c\033[38;5;150;48;5;179m\u2598\033[38;5;180;48;5;173m\u2581\033[38;5;215;48;5;167m\u2583\033[38;5;179;48;5;238m\u2598\033[38;5;246;48;5;16m\u258c\033[0m'
ICON_LINE_3=$'\033[38;5;246;48;5;236m\u2597\033[38;5;179;48;5;239m\u2597\033[38;5;180;48;5;215m\u2583\033[38;5;23;48;5;108m\u2597\033[38;5;116;48;5;238m\u2594\033[38;5;23;48;5;109m\u2582\033[38;5;151;48;5;215m\u258d\u2595\033[38;5;66;48;5;109m\u259e\033[48;5;73m\u258b\033[48;5;72m\u258e\033[38;5;151;48;5;179m\u258f\033[38;5;138;48;5;236m\u259e\033[38;5;247;48;5;233m\u2596\033[0m'
ICON_LINE_4=$'\033[38;5;235;48;5;244m\u2596\033[38;5;239;48;5;137m\u258d\033[38;5;179;48;5;173m\u2598\033[38;5;109m\u2594\033[38;5;179;48;5;239m\u2586\033[48;5;66m\u2586\033[48;5;173m\u2594\u2594\033[38;5;66;48;5;179m\u2594\033[38;5;179;48;5;235m\u2586\033[38;5;66;48;5;173m\u2594\033[38;5;137m\u2595\033[48;5;237m\u258b\033[38;5;247;48;5;59m\u2584\033[0m'
ICON_LINE_5=$'\033[38;5;232;48;5;145m\u258a\033[38;5;244;48;5;237m\u258d\033[38;5;173;48;5;239m\u259d\033[38;5;237;48;5;179m\u2583\033[38;5;95m\u2582\033[38;5;172;48;5;215m\u2581\u2582\u2582\u2582\033[38;5;95;48;5;180m\u2582\033[38;5;236;48;5;179m\u2583\033[38;5;173;48;5;8m\u2598\033[38;5;243;48;5;237m\u259d\033[38;5;102;48;5;234m\u258c\033[0m'
ICON_LINE_6=$'\033[38;5;240;48;5;233m\u2595\033[38;5;244;48;5;59m\u258e\033[38;5;53;48;5;60m\u259d\u2598\033[38;5;236;48;5;54m\u2594\033[38;5;8;48;5;239m\u2584\033[38;5;239;48;5;130m\u2586\u2585\033[38;5;236;48;5;240m\u2597\033[38;5;234;48;5;54m\u2594\033[38;5;60m\u2595\033[38;5;238;48;5;60m\u2598\033[38;5;246;48;5;237m\u259d\033[38;5;247;48;5;235m\u2596\033[0m'

# ── Logo rendering ─────────────────────────────────────────────
render_main_logo() {
    local tw="$1"

    # Capture logo lines via gum (text color, no ANSI), then zip with icon
    local logo_lines=()
    while IFS= read -r line; do
        logo_lines+=("$line")
    done < <(gum style --foreground "$C_TEXT" --bold "$JOTMATE_LOGO")

    local icon_lines=(
        "$ICON_LINE_0"
        "$ICON_LINE_1"
        "$ICON_LINE_2"
        "$ICON_LINE_3"
        "$ICON_LINE_4"
        "$ICON_LINE_5"
        "$ICON_LINE_6"
    )

    # 7 icon lines, 6 logo lines — offset logo by 1 for vertical centering
    local logo_offset=1
    local icon_count=${#icon_lines[@]}
    local logo_count=${#logo_lines[@]}
    local max=$(( icon_count > (logo_count + logo_offset) ? icon_count : (logo_count + logo_offset) ))

    # icon is 14 visible chars wide; add 2-char gap → column width 16
    local icon_col=16
    # logo visible width: strip ANSI from first line
    local logo_vis
    logo_vis="$(printf '%s' "${logo_lines[0]:-}" | sed 's/\x1b\[[^a-zA-Z]*[a-zA-Z]//g')"
    local logo_w=${#logo_vis}
    local combined=$(( icon_col + logo_w ))
    local pad=$(( tw > combined ? (tw - combined) / 2 : 0 ))
    local indent
    printf -v indent '%*s' "$pad" ''

    local output=''
    local i
    for (( i=0; i<max; i++ )); do
        local icon_part="${icon_lines[$i]:-}"
        local logo_idx=$(( i - logo_offset ))
        local logo_part=''
        [[ $logo_idx -ge 0 && $logo_idx -lt $logo_count ]] && logo_part="${logo_lines[$logo_idx]}"
        # measure visible width of icon part (strip ANSI)
        local vis
        vis="$(printf '%s' "$icon_part" | sed 's/\x1b\[[^a-zA-Z]*[a-zA-Z]//g')"
        local vis_len=${#vis}
        local spaces=$(( icon_col - vis_len ))
        [[ $spaces -lt 0 ]] && spaces=0
        local gap
        printf -v gap '%*s' "$spaces" ''
        output+="${indent}${icon_part}${gap}${logo_part}"$'\n'
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
    show_tool_header "Sync" "Sync repos to upstream"
    "$JOTMATE_BIN" sync
    show_done_screen "Sync" || return 1
}

# ── Run Time ──────────────────────────────────────────────────
run_time() {
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
            "Sync"*)       run_sync     || break ;;
            "Time Doctor"*) run_time    || break ;;
            "Settings"*)   run_settings          ;;
            "Exit"|"")     break                 ;;
        esac
    done
}

main
