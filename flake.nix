# inspired by https://ayats.org/blog/nix-rustup/

{
  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  in {
    devShells.${system}.default = with pkgs; pkgs.mkShell {
      packages = [
        toolchain

        # We want the unwrapped version, "rust-analyzer" (wrapped) comes with nixpkgs' toolchain
        rust-analyzer-unwrapped

        cargo
        cargo-watch

        git
        just
        jq
        sqlx-cli
        sqlite-interactive
        nodePackages.browser-sync # dev hot reloading
        process-compose # orchestrate non-containerized processes
        entr # file watching

        # required to build openssl-sys, which openai uses
        pkg-config
        openssl
        cmake
        
        # http benchmarking
        wrk
        apacheHttpd # apache bench

        # deployemnt
        flyctl
        docker

        # building sqlite-vector
        sqlite
      ];
    };
  };
}
