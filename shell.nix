let
  # Pinned nixpkgs, deterministic. Last updated: 2/12/21.
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/7b08f3fd0dad6e07a1611401fb8772a2469e64ac.tar.gz")) {};

  # Rolling updates, not deterministic.
  # pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};
in pkgs.mkShell {
  buildInputs = [ 
    pkgs.cargo
    pkgs.rustfmt
    pkgs.rustc
    pkgs.rust-analyzer # for language server
    pkgs.sqlx-cli
    pkgs.sqlite-interactive
  ];
}
