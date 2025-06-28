{
  description = "socket-notify reads from a socket and triggers notifications";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import "${nixpkgs}" {
          inherit system;
        });

        buildDeps = with pkgs; [
          pkg-config
          dbus
        ];
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            cargo-expand
            rustc
            rust-analyzer
            clippy

          ] ++ buildDeps;
        };
      });
}
