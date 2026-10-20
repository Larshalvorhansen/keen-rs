{
  description = "Minsky-RS: Rust rebuild of Steve Keen's Minsky (macOS-first dev shell)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachSystem
      [
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ]
      (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.rust-bin.stable."1.85.0".default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
              "rustfmt"
              "llvm-tools-preview"
            ];
          };

          darwinLibs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
            pkgs.libiconv
          ];

        in
        {
          devShells.default = pkgs.mkShell {
            name = "minsky-rs";

            packages = [
              rustToolchain
              pkgs.pkg-config
              pkgs.openssl
              pkgs.cargo-watch
              pkgs.cargo-deny
              pkgs.just
            ]
            ++ darwinLibs;

            shellHook = ''
              echo "=== Minsky-RS dev shell ==="
              echo "Rust:  $(rustc --version)"
              echo "Cargo: $(cargo --version)"
              echo ""
              echo "Toolchain is Nix-pinned. rustup is not required."
            '';
          };

          devShells.upstream = pkgs.mkShell {
            name = "minsky-upstream";

            packages =
              with pkgs;
              [
                gnumake
                cmake
                pkg-config
                python3
                perl
                bison
                flex
                boost
                gsl
                openssl
                zlib
                libxml2
                cairo
                pango
                librsvg
                freetype
                fontconfig
                libpng
                soci
                sqlite
                postgresql
                git
              ]
              ++ darwinLibs;

            shellHook = ''
              echo "=== Minsky upstream C++ environment ==="
              echo "Python: $(python3 --version)"
              echo "C++:    $(c++ --version | head -n 1)"
            '';
          };

          formatter = pkgs.nixfmt-rfc-style;
        }
      );
}
