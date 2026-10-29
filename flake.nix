{
  description = "Leptos components styled with Tailwind CSS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable."1.95.0".default.override {
          targets = [ "wasm32-unknown-unknown" ];
          extensions = [
            "rust-analyzer"
            "rust-src"
          ];
        };

        wasmBindgenCli = pkgs.buildWasmBindgenCli rec {
          src = pkgs.fetchCrate {
            pname = "wasm-bindgen-cli";
            version = "0.2.128";
            hash = "sha256-a7lcXJnnZkYReja+iUO7NqqrWyv3toxnUgQb8s4IS5s=";
          };

          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit src;
            inherit (src) pname version;
            hash = "sha256-R1Tas33Ursy8kqsxguAkG0ZhNed2n5uFTAhw1l2qlLY=";
          };
        };

        galleryWatch = pkgs.writeShellApplication {
          name = "gallery-watch";
          text = ''
            exec nix develop --command cargo leptos watch -p component-gallery
          '';
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkgs.cargo-leptos
            pkgs.tailwindcss_4
            pkgs.leptosfmt
            pkgs.just
            wasmBindgenCli
            pkgs.binaryen
          ];
        };

        apps.gallery-watch = {
          type = "app";
          program = "${galleryWatch}/bin/gallery-watch";
        };
      }
    );
}
