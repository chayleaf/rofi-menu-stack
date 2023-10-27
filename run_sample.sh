#!/usr/bin/env bash
cargo build || exit 1
export PATH="$PWD/target/debug:$PATH"
INITIAL_STACK='["0","1","0"]' INITIAL_SCRIPT=sample/fib.sh rofi -modi "a:rofi-menu-stack" -show a
