# This fixture uses a known prompt and single-width, printable ASCII input.
# Keep the caller's startup files out of this disposable prototype.
PROMPT='myt> '
RPROMPT=
unsetopt SINGLE_LINE_ZLE
autoload -Uz add-zle-hook-widget add-zsh-hook
typeset -gi myt_generation=0

myt_hide_input() {
    printf '\e]666;vte.ext.mytthy.zle\e\\'
}

myt_publish_input() {
    local LC_ALL=C
    (( ++myt_generation ))
    if [[ $BUFFER != *[^\ -~]* && -z $PREBUFFER && $CONTEXT == start ]]; then
        printf '\e]666;vte.ext.mytthy.zle=%d %d %d %d\e\\' \
            "$myt_generation" "$CURSOR" "${#BUFFER}" "$COLUMNS"
    else
        myt_hide_input
    fi
}

myt_place_caret() {
    local generation target
    if ! read -r -u "$1" generation target; then
        zle -F "$1"
        return
    fi
    # A click queued before another redraw must not move the new edit buffer.
    if [[ $generation == "$myt_generation" && $target == <-> ]] &&
        (( target <= ${#BUFFER} )); then
        CURSOR=$target
        myt_publish_input
        zle -R
    fi
}

myt_arm_input() {
    zle -F -w 3 myt_place_caret
}

zle -N myt_place_caret
add-zle-hook-widget line-init myt_arm_input
add-zle-hook-widget line-pre-redraw myt_publish_input
add-zle-hook-widget line-finish myt_hide_input
add-zsh-hook preexec myt_hide_input
