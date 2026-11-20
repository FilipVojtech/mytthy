if (( _mytthy_user_zdotdir_set )); then
  ZDOTDIR=$_mytthy_user_dir
else
  unset ZDOTDIR
fi
[[ -r $_mytthy_user_dir/.zshrc ]] && source $_mytthy_user_dir/.zshrc
source $_mytthy_shim/integration.zsh
