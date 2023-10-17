#!/usr/bin/env bash
cargo build
INITIAL_SCRIPT=sample/incrementor.sh rofi -modi "a:./target/debug/rofi-menu-stack" -show a
