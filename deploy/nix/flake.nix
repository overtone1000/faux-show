{
  description = "Cross compiling a rust program using rust-overlay";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-25.11";

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
        trm_rust_libs_source_dir=./../../../trm-rust-libs; #Uses two modules from this

        pkgs = import nixpkgs {
          inherit crossSystem localSystem;
          overlays = [ (import rust-overlay) ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);

        trm_libs_expression =
        {
          lib,
          stdenv
        }:
        craneLib.buildDepsOnly {
          src = craneLib.cleanCargoSource trm_rust_libs_source_dir;
          strictDeps = true;
        };

        trm_libs = pkgs.callPackage trm_libs_expression { };

        commonArgs={
          src = craneLib.cleanCargoSource main_source_dir;
          strictDeps = true;
        };

        main_deps_expression =
        {
          lib,
          stdenv
        }:
        craneLib.buildDepsOnly (commonArgs // {
          cargoArtifacts=trm_libs_expression;
        });

        main_deps = pkgs.callPackage main_deps_expression { };

        main_expression =
        {
          lib,
          stdenv
        }:
        craneLib.buildPackage (commonArgs // {
          cargoArtifacts=main_deps;
        });

        main = pkgs.callPackage main_deps_expression { };
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

#nix build --extra-experimental-features 'nix-command flakes' ./deploy/nix/#cross-rust-overlay