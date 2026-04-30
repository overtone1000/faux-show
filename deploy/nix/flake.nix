{
  description = "Cross compiling a rust program using rust-overlay";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-25.11";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      localSystem:
      let
        # Replace with the system you want to build for
        crossSystem = "aarch64-linux";

        main_source_dir=./../..;
        #trm_rust_libs_source_dir=./../../../trm-rust-libs; #Uses two modules from this

        pkgs = import nixpkgs {
          inherit crossSystem localSystem;
          overlays = [ (import rust-overlay) ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);

        commonArgs={
          src = craneLib.cleanCargoSource main_source_dir;
          strictDeps = true;
          pname = "faux-show-backend"; #Name of the package of interest
          version = "0.3.0"; #Package version
        };

        deps_expression =
        {
          lib,
          stdenv
        }:
        craneLib.buildDepsOnly commonArgs;

        deps = pkgs.callPackage deps_expression { };

        main_expression =
        {
          lib,
          stdenv
        }:
        craneLib.buildPackage (commonArgs // {
          cargoArtifacts=deps;
        });

        main = pkgs.callPackage main_expression { };
      in
      {
        checks = {
          inherit main;
        };

        packages.default = main;

        apps.default = flake-utils.lib.mkApp {
          drv = pkgs.writeScriptBin "faux-show-backend" ''
            ${pkgs.pkgsBuildBuild.qemu}/bin/qemu-aarch64 ${main}/bin/cross-rust-overlay
          '';
        };
      }
    );
}

#Run from repo root
#nix build --extra-experimental-features 'nix-command flakes' ./deploy/nix/#