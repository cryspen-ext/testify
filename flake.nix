{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);

      craneLib = (crane.mkLib pkgs).overrideToolchain rust;
      my-crate = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        strictDeps = true;

        # LIBCLANG_PATH="${pkgs.clang.cc.lib}/lib";
        buildInputs = [
          pkgs.clang
          pkgs.cmake
          pkgs.rust-bindgen
        ];
      };
    in {
      checks = {
        inherit my-crate;
      };

      packages.default = my-crate;

      apps.default = flake-utils.lib.mkApp {
        drv = my-crate;
      };

      devShells.default = pkgs.mkShell {
        packages = [
          pkgs.rust-analyzer
          pkgs.rustfmt
          pkgs.rustup
          pkgs.bacon
          pkgs.cargo-expand
          pkgs.cargo-tarpaulin
        ];
      };
      
      # devShells.default = craneLib.devShell {
      #   LIBCLANG_PATH="${pkgs.clang.cc.lib}/lib";
      #   # Z3_SYS_Z3_HEADER="${pkgs.z3.lib}";
      #   # Z3_SYS_Z3_HEADER="${pkgs.z3.dev}/include";
      #   checks = self.checks.${system};
      #   packages = [
      #     pkgs.rust-analyzer
      #     pkgs.rustfmt
      #     pkgs.rustup
      #     pkgs.rust-bindgen
      #     # pkgs.z3
      #     # pkgs.z3.dev
      #     # pkgs.z3.lib
          
      #   ];
      # };
    });
}
