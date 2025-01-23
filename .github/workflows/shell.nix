{
  pkgs ? import <nixpkgs> { },
}:

let
  baseDir = toString ./.;
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
  ];
}
