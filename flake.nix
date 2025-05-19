{
  description = "tiny_ledger dev flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
  } @ inputs: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = function: nixpkgs.lib.genAttrs supportedSystems (system: function system);

    pkgsFor = system:
      import nixpkgs {
        inherit system;
        overlays = [fenix.overlays.default];
      };
  in {
    devShells = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell {
        name = "payments-engine";

        nativeBuildInputs = [
          pkgs.fenix.stable.toolchain
          pkgs.pkg-config
          pkgs.rust-analyzer
          pkgs.bacon
        ];

        buildInputs = [];

        shellHook = ''
          export RUST_BACKTRACE=1
          export RUST_SRC_PATH="${pkgs.fenix.stable.toolchain}/lib/rustlib/src/rust/library"
        '';
      };
    });
  };
}
