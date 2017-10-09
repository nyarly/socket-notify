{ pkgs ? import <nixpkgs> {} }:
with pkgs;
stdenv.mkDerivation {
  name = "socket-notify";
  buildInputs = [ dbus ];
  nativeBuildInputs = [ pkgconfig ];
}
