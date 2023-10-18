# rofi-menu-stack

This is a stack machine for writing complex menus in `rofi{,-wayland}`.
Technically it can be adapted to other menus (like `bemenu`/`dmenu`),
but I'm not interested in it because `rofi` has more features anyway.

I've made this because I want to create a declarative alternative to
[SXMO](https://sxmo.org), which uses dmenu/bemenu extensively.

As an example, let's assume we want to create a "system settings" menu,
and that we will have a `Settings -> Audio -> <device> -> Volume`
submenu. If the user selects the "close" button, the menu closes, if the
user selects the "back" button, they get back to the per-audio device
menu, and if the user enters a number, the device volume changes to that
number.

In that case, the script that manages that menu will have to know what
device was chosen in a previous menu - hence the stack. And a call stack
is necessary as well, so the back button can work.

## Dependencies

lib.sh depends on `jq`. Of course, `rofi` is required as well (I use
[`rofi-wayland`](https://github.com/lbonn/rofi)).

## Spec

[JSON5](https://json5.org) is used everywhere (a subset of ECMAScript, a
superset of JSON).

Each submenu is generated with a shell script. The script arguments are
whatever is currently on the stack.

When a user selects an option, a certain amount of values is popped from
the stack, certain values are pushed onto the stack, and a command may
be executed.

Besides the value stack, there's also the call stack. Of course it's
technically possible to merge them, but that isn't very convenient, so
they are separate.

The call stack is manipulated in a similar way to the value stack. The
top value in the call stack determines which script will be used for
the next menu.

Initial call stack contents is provided in the `INITIAL_SCRIPT` env var,
initial value stack contents are provided in the `INITIAL_STACK` env
var. They can either be JSON5 arrays, or simple strings (in which case
that string will be the only value on the stack - by the way, the string
can't start with `[` and end with `]`, or it will be parsed as a JSON5
array).

### Menu options

Global menu options must be printed by the script before all menu
entries, and are mandatory. All options are optional, so an empty object
is a valid instance of menu options.

- `prompt: <string>` - user prompt
- `message: <string>` - user-facing message (notice, etc)
- `markup: "pango"` - to enable pango markup
- `selection: <number>` - to select item by index (0-based)
- `selection: "keep"` - to keep whatever was selected previously
- `fallback: {...}` - this allows the user to input custom text. The
  format is similar to per-row options, but doesn't allow any cosmetic
  fields (i.e. only stack operations/commands are accepted).

### Menu Entry

Each entry is defined as follows:

- Cosmetic options:
  - `text: <string>` - user-facing text
  - `icon: <path>` - a path to this option's graphical icon
  - `meta: <string>` - search terms for this entry (hidden from the
    user)
  - `selectable: false` - marks the entry as unselectable
  - `urgent: true` - marks the entry as urgent
  - `active: true` - marks the entry as active
- Operations to be executed on entry selection:
  - `pop: null` - remove all values from the stack
  - `pop: <number>` - remove a certain amount of values from the top of
    the stack
  - `push: <string/list/null>` - add values on top of the stack when
    this option is selected. `null` means add user input.
    - If only one item is to be pushed, you can simply use that item
      instead of enclosing it in a list (i.e. `push: "a"` instead of
      `push: ["a"]`)
    - If a list is one of the items of the list, the values will be
      concatenated. For example, `[["a", null], "b"]` will push the
      concatenation of `a` and user input, and then push `b`.
  - `jump: <string/list/null>` - push a new script to the call stack,
    exactly the same format as `push`
  - `return: ...` - pop scripts from the call stack, exactly the same
    format as `pop`.
  - `goto: <string>` - shorthand for `return: 1; jump: <string>` (jumps
    to another script without remembering this script)
  - `exec: <string/list/null>` - bash command to execute. The format
    is the same as `push` and `jump`, each array element is an argument,
    starting from argv0.
    - If you only pass a single string not enclosed in an array, it will
      be interpreted as the entire command line (rather than the argv0).
  - `fork: true` - don't wait for the bash command's completion and
    run it in the background
