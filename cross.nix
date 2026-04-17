# for cross compiling backend
with import <nixpkgs> {
  #Defines architecture of target system
  crossSystem = {
    config = "aarch64-unknown-linux-gnu";
  };
};
mkShell {
    name = "cross-environment";

    #Architecture of target system is aarch64-multiplatform if it's a 64 bit raspberry pi
    nativeBuildInputs = [
      pkgs.pkgsCross.aarch64-multiplatform.rustc
      pkgs.pkgsCross.aarch64-multiplatform.cargo
      pkgs.pkgsCross.aarch64-multiplatform.pkg-config
      pkgs.pkgsCross.aarch64-multiplatform.gcc
    ];

    # Certain Rust tools won't work without this
    # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
    # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";

    #CC = "aarch64-linux-gnu-gcc";
    #CXX = "aarch64-linux-gnu-g++";
}

# try with nix-shell build.nix --run "cargo build --release --target aarch64-unknown-linux-gnu"