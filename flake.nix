{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: 
    let
      forEachSystem = fn: nixpkgs.lib.genAttrs [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ]
        (system: fn (import nixpkgs {
          inherit system;
        }));
    in
    {
      packages = forEachSystem (pkgs: rec {
        rofi-menu-stack = pkgs.rustPlatform.buildRustPackage {
          pname = "rofi-menu-stack";
          version = "0.1.0";
          src = pkgs.nix-gitignore.gitignoreSource ["*.nix"] (nixpkgs.lib.cleanSource ./.);
          cargoLock.lockFile = ./Cargo.lock;
          meta.license = nixpkgs.lib.licenses.mit;
        };
        test = let
          inherit (nixpkgs) lib;
          inherit (lib.modules) evalModules;
          getVal = type: expr:
            (evalModules {
              modules = [
                {
                  options.val = lib.mkOption { inherit type; };
                  config.val = expr;
                }
              ];
            }).config.val;
        in pkgs.writeShellScript "test-script" (self.lib.${pkgs.system}.compileMenu (getVal self.lib.${pkgs.system}.types.submenu {
          prompt = "Fibonacci >";
          message._bashStr = ''echo -n "Value #$1: $3"'';
          rows = {
            a = {
              text = "Next";
              pop = 3;
              push = [
                { _bashStr = ''echo -n "$2"''; }
                { _bashStr = ''python3 -c "print($2+$3)"''; }
                { _bashStr = ''echo -n "$(("$1" + 1))"''; }
              ];
            };
            b = {
              text = "Switch to incrementor";
              pop = 2;
              return = 1;
              jump = "sample/incrementor.sh";
            };
            c = {
              text = "Close";
              return = null;
            };
          };
        }));
        default = rofi-menu-stack;
      });
      lib = forEachSystem (_: import ./lib.nix { inherit (nixpkgs) lib; });
    };
}
