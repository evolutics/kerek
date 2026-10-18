# Update-worthy: `flake.lock` file.
{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    travel-kit = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:evolutics/travel-kit";
    };
  };

  outputs = {
    flake-utils,
    nixpkgs,
    travel-kit,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
    in {
      devShell = pkgs.mkShellNoCC {
        buildInputs =
          (with pkgs; [
            curl
            vagrant
          ])
          ++ [travel-kit.packages.${system}.default];

        shellHook = ''
          export DOCKER_HOST="unix://$XDG_RUNTIME_DIR/podman/podman.sock";
        '';
      };
    });
}
