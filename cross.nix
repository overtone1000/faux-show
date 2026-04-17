# for cross compiling backend
let
  nixpkgs_release = "25.11";

  #This is the build system architecture
  build = "x86_64-linux";

  #This is the targeted architecture
  host = "aarch64-linux";

  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/release-${nixpkgs_release}";
  
  pkgs = import nixpkgs {
    crossSystem = {
      config = host;
    };
  };
in

pkgs.mkShell {
    name = "cross-environment";

    #Architecture of target system is aarch64-multiplatform if it's a 64 bit raspberry pi
    nativeBuildInputs = [
      pkgs.rustc
      pkgs.cargo
      pkgs.pkg-config
      pkgs.gcc

      #With crossSystem, don't need to use pkgsCross? Trying.
      #pkgs.pkgsCross.aarch64-multiplatform.rustc
      #pkgs.pkgsCross.aarch64-multiplatform.cargo
      #pkgs.pkgsCross.aarch64-multiplatform.pkg-config
      #pkgs.pkgsCross.aarch64-multiplatform.gcc
    ];

    # Certain Rust tools won't work without this
    # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
    # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";

    #CC = "aarch64-linux-gnu-gcc";
    #CXX = "aarch64-linux-gnu-g++";

    #Can use this environment variable instead of changing target linkiner in .cargo/config.toml
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gcc";
}

# try with nix-shell build.nix --run "cargo build --release --target aarch64-unknown-linux-gnu"