/// Generate shell integration script for the specified shell type.
pub fn generate_shell_integration(shell_type: &str) -> String {
    match shell_type {
        "zsh" => generate_zsh_integration(),
        "bash" => generate_bash_integration(),
        "fish" => generate_fish_integration(),
        _ => format!("Unsupported shell type: {shell_type}"),
    }
}

/// Generate Zsh integration script
fn generate_zsh_integration() -> String {
    r#"### key-bindings.zsh ###
#
# - $FF_TMUX_OPTS
# - $FF_CTRL_T_COMMAND
# - $FF_CTRL_T_OPTS
# - $FF_CTRL_R_OPTS
# - $FF_ALT_C_COMMAND
# - $FF_ALT_C_OPTS

if [[ -o interactive ]]; then

# Key bindings
# ------------

__ff_defaults() {
  # $1: Prepend to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  # $2: Append to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  echo -E "--height ${FF_TMUX_HEIGHT:-40%} --min-height 20+ $1"
  command cat "${FF_DEFAULT_OPTS_FILE-}" 2> /dev/null
  echo -E "${FF_DEFAULT_OPTS-} $2"
}

# CTRL-T - Paste the selected file path(s) into the command line
__ff_select() {
  setopt localoptions pipefail no_aliases 2> /dev/null
  local item
  FF_DEFAULT_COMMAND=${FF_CTRL_T_COMMAND:-} \
  FF_DEFAULT_OPTS=$(__ff_defaults "--reverse" "${FF_CTRL_T_OPTS-} -m") \
  FF_DEFAULT_OPTS_FILE='' $(__ffcmd) "$@" < /dev/tty | while read -r item; do
    echo -n -E "${(q)item} "
  done
  local ret=$?
  echo
  return $ret
}

__ffcmd() {
  [ -n "${TMUX_PANE-}" ] && { [ "${FF_TMUX:-0}" != 0 ] || [ -n "${FF_TMUX_OPTS-}" ]; } &&
    echo "ff-tmux ${FF_TMUX_OPTS:--d${FF_TMUX_HEIGHT:-40%}} -- " || echo "ff"
}

ff-file-widget() {
  LBUFFER="${LBUFFER}$(__ff_select)"
  local ret=$?
  zle reset-prompt
  return $ret
}
if [[ "${FF_CTRL_T_COMMAND-x}" != "" ]]; then
  zle     -N            ff-file-widget
  bindkey -M emacs '^T' ff-file-widget
  bindkey -M vicmd '^T' ff-file-widget
  bindkey -M viins '^T' ff-file-widget
fi

# ALT-C - cd into the selected directory
ff-cd-widget() {
  setopt localoptions pipefail no_aliases 2> /dev/null
  local dir="$(
    FF_DEFAULT_COMMAND=${FF_ALT_C_COMMAND:-} \
    FF_DEFAULT_OPTS=$(__ff_defaults "--reverse" "${FF_ALT_C_OPTS-} +m") \
    FF_DEFAULT_OPTS_FILE='' $(__ffcmd) < /dev/tty)"
  if [[ -z "$dir" ]]; then
    zle redisplay
    return 0
  fi
  zle push-line # Clear buffer. Auto-restored on next prompt.
  BUFFER="builtin cd -- ${(q)dir:a}"
  zle accept-line
  local ret=$?
  unset dir # ensure this doesn't end up appearing in prompt expansion
  zle reset-prompt
  return $ret
}
if [[ "${FF_ALT_C_COMMAND-x}" != "" ]]; then
  zle     -N             ff-cd-widget
  bindkey -M emacs '\ec' ff-cd-widget
  bindkey -M vicmd '\ec' ff-cd-widget
  bindkey -M viins '\ec' ff-cd-widget
fi

