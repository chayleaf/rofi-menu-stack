wrote_options=""

options() {
  if [ -n "$wrote_options" ]; then
    echo "Already wrote options!" >&2
    exit 1
  fi
  wrote_options=1
  echo "$@"
}

row() {
  if [ -z "$wrote_options" ]; then
    echo "You must write options first!" >&2
    exit 1
  fi
  echo "$@"
}

quote() {
  echo -n "$@" | jq -Rs .
}
