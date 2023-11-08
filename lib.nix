{ lib, ... }:

let self = {
  types = rec {
    stringListNull = lib.types.oneOf [ bashStr lib.types.null (lib.types.listOf stringListNull) bashValue ];
    bashValue = lib.types.submodule {
      options._bash = lib.mkOption {
        description = "Bash code to execute that prints the desired JSON output";
        type = lib.types.str;
      };
    };
    bashStr = lib.types.either lib.types.str bashValue;
    bashBool = lib.types.either lib.types.bool bashValue;
    bashInt = lib.types.either lib.types.int bashValue;
    row' = cosmetic: lib.types.submodule {
      options = {
        enable = lib.mkOption {
          default = true;
          type = lib.types.bool;
        };
        _bash = lib.mkOption {
          description = "Bash code to execute that prints the row JSON. If set, all other options (except enable) are ignored.";
          type = lib.types.nullOr lib.types.lines;
          default = null;
        };
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
          default = {};
          type = submenu;
        };
      } // lib.optionalAttrs cosmetic {
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
    submenu = lib.types.submodule {
      options = {
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
        rows = lib.mkOption {
          default = [ ];
          type = lib.types.attrsOf row;
        };
      };
    };
  };

  escapeBashVal = x: if x?_bash then ''"$(${x._bash})"'' else lib.escapeShellArg (builtins.toJSON x);
  filterJson = type: val: lib.filterAttrs (k: v: v != (type.getSubOptions { }).${k}.default) val;

  compileRow = row: if row._bash != "" then row._bash else
    builtins.concatStringsSep " " ([
      "row ${self.escapeBashVal row.text}"
    ] ++ lib.mapAttrsToList
      (k: v: "${k} ${self.escapeBashVal v}")
      (self.filterJson self.types.row (builtins.removeAttrs row [ "enable" "_bash" ])));
  compileOptions = options: if options._bash != "" then options._bash else
    builtins.concatStringsSep " " ([
      "options"
    ] ++ lib.mapAttrsToList
      (k: v: "${k} ${self.escapeBashVal v}")
      (self.filterJson self.types.submenu (builtins.removeAttrs options [ "enable" "execPre" "_bash" "rows" ])));
  compileMenu = menu: ''
    ${self.compileOptions menu}
    ${builtins.concatStringsSep "\n" (map self.compileRow menu.rows)}
  '';
}; in self
