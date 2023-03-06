let
  # Pinned nixpkgs, deterministic.
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/3a6307ccdab8fa113cf28c84848f6c46b546a264.tar.gz")) {};

  # Rolling updates, not deterministic.
  # pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};
in pkgs.mkShell {
  buildInputs = with pkgs; [ 
    cargo
    cargo-watch
    direnv
    just
    rustfmt
    rustc
    rust-analyzer # for language server
    sqlx-cli
    sqlite-interactive

    # http benchmarking
    wrk
    apacheHttpd # apache bench

    # deployemnt
    flyctl
    docker
  ];

  shellHook = ''
eval "$(direnv hook $SHELL)"
echo DATABASE_URL: $DATABASE_URL

if [ ! $DATABASE_URL = "" ] && [ ! -e data ]; then
   just create-db
fi
'';
}
