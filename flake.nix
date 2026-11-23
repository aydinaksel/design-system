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
            version = "0.2.130";
            hash = "sha256-9Og1O/6/VC3sc5hBjoIWBErmLW2UHGMsTQO+6+m/dng=";
          };

          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit src;
            inherit (src) pname version;
            hash = "sha256-j81sUf1m5yXU4FJfGj7GV+hdqVCV8mRmNb0QIsMCjqw=";
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
            pkgs.curl
            (pkgs.python3.withPackages (packages: [ packages.playwright ]))
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.chromium ];
          STRESS_CHROMIUM = pkgs.lib.optionalString pkgs.stdenv.isLinux "${pkgs.chromium}/bin/chromium";
        };

        apps.gallery-watch = {
          type = "app";
          program = "${galleryWatch}/bin/gallery-watch";
        };
      }
    );
}
