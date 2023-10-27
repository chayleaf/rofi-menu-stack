{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: {
    packages = nixpkgs.lib.genAttrs [ "x86_64-linux" ] (system: let
      pkgs = import nixpkgs { inherit system; };
    in rec {
      rofi-menu-stack = pkgs.rustPlatform.buildRustPackage {
        pname = "rofi-menu-stack";
        version = "0.1.0";
        src = pkgs.nix-gitignore.gitignoreSource ["*.nix"] (nixpkgs.lib.cleanSource ./.);
        cargoLock.lockFile = ./Cargo.lock;
        meta.license = nixpkgs.lib.licenses.mit;
      };
      default = rofi-menu-stack;
    });
    lib.x86_64-linux = import ./lib.nix { inherit (nixpkgs) lib; };
  };
}
