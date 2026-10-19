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
    flake-utils.lib.eachSystem [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Pin a specific stable Rust version; bump this deliberately.
        rustToolchain = pkgs.rust-bin.stable."1.85.0".default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
            "llvm-tools-preview"
          ];
        };

        # macOS-only build inputs. The Apple frameworks (AppKit,
        # QuartzCore, Metal, Security, SystemConfiguration, CoreFoundation)
        # are now provided automatically by the Darwin stdenv via
        # $SDKROOT, so they must NOT be listed explicitly here.
        # See: https://nixos.org/manual/nixpkgs/stable/#sec-darwin-legacy-frameworks
        darwinLibs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
          pkgs.libiconv
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          name = "minsky-rs";

          buildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.openssl
            pkgs.cargo-watch
            pkgs.cargo-deny
            pkgs.just # optional task runner
          ]
          ++ darwinLibs;

          shellHook = ''
            echo "=== Minsky-RS dev shell ==="
            echo "Rust:  $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo ""
            echo "Phase 1 checklist:"
            echo "  rustc --version"
            echo "  cargo --version"
            echo "  cargo new minsky-rs --bin && cd minsky-rs && cargo run"
            echo ""
            echo "Note: rustup is NOT used here; the toolchain is Nix-pinned."
            echo "Tip: use 'nix develop -c cargo run' for one-off commands."
          '';
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
