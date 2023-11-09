{ lib, ... }:

let self = {
  types = rec {
    stringListNull = lib.types.oneOf [ (lib.types.nullOr bashStr) (lib.types.listOf stringListNull) bashValue ];
    bashValue = lib.types.submodule {
      options._bash = lib.mkOption {
        description = "Bash code to execute that prints the desired JSON output";
        type = with lib.types; nullOr str;
        default = null;
      };
      options._bashStr = lib.mkOption {
        description = "Bash code to execute that prints the desired string to get escaped as JSON";
        type = with lib.types; nullOr str;
        default = null;
      };
    };
    bashStr = lib.types.either lib.types.str bashValue;
    bashBool = lib.types.either lib.types.bool bashValue;
    bashInt = lib.types.either lib.types.int bashValue;
    row' = full: lib.types.submodule {
      options = lib.optionalAttrs full {
        enable = lib.mkOption {
          default = true;
          type = lib.types.bool;
        };
        _bash = lib.mkOption {
          description = "Bash code to execute that prints the row JSON. If set, all other options (except enable) are ignored.";
          type = lib.types.nullOr lib.types.lines;
          default = null;
        };
      } // {
        push = lib.mkOption {
          default = [ ];
          type = stringListNull;
        };
        pop = lib.mkOption {
          default = 0;
          type = lib.types.nullOr bashInt;
        };
        jump = lib.mkOption {
          default = [ ];
          type = stringListNull;
        };
        return = lib.mkOption {
          default = 0;
          type = lib.types.nullOr bashInt;
        };
        exec = lib.mkOption {
          default = [ ];
          type = stringListNull;
        };
        fork = lib.mkOption {
          default = false;
          type = bashBool;
        };
        menu = lib.mkOption {
          default = null;
          type = lib.types.nullOr (submenu' false);
        };
      } // lib.optionalAttrs full {
        text = lib.mkOption {
          type = bashStr;
        };
        icon = lib.mkOption {
          default = null;
          type = lib.types.nullOr bashStr;
        };
        meta = lib.mkOption {
          default = null;
          type = lib.types.nullOr bashStr;
        };
        selectable = lib.mkOption {
          default = true;
          type = bashBool;
        };
        urgent = lib.mkOption {
          default = false;
          type = bashBool;
        };
        active = lib.mkOption {
          default = false;
          type = bashBool;
        };
      };
    };
    row = row' true;
    fallbackRow = row' false;
    submenu' = full: lib.types.submodule {
      options = lib.optionalAttrs full {
        enable = lib.mkOption {
          default = true;
          type = lib.types.bool;
        };
        execPre = lib.mkOption {
          default = "";
          type = lib.types.lines;
        };
        _bash = lib.mkOption {
          description = "Bash code to execute that prints the menu options (config). If set, all other menu options (except enable, rows, execPre) are ignored.";
          type = lib.types.nullOr lib.types.lines;
          default = null;
        };
        rows = lib.mkOption {
          default = [ ];
          type = lib.types.attrsOf row;
        };
      } // {
        prompt = lib.mkOption {
          default = null;
          type = lib.types.nullOr bashStr;
        };
        message = lib.mkOption {
          default = null;
          type = lib.types.nullOr bashStr;
        };
        markup = lib.mkOption {
          default = null;
          type = lib.types.nullOr bashStr;
        };
        autoselect = lib.mkOption {
          default = false;
          type = bashBool;
        };
        fallback = lib.mkOption {
          default = null;
          type = lib.types.nullOr fallbackRow;
        };
      };
    };
    submenu = submenu' true;
  };

  escapeBashVal = x:
    if (x._bash or null) != null then ''"$(${x._bash})"''
    else if (x._bashStr or null) != null then ''"$(val "$(${x._bashStr})")"''
    else if builtins.isList x then "[" + builtins.concatStringsSep "," (map self.escapeBashVal x) + "]"
    else lib.escapeShellArg (builtins.toJSON x);
  filterJson = type: val: lib.filterAttrs (k: v: (v.enable or true) && v != (type.getSubOptions { }).${k}.default) val;

  compileRow = row: if row._bash != null then row._bash else
    builtins.concatStringsSep " " ([
      "row ${self.escapeBashVal row.text}"
    ] ++ lib.mapAttrsToList
      (k: v: "${k} ${self.escapeBashVal v}")
      (self.filterJson self.types.row (builtins.removeAttrs row [ "enable" "_bash" "text" ])));
  compileOptions = options: if options._bash != null then options._bash else
    builtins.concatStringsSep " " ([
      "options"
    ] ++ lib.mapAttrsToList
      (k: v: "${k} ${self.escapeBashVal v}")
      (self.filterJson self.types.submenu (builtins.removeAttrs options [ "enable" "execPre" "_bash" "rows" ])));
  compileMenu = menu: ''
    . ${./lib.sh}
    ${self.compileOptions menu}
    ${builtins.concatStringsSep "\n" (map self.compileRow (builtins.attrValues menu.rows))}
  '';
}; in self
