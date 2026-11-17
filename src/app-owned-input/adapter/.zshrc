# Load the configured shell before installing the prototype hooks. Keep the
# session history separate from the user's history file.
typeset -gr myt_token=$MYT_TOKEN myt_history=$MYT_HISTORY
typeset -gr myt_submit_key=$'\e[mytthy-'$myt_token'~'
unset MYT_TOKEN MYT_HISTORY
ZDOTDIR=${MYT_ORIGINAL_ZDOTDIR:-$HOME}
unset MYT_ORIGINAL_ZDOTDIR
[[ -r $ZDOTDIR/.zshrc ]] && source "$ZDOTDIR/.zshrc"
HISTFILE=$myt_history
(( HISTSIZE > 0 )) || HISTSIZE=1000
(( SAVEHIST > 0 )) || SAVEHIST=1000
typeset -gi myt_generation=0

myt_hide_editor() {
    printf '\e]666;vte.ext.mytthy.prompt\e\\'
    return 0
}

myt_show_editor() {
    if [[ $CONTEXT == start && -z $BUFFER ]]; then
        printf '\e]666;vte.ext.mytthy.prompt=%s:%d\e\\' "$myt_token" "$(( ++myt_generation ))"
    fi
    return 0
}

myt_submit() {
    local generation encoded command
    # The frontend writes a complete pipe record before sending the bound
    # key. An ordinary or stale key event must never block ZLE.
    zselect -t 0 -r 3 || return 0
    read -r -u 3 generation encoded || return 0
    if [[ $generation == "$myt_generation" && $CONTEXT == start && -z $BUFFER ]]; then
        # Preserve final newlines across command substitution.
        command=$(printf '%b.' "$encoded")
        BUFFER=${command%.}
        CURSOR=${#BUFFER}
        zle .accept-line
    fi
    return 0
}

zmodload zsh/zselect || return
autoload -Uz add-zle-hook-widget add-zsh-hook
zle -N myt_submit
bindkey -M main "$myt_submit_key" myt_submit || return
add-zle-hook-widget line-init myt_show_editor
add-zle-hook-widget line-finish myt_hide_editor
add-zsh-hook preexec myt_hide_editor
