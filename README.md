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
- `push: <string>` - add a value on top of stack when this option is
  selected (`$1` is top of stack)
- `push: null` - remove a value from the top of the stack when this
  option is selected
- `push: [...]` - same as `push` multiple times in a row for each entry
  in the list
- `jump: <string>` - switch to a different submenu when this option is
  selected
- `jump: null` - close the submenu

# General options

General options are printed before all entries, and are mandatory.

- `prompt: <string>` - user prompt
- `message: <string>` - user-facing message (notice, etc)
- `markup: "pango"` - to enable pango markup
- `selection: 1` - to select item number 1 (0-based)
- `selection: "keep"` - to keep whatever was selected previously
- `fallback: {...}` - this allows the user to input custom text. If the
  user entered it, the `push` and `jump` operations in `fallback` will
  be executed. *Additionally*, `push` may be set to `0` (or a list
  containing `0`) to push the user-provided input value to the stack.