# CTRL-R - Paste the selected command from history into the command line
ff-history-widget() {
  local selected
  setopt localoptions noglobsubst noposixbuiltins pipefail no_aliases noglob nobash_rematch 2> /dev/null
  selected="$(fc -rl 1 | awk '{ cmd=$0; sub(/^[ \t]*[0-9]+\**[ \t]+/, "", cmd); if (!seen[cmd]++) print $0 }' |
    FF_DEFAULT_OPTS=$(__ff_defaults "" "-n2..,.. --highlight-line ${FF_CTRL_R_OPTS-} +m") \
    FF_DEFAULT_OPTS_FILE='' $(__ffcmd) --query=${(qqq)LBUFFER})"
  local ret=$?
  if [ -n "$selected" ]; then
    if [[ $(awk '{print $1; exit}' <<< "$selected") =~ ^[1-9][0-9]* ]]; then
      zle vi-fetch-history -n $MATCH
    else # selected is a custom query, not from history
      LBUFFER="$selected"
    fi
  fi
  zle reset-prompt
  return $ret
}
zle     -N            ff-history-widget
bindkey -M emacs '^R' ff-history-widget
bindkey -M vicmd '^R' ff-history-widget
bindkey -M viins '^R' ff-history-widget

fi
### end: key-bindings.zsh ###
### completion.zsh ###
#
# - $FF_TMUX                 (default: 0)
# - $FF_TMUX_OPTS            (default: empty)
# - $FF_COMPLETION_TRIGGER   (default: '**')
# - $FF_COMPLETION_OPTS      (default: empty)
# - $FF_COMPLETION_PATH_OPTS (default: empty)
# - $FF_COMPLETION_DIR_OPTS  (default: empty)

if [[ -o interactive ]]; then

__ff_defaults() {
  # $1: Prepend to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  # $2: Append to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  echo -E "--height ${FF_TMUX_HEIGHT:-40%} --min-height 20+ $1"
  command cat "${FF_DEFAULT_OPTS_FILE-}" 2> /dev/null
  echo -E "${FF_DEFAULT_OPTS-} $2"
}

__ff_comprun() {
  if [[ "$(type _ff_comprun 2>&1)" =~ function ]]; then
    _ff_comprun "$@"
  elif [ -n "${TMUX_PANE-}" ] && { [ "${FF_TMUX:-0}" != 0 ] || [ -n "${FF_TMUX_OPTS-}" ]; }; then
    shift
    if [ -n "${FF_TMUX_OPTS-}" ]; then
      ff-tmux ${(Q)${(Z+n+)FF_TMUX_OPTS}} -- "$@"
    else
      ff-tmux -d ${FF_TMUX_HEIGHT:-40%} -- "$@"
    fi
  else
    shift
    ff "$@"
  fi
}

