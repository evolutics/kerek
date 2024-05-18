{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    travel-kit.url = "github:evolutics/travel-kit";
  };

  outputs = {
    flake-utils,
    nixpkgs,
    travel-kit,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      devShell = pkgs.mkShellNoCC {
        buildInputs =
          (with pkgs; [
            curl
          ])
          ++ [travel-kit.packages.${system}.default];

        # TODO: Remove once looper is integrated.
        shellHook = ''
          export PATH="$PWD/target/debug:$PATH"
        '';
      };
    });
}
