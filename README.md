# Spec

Each menu option can do one of the following:

- Do nothing
- Run a command
- Exit
- Run a command and exit
- Switch the submenu
- Run a command and switch the submenu

A submenu is determined by all the menu choices up to here (prepended to
arg list).

For example, `Settings -> Audio -> Device A -> Volume -> Increase by 1`
doesn't change the submenu (it stays "volume"), but it does run a
command.

On the other hand, `Settings -> Audio -> Device A -> Volume -> Close`
closes the menu and does nothing else.

A different submenu is created with a different script.

So, each entry is defined as follows:

- `text: <string>` - user-facing text
- `icon: <path>` - a path to this option's graphical icon
- `push: [<string?>]` - add a value on top of stack when this option is
  selected (`$1` is top of stack). `null` means add user input.
- `push: [null]` - add the selected option's text on top of the stack
- `pop: null` - remove all values from the top of the stack
- `pop: 1` - remove 1 value from the top of the stack
- `jump: [<string?>]` - switch to a different submenu when this option
  is selected, same format as `push`
- `return: null` - close the submenu
- `return: 1` - go to the previous submenu (same as `pop`)
- `exec: <string>` - bash command to execute
- `fork: true` - don't wait for the bash command's completion and
  run it in the background

# General options

General options are printed before all entries, and are mandatory.

- `prompt: <string>` - user prompt
- `message: <string>` - user-facing message (notice, etc)
- `markup: "pango"` - to enable pango markup
- `selection: 1` - to select item number 1 (0-based)
- `selection: "keep"` - to keep whatever was selected previously
- `fallback: {...}` - this allows the user to input custom text. If the
  user entered it, the `push`, `jump` and `exec`/`fork` operations in
  `fallback` will be executed.
