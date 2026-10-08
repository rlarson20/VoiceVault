{
  description = "Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, fenix, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f {
        pkgs = nixpkgs.legacyPackages.${system};
        rust = fenix.packages.${system}.stable.withComponents [
          "cargo" "clippy" "rust-src" "rustc" "rustfmt" "rust-analyzer"
        ];
      });
    in {
      devShells = forAll ({ pkgs, rust }: {
        default = pkgs.mkShell {
          packages = [ rust pkgs.sccache ];
          RUSTC_WRAPPER = "sccache";
        };
      });
    };
}
