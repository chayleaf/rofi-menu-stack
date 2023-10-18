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

# write row
# arg1: row in json5 format
row() {
  if [ -z "$wrote_options" ]; then
    echo "You must write options first!" >&2
    exit 1
  fi
  echo "$@"
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
