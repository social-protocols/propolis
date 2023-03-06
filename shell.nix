let
  # Pinned nixpkgs, deterministic.
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/507feca6063e9360449e9a45ec6959a5199f82c3.tar.gz")) {};

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
