{ pkgs ? import <nixpkgs> {} }:

let
  cargoLockFile = pkgs.fetchurl {
    url = "https://raw.githubusercontent.com/GiviMAD/rustpotter-cli/refs/heads/main/Cargo.lock";
    hash = "sha256-dNAhgZLgl30FNySPZkUGC369OCTHjetnoK2xZDMbRxc=";
  };
in 
pkgs.rustPlatform.buildRustPackage rec {
  pname = "rustpotter-cli";
  version = "3.0.2";

  # Fetch the source code from GitHub
  src = pkgs.fetchFromGitHub {
    owner = "GiviMAD";
    repo = "rustpotter-cli";
    rev = "a9e64a2"; # Can be a git tag, branch name, or commit hash
    hash = "sha256-nIk2xiAB4X5HMhgZhEEg8xNKMKWgFlEqHSCnbNCTLrg="; 
  };

  # Hash of the dependencies extracted from Cargo.lock
  # cargoHash = ""; #Doesnt' work, looks like one dependency doesn't publish Cargo.lock
  # cargoVendorDir = "./vendor";
  
  cargoLock = {
    lockFile = cargoLockFile;

    # If your Cargo.lock contains git dependencies, Nix requires their explicit output hashes.
    # Leave this empty first, and Nix will tell you what hashes are missing in the build error.
    outputHashes = {
      # "crate-name-version" = "sha256-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=";
    };
  };

  nativeBuildInputs = [ 
    pkgs.pkg-config
  ];

  buildInputs = [ 
    pkgs.alsa-lib
  ];

  meta = {
    description = "Rustpotter CLI";
    homepage = "https://github.com/GiviMAD/rustpotter-cli";
  };
}