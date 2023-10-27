wrote_options=""

# write options
# arg1: options in json5 format
options() {
  if [ -n "$wrote_options" ]; then
    echo "Already wrote options!" >&2
    exit 1
  fi
  wrote_options=1
  echo "$@"
}

# write row (raw json5)
# arg1: row in json5 format
raw() {
  if [ -z "$wrote_options" ]; then
    echo "You must write options first!" >&2
    exit 1
  fi
  echo "$@"
}

_row() {
  echo -n "{text:$(val "$1")"
  shift 1
  while (( "$#" >= 2 )); do
    echo -n ",$(val "$1"):$2"
    shift 2
  done
  echo "}"
}
# write row (first text, then string key-json value pairs)
# example: row "row text" pop 1 push "$(val string_to_push)"
row() {
  raw "$(_row "$@")"
}

# quote a string
# arg1: unquoted string
# returns: quoted json string
# example: val 'a b' returns '"a b "'
val() {
  echo -n "$@" | jq -Rs .
}

# quote multiple strings
# returns: a json list of quoted strings, one string per argument
# example: val a b returns ["a","b"]
vals() {
  jq -cn '$ARGS.positional' --args -- "$@"
}

# convert json5 to json
# arg1: json5
# returns: json
unjson5() {
  rofi-menu-stack unjson5 "$1"
}

call_stack_len() {
  if [[ "$_CALL_STACK_LEN" != "0" ]]; then
    echo -n "$_CALL_STACK_LEN"
  fi
}