__ff_generic_path_completion() {
  local base lbuf compgen ff_opts suffix tail dir leftover matches
  base=$1
  lbuf=$2
  compgen=$3
  ff_opts=$4
  suffix=$5
  tail=$6

  setopt localoptions nonomatch
  if [[ $base = *'$('* ]] || [[ $base = *'<('* ]] || [[ $base = *'>('* ]] || [[ $base = *':='* ]] || [[ $base = *'`'* ]]; then
    return
  fi
  eval "base=$base" 2> /dev/null || return
  [[ $base = *"/"* ]] && dir="$base"
  while [ 1 ]; do
    if [[ -z "$dir" || -d ${dir} ]]; then
      leftover=${base/#"$dir"}
      leftover=${leftover/#\/}
      [ -z "$dir" ] && dir='.'
      [ "$dir" != "/" ] && dir="${dir/%\//}"
      matches=$(
        export FF_DEFAULT_OPTS
        FF_DEFAULT_OPTS=$(__ff_defaults "--reverse" "${FF_COMPLETION_OPTS-}")
        unset FF_DEFAULT_COMMAND FF_DEFAULT_OPTS_FILE
        if declare -f "$compgen" > /dev/null; then
          eval "$compgen $(printf %q "$dir")" | __ff_comprun "$cmd_word" ${(Q)${(Z+n+)ff_opts}}
        else
          __ff_comprun "$cmd_word" ${(Q)${(Z+n+)ff_opts}} "$dir" < /dev/tty
        fi | while read -r item; do
          item="${item%$suffix}$suffix"
          echo -n -E "${(q)item} "
        done
      )
      matches=${matches% }
      if [ -n "$matches" ]; then
        LBUFFER="$lbuf$matches$tail"
      fi
      zle reset-prompt
      break
    fi
    dir=$(dirname "$dir")
    dir=${dir%/}/
  done
}

_ff_path_completion() {
  __ff_generic_path_completion "$1" "$2" _ff_compgen_path \
    "-m" "" " "
}

_ff_dir_completion() {
  __ff_generic_path_completion "$1" "$2" _ff_compgen_dir \
    "" "/" ""
}

ff-completion() {
  local tokens prefix trigger tail matches lbuf d_cmds cursor_pos cmd_word
  setopt localoptions noshwordsplit noksh_arrays noposixbuiltins

  tokens=(${(z)LBUFFER})
  if [ ${#tokens} -lt 1 ]; then
    zle ${ff_default_completion:-expand-or-complete}
    return
  fi

  trigger=${FF_COMPLETION_TRIGGER-'**'}
  [[ -z $trigger && ${LBUFFER[-1]} == ' ' ]] && tokens+=("")

  if [[ ${LBUFFER} = *"${tokens[-2]-}${tokens[-1]}" ]]; then
    tokens[-2]="${tokens[-2]-}${tokens[-1]}"
    tokens=(${tokens[0,-2]})
  fi

  lbuf=$LBUFFER
  tail=${LBUFFER:$(( ${#LBUFFER} - ${#trigger} ))}

  if [ ${#tokens} -gt 1 -a "$tail" = "$trigger" ]; then
    d_cmds=(${=FF_COMPLETION_DIR_COMMANDS-cd pushd rmdir})

    {
      cursor_pos=$CURSOR
      CURSOR=$((cursor_pos - ${#trigger} - 1))
      if ! zmodload -F zsh/parameter p:functions 2>/dev/null || ! (( ${+functions[compdef]} )); then
        zmodload -F zsh/compctl 2>/dev/null
      fi
      zle -C __ff_extract_command .complete-word __ff_extract_command
      zle __ff_extract_command
    } always {
      CURSOR=$cursor_pos
      zle -D __ff_extract_command  2>/dev/null
    }

    [ -z "$trigger"      ] && prefix=${tokens[-1]} || prefix=${tokens[-1]:0:-${#trigger}}
    if [[ $prefix = *'$('* ]] || [[ $prefix = *'<('* ]] || [[ $prefix = *'>('* ]] || [[ $prefix = *':='* ]] || [[ $prefix = *'`'* ]]; then
      return
    fi
    [ -n "${tokens[-1]}" ] && lbuf=${lbuf:0:-${#tokens[-1]}}

    if eval "noglob type _ff_complete_${cmd_word} >/dev/null"; then
      prefix="$prefix" eval _ff_complete_${cmd_word} ${(q)lbuf}
      zle reset-prompt
    elif [ ${d_cmds[(i)$cmd_word]} -le ${#d_cmds} ]; then
      _ff_dir_completion "$prefix" "$lbuf"
    else
      _ff_path_completion "$prefix" "$lbuf"
    fi
  else
    zle ${ff_default_completion:-expand-or-complete}
  fi
}

[ -z "$ff_default_completion" ] && {
  binding=$(bindkey '^I')
  [[ $binding =~ 'undefined-key' ]] || ff_default_completion=$binding[(s: :w)2]
  unset binding
}

zle     -N   ff-completion
bindkey '^I' ff-completion

fi
### end: completion.zsh ###"#.to_string()
}

/// Generate Bash integration script
fn generate_bash_integration() -> String {
    r#"### key-bindings.bash ###
#
# - $FF_TMUX_OPTS
# - $FF_CTRL_T_COMMAND
# - $FF_CTRL_T_OPTS
# - $FF_CTRL_R_OPTS
# - $FF_ALT_C_COMMAND
# - $FF_ALT_C_OPTS

if [[ $- =~ i ]]; then

# Key bindings
# ------------

__ff_defaults() {
  # $1: Prepend to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  # $2: Append to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  echo "--height ${FF_TMUX_HEIGHT:-40%} --min-height 20+ $1"
  command cat "${FF_DEFAULT_OPTS_FILE-}" 2> /dev/null
  echo "${FF_DEFAULT_OPTS-} $2"
}

__ff_select__() {
  FF_DEFAULT_COMMAND=${FF_CTRL_T_COMMAND:-} \
  FF_DEFAULT_OPTS=$(__ff_defaults "--reverse" "${FF_CTRL_T_OPTS-} -m") \
  FF_DEFAULT_OPTS_FILE='' $(__ffcmd) "$@" |
    while read -r item; do
      printf '%q ' "$item"  # escape special chars
    done
}

__ffcmd() {
  [[ -n "${TMUX_PANE-}" ]] && { [[ "${FF_TMUX:-0}" != 0 ]] || [[ -n "${FF_TMUX_OPTS-}" ]]; } &&
    echo "ff-tmux ${FF_TMUX_OPTS:--d${FF_TMUX_HEIGHT:-40%}} -- " || echo "ff"
}

ff-file-widget() {
  local selected="$(__ff_select__ "$@")"
  READLINE_LINE="${READLINE_LINE:0:$READLINE_POINT}$selected${READLINE_LINE:$READLINE_POINT}"
  READLINE_POINT=$(( READLINE_POINT + ${#selected} ))
}

__ff_cd__() {
  local dir
  dir=$(
    FF_DEFAULT_COMMAND=${FF_ALT_C_COMMAND:-} \
    FF_DEFAULT_OPTS=$(__ff_defaults "--reverse" "${FF_ALT_C_OPTS-} +m") \
    FF_DEFAULT_OPTS_FILE='' $(__ffcmd)
  ) && printf 'builtin cd -- %q' "$(builtin unset CDPATH && builtin cd -- "$dir" && builtin pwd)"
}

__ff_history__() {
  local output script
  script='BEGIN { getc; $/ = "\n\t"; $HISTCOUNT = $ENV{last_hist} + 1 } s/^[ *]//; s/\n/\n\t/gm; print $HISTCOUNT - $. . "\t$_" if !$seen{$_}++'
  output=$(
    set +o pipefail
    builtin fc -lnr -2147483648 |
      last_hist=$(HISTTIMEFORMAT='' builtin history 1) command perl -n -l0 -e "$script" |
      FF_DEFAULT_OPTS=$(__ff_defaults "" "-n2..,.. --highlight-line ${FF_CTRL_R_OPTS-} +m") \
      FF_DEFAULT_OPTS_FILE='' $(__ffcmd) --query "$READLINE_LINE"
  ) || return
  READLINE_LINE=$(command perl -pe 's/^\d*\t//' <<< "$output")
  if [[ -z "$READLINE_POINT" ]]; then
    echo "$READLINE_LINE"
  else
    READLINE_POINT=0x7fffffff
  fi
}

# Required to refresh the prompt after ff
bind -m emacs-standard '"\er": redraw-current-line'

bind -m vi-command '"\C-z": emacs-editing-mode'
bind -m vi-insert '"\C-z": emacs-editing-mode'
bind -m emacs-standard '"\C-z": vi-editing-mode'

if (( BASH_VERSINFO[0] < 4 )); then
  # CTRL-T - Paste the selected file path into the command line
  if [[ "${FF_CTRL_T_COMMAND-x}" != "" ]]; then
    bind -m emacs-standard '"\C-t": " \C-b\C-k \C-u`__ff_select__`\e\C-e\er\C-a\C-y\C-h\C-e\e \C-y\ey\C-x\C-x\C-f"'
    bind -m vi-command '"\C-t": "\C-z\C-t\C-z"'
    bind -m vi-insert '"\C-t": "\C-z\C-t\C-z"'
  fi

  # CTRL-R - Paste the selected command from history into the command line
  bind -m emacs-standard '"\C-r": "\C-e \C-u\C-y\ey\C-u`__ff_history__`\e\C-e\er"'
  bind -m vi-command '"\C-r": "\C-z\C-r\C-z"'
  bind -m vi-insert '"\C-r": "\C-z\C-r\C-z"'
else
  # CTRL-T - Paste the selected file path into the command line
  if [[ "${FF_CTRL_T_COMMAND-x}" != "" ]]; then
    bind -m emacs-standard -x '"\C-t": ff-file-widget'
    bind -m vi-command -x '"\C-t": ff-file-widget'
    bind -m vi-insert -x '"\C-t": ff-file-widget'
  fi

  # CTRL-R - Paste the selected command from history into the command line
  bind -m emacs-standard -x '"\C-r": __ff_history__'
  bind -m vi-command -x '"\C-r": __ff_history__'
  bind -m vi-insert -x '"\C-r": __ff_history__'
fi

# ALT-C - cd into the selected directory
if [[ "${FF_ALT_C_COMMAND-x}" != "" ]]; then
  bind -m emacs-standard '"\ec": " \C-b\C-k \C-u`__ff_cd__`\e\C-e\er\C-m\C-y\C-h\e \C-y\ey\C-x\C-x\C-d"'
  bind -m vi-command '"\ec": "\C-z\ec\C-z"'
  bind -m vi-insert '"\ec": "\C-z\ec\C-z"'
fi

fi
### end: key-bindings.bash ###
### completion.bash ###
#
# - $FF_TMUX                 (default: 0)
# - $FF_TMUX_OPTS            (default: empty)
# - $FF_COMPLETION_TRIGGER   (default: '**')
# - $FF_COMPLETION_OPTS      (default: empty)
# - $FF_COMPLETION_PATH_OPTS (default: empty)
# - $FF_COMPLETION_DIR_OPTS  (default: empty)

if [[ $- =~ i ]]; then

# To redraw line after ff closes (printf '\e[5n')
bind '"\e[0n": redraw-current-line' 2> /dev/null

__ff_defaults() {
  # $1: Prepend to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  # $2: Append to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  echo "--height ${FF_TMUX_HEIGHT:-40%} --min-height 20+ $1"
  command cat "${FF_DEFAULT_OPTS_FILE-}" 2> /dev/null
  echo "${FF_DEFAULT_OPTS-} $2"
}

__ff_comprun() {
  if [[ "$(type -t _ff_comprun 2>&1)" = function ]]; then
    _ff_comprun "$@"
  elif [[ -n "${TMUX_PANE-}" ]] && { [[ "${FF_TMUX:-0}" != 0 ]] || [[ -n "${FF_TMUX_OPTS-}" ]]; }; then
    shift
    ff-tmux ${FF_TMUX_OPTS:--d${FF_TMUX_HEIGHT:-40%}} -- "$@"
  else
    shift
    ff "$@"
  fi
}

__ff_generic_path_completion() {
  local cur base dir leftover matches trigger cmd
  cmd="${COMP_WORDS[0]}"
  if [[ $cmd == \\* ]]; then
    cmd="${cmd:1}"
  fi
  COMPREPLY=()
  trigger=${FF_COMPLETION_TRIGGER-'**'}
  [[ $COMP_CWORD -ge 0 ]] && cur="${COMP_WORDS[COMP_CWORD]}"
  if [[ "$cur" == *"$trigger" ]] && [[ $cur != *'$('* ]] && [[ $cur != *':='* ]] && [[ $cur != *'`'* ]]; then
    base=${cur:0:${#cur}-${#trigger}}
    eval "base=$base" 2> /dev/null || return

    dir=
    [[ $base = *"/"* ]] && dir="$base"
    while true; do
      if [[ -z "$dir" ]] || [[ -d "$dir" ]]; then
        leftover=${base/#"$dir"}
        leftover=${leftover/#\/}
        [[ -z "$dir" ]] && dir='.'
        [[ "$dir" != "/" ]] && dir="${dir/%\//}"
        matches=$(
          export FF_DEFAULT_OPTS=$(__ff_defaults "--reverse" "${FF_COMPLETION_OPTS-} $2")
          unset FF_DEFAULT_COMMAND FF_DEFAULT_OPTS_FILE
          if declare -F "$1" > /dev/null; then
            eval "$1 $(printf %q "$dir")" | __ff_comprun "$4"
          else
            __ff_comprun "$4" "$dir"
          fi | while read -r item; do
            printf "%q " "${item%$3}$3"
          done
        )
        matches=${matches% }
        [[ -z "$3" ]] && [[ "${__ff_nospace_commands-}" = *" ${COMP_WORDS[0]} "* ]] && matches="$matches "
        if [[ -n "$matches" ]]; then
          COMPREPLY=( "$matches" )
        else
          COMPREPLY=( "$cur" )
        fi
        printf '\e[5n'
        return 0
      fi
      dir=$(command dirname "$dir")
      [[ "$dir" =~ /$ ]] || dir="$dir"/
    done
  else
    shift
    shift
    shift
    _ff_handle_dynamic_completion "$cmd" "$@"
  fi
}

_ff_path_completion() {
  __ff_generic_path_completion _ff_compgen_path "-m" "" "$@"
}

_ff_dir_completion() {
  __ff_generic_path_completion _ff_compgen_dir "" "/" "$@"
}

ff-completion() {
  local cur selected trigger cmd post
  post="$(caller 0 | command awk '{print $2}')_post"
  type -t "$post" > /dev/null 2>&1 || post='command cat'

  trigger=${FF_COMPLETION_TRIGGER-'**'}
  cmd="${COMP_WORDS[0]}"
  cur="${COMP_WORDS[COMP_CWORD]}"
  if [[ "$cur" == *"$trigger" ]] && [[ $cur != *'$('* ]] && [[ $cur != *':='* ]] && [[ $cur != *'`'* ]]; then
    cur=${cur:0:${#cur}-${#trigger}}

    selected=$(
      FF_DEFAULT_OPTS=$(__ff_defaults "--reverse" "${FF_COMPLETION_OPTS-} $str_arg") \
      FF_DEFAULT_OPTS_FILE='' \
        __ff_comprun "${rest[0]}" "${args[@]}" | eval "$post" | command tr '\n' ' ')
    selected=${selected% } # Strip trailing space not to repeat "-o nospace"
    if [[ -n "$selected" ]]; then
      COMPREPLY=("$selected")
    else
      COMPREPLY=("$cur")
    fi
    printf '\e[5n'
    return 0
  else
    _ff_handle_dynamic_completion "$cmd" "${rest[@]}"
  fi
}

# Default path completion
__ff_default_completion() {
  __ff_generic_path_completion _ff_compgen_path "-m" "" "$@"

  # Dynamic completion loader has updated the completion for the command
  if [[ $? -eq 124 ]]; then
    _ff_setup_completion path "$1"
    return 124
  fi
}

# Set fuzzy path completion as the default completion for all commands.
complete | command grep -q __ff_default_completion ||
  complete | command grep -- '-D$' | command grep -qv _comp_complete_load ||
  complete -D -F __ff_default_completion -o default -o bashdefault 2> /dev/null

fi
### end: completion.bash ###"#.to_string()
}

/// Generate Fish integration script
fn generate_fish_integration() -> String {
    r#"### key-bindings.fish ###
#
# - $FF_TMUX_OPTS
# - $FF_CTRL_T_COMMAND
# - $FF_CTRL_T_OPTS
# - $FF_CTRL_R_OPTS
# - $FF_ALT_C_COMMAND
# - $FF_ALT_C_OPTS

if status is-interactive

# Key bindings
# ------------

function __ff_defaults
  # $1: Prepend to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  # $2: Append to FF_DEFAULT_OPTS_FILE and FF_DEFAULT_OPTS
  echo "--height $FF_TMUX_HEIGHT --min-height 20+ $argv[1]"
  if test -n "$FF_DEFAULT_OPTS_FILE"
    cat "$FF_DEFAULT_OPTS_FILE" 2> /dev/null
  end
  echo "$FF_DEFAULT_OPTS $argv[2]"
end

function __ffcmd
  if test -n "$TMUX_PANE"
    if test "$FF_TMUX" != "0"; or test -n "$FF_TMUX_OPTS"
      echo "ff-tmux $FF_TMUX_OPTS -- "
    else
      echo "ff"
    end
  else
    echo "ff"
  end
end

# CTRL-T - Paste the selected file path(s) into the command line
function __ff_select
  set -l cmd (__ffcmd)
  if test -n "$FF_CTRL_T_COMMAND"
    set -l FF_DEFAULT_COMMAND $FF_CTRL_T_COMMAND
  end
  set -l FF_DEFAULT_OPTS (__ff_defaults "--reverse" "$FF_CTRL_T_OPTS -m")
  set -l FF_DEFAULT_OPTS_FILE ""
  $cmd $argv | while read -l item
    printf '%s ' (string escape -- $item)
  end
end

function ff-file-widget
  set -l selected (__ff_select)
  commandline -i -- $selected
  commandline -f repaint
end

# CTRL-R - Paste the selected command from history into the command line
function __ff_history
  set -l cmd (__ffcmd)
  set -l FF_DEFAULT_OPTS (__ff_defaults "" "-n2..,.. --highlight-line $FF_CTRL_R_OPTS +m")
  set -l FF_DEFAULT_OPTS_FILE ""
  set -l selected (history | $cmd --query (commandline))
  if test -n "$selected"
    commandline -r -- $selected
  end
  commandline -f repaint
end

# ALT-C - cd into the selected directory
function __ff_cd
  set -l cmd (__ffcmd)
  if test -n "$FF_ALT_C_COMMAND"
    set -l FF_DEFAULT_COMMAND $FF_ALT_C_COMMAND
  end
  set -l FF_DEFAULT_OPTS (__ff_defaults "--reverse" "$FF_ALT_C_OPTS +m")
  set -l FF_DEFAULT_OPTS_FILE ""
  set -l dir ($cmd)
  if test -n "$dir"
    cd "$dir"
    commandline -f repaint
  end
end

# Bind keys
if test "$FF_CTRL_T_COMMAND" != ""
  bind \ct ff-file-widget
end

bind \cr __ff_history
bind \ec __ff_cd

end
### end: key-bindings.fish ###"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_shell_integration_zsh() {
        let result = generate_shell_integration("zsh");
        assert!(result.contains("### key-bindings.zsh ###"));
        assert!(result.contains("ff-file-widget"));
        assert!(result.contains("ff-cd-widget"));
        assert!(result.contains("ff-history-widget"));
    }

    #[test]
    fn test_generate_shell_integration_bash() {
        let result = generate_shell_integration("bash");
        assert!(result.contains("### key-bindings.bash ###"));
        assert!(result.contains("ff-file-widget"));
        assert!(result.contains("__ff_cd__"));
        assert!(result.contains("__ff_history__"));
    }

    #[test]
    fn test_generate_shell_integration_fish() {
        let result = generate_shell_integration("fish");
        assert!(result.contains("### key-bindings.fish ###"));
        assert!(result.contains("ff-file-widget"));
        assert!(result.contains("__ff_cd"));
        assert!(result.contains("__ff_history"));
    }

    #[test]
    fn test_generate_shell_integration_unsupported() {
        let result = generate_shell_integration("unsupported");
        assert!(result.contains("Unsupported shell type"));
    }
}
