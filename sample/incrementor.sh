#!/usr/bin/env bash
. ./lib.sh
a="$1"
b="$(python3 -c "print($a+1)")"
options prompt "$(val "Incrementor >")" message "$(val "Current value: $a")" selection null
row "$(val "Increment")" pop 1 push "$(val "$b")" exec "$(val "sleep 5 && echo $b > value.txt")" fork true
row "$(val "Switch to decrementor")" goto "$(val "sample/decrementor.sh")"
row "$(val "Close")" return null
