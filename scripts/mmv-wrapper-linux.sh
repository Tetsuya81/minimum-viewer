mmv() {
  local lastdir="${MINIMUM_VIEWER_LAST_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/mmv/lastdir}"

  command mmv "$@"
  local exit_code=$?

  if [ "$exit_code" -eq 0 ] && [ -f "$lastdir" ]; then
    . "$lastdir"
    rm -f "$lastdir"
  fi

  return "$exit_code"
}
