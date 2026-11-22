#!/bin/sh
/bin/rm -f -- "$0"
cd "$HOME" || exit 1
@EXPORTS@
export BASH_SILENCE_DEPRECATION_WARNING=1
printf '\033[H\033[2J\033[3J'
exec "${SHELL:-/bin/zsh}" -l
