# for cross compiling backend
let
  commons = import ./commons.nix;
in

commons.cross_pkgs.mkShell commons.build_configuration // {

}

# try with nix-shell ./deploy/nix/cross-shell.nix --run "cargo build --release --target aarch64-unknown-linux-gnu"