# This channel belongs to this shell session. Terminal output cannot publish readiness.
zmodload zsh/net/socket || return 0
zsocket "$MYTTHY_INPUT_SOCKET" || return 0
typeset -gi _mytthy_fd=$REPLY _mytthy_generation=0 _mytthy_idle=0
typeset -g _mytthy_wake=$MYTTHY_INPUT_WAKE
unset MYTTHY_INPUT_SOCKET MYTTHY_INPUT_WAKE MYTTHY_USER_ZDOTDIR

_mytthy_busy() {
  emulate -L zsh
  _mytthy_idle=0
  print -r -u $_mytthy_fd -- "B $_mytthy_generation" 2>/dev/null || :
  return 0
}
_mytthy_prompt() {
  emulate -L zsh
  if [[ $CONTEXT == start && -z $BUFFER ]]; then
    (( ++_mytthy_generation ))
    _mytthy_idle=1
    print -r -u $_mytthy_fd -- "P $_mytthy_generation" 2>/dev/null || :
  else
    _mytthy_busy
  fi
  return 0
}
_mytthy_accept() {
  emulate -L zsh
  local kind generation path draft
  # A private wake key without an accompanying request must leave ZLE alone.
  if ! IFS=' ' read -r -t 0.1 -u $_mytthy_fd kind generation; then
    return 0
  fi
  path=$_mytthy_shim/draft-$generation
  if [[ $kind != S || $generation != <-> || $generation != $_mytthy_generation ||
        $_mytthy_idle != 1 || $CONTEXT != start || -n $BUFFER || ! -f $path ]]; then
    print -r -u $_mytthy_fd -- "R ${generation:-0}" 2>/dev/null || :
    return 0
  fi
  # NUL delimiters preserve embedded and trailing newlines in the UTF-8 draft.
  IFS= read -r -d '' draft < "$path"
  BUFFER=$draft
  CURSOR=${#BUFFER}
  _mytthy_idle=0
  print -r -u $_mytthy_fd -- "A $generation" 2>/dev/null || :
  zle .accept-line
}
autoload -Uz add-zle-hook-widget add-zsh-hook
add-zle-hook-widget line-init _mytthy_prompt
add-zle-hook-widget line-finish _mytthy_busy
add-zsh-hook preexec _mytthy_busy
zle -N _mytthy_accept
bindkey "$_mytthy_wake" _mytthy_accept
