# Source user startup files with their original ZDOTDIR, then return to our shim.
typeset -g _mytthy_shim=$ZDOTDIR
if [[ -n ${MYTTHY_USER_ZDOTDIR+x} ]]; then
  ZDOTDIR=$MYTTHY_USER_ZDOTDIR
else
  unset ZDOTDIR
fi
[[ -r ${ZDOTDIR:-$HOME}/.zshenv ]] && source ${ZDOTDIR:-$HOME}/.zshenv
typeset -g _mytthy_user_dir=${ZDOTDIR:-$HOME}
typeset -g _mytthy_user_zdotdir_set=${+ZDOTDIR}
ZDOTDIR=$_mytthy_shim
