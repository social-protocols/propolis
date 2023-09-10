# inspired by https://ayats.org/blog/nix-rustup/

{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    # derive the correct compiler versions from rust-toolchain.toml
    rust-overlay.url = "github:oxalica/rust-overlay";

    # for `flake-utils.lib.eachSystem`
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
          config.allowUnfree = false;
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPkgs = [ rustToolchain ] ++ (with pkgs; [ openssl pkg-config cmake sqlite cargo-chef ]);
        rustDevPkgs = rustPkgs ++ (with pkgs; [ cargo-watch rust-analyzer ]);
      in
      {
        devShells = {
          default = with pkgs; pkgs.mkShellNoCC {
            buildInputs = rustDevPkgs ++ [
              git
              just
              jq
              sqlx-cli
              sqlite-interactive
              nodePackages.browser-sync # dev hot reloading
              process-compose # orchestrate non-containerized processes
              entr # file watching

              # http benchmarking
              wrk
              apacheHttpd # apache bench

              # deployemnt
              flyctl
              docker
              earthly
            ];
          };
          buildRust = with pkgs; pkgs.mkShellNoCC {
            buildInputs = rustPkgs;
          };
        };
      }
    );
}


