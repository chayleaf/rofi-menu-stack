#!/usr/bin/env bash
cargo build || exit 1
INITIAL_STACK='["0","1","0"]' INITIAL_SCRIPT=sample/fib.sh rofi -modi "a:./target/debug/rofi-menu-stack" -show a
