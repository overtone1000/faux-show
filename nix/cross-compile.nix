# for cross compiling backend
let
  commons = import ./commons.nix;
in

commons.cross_pkgs.rustPlatform.buildRustPackage {
  name = "shmashmexa";
  pname = "shmashmexa-backend";
  version = "0.3.0";

  srcs = [
    ./..
    ./../../trm-rust-libs #Uses two modules from this
  ];

  unpackPhase = ''
    #Necessary if using custom unpack hook
    runHook preUnpack

    echo Unpacking
    
    mkdir sources
    mkdir -p sources/repos/shmashmexa
    mkdir -p sources/repos/trm-rust-libs
    cp -r ${./..}/** sources/repos/shmashmexa
    cp -r ${./../../trm-rust-libs}/** sources/repos/trm-rust-libs

    #ls -lash .
    #ls -lash ./sources
    #ls -lash ./sources/repos/shmashmexa
    #ls -lash ..

    #Make sure any pre-existing build artifacts are removed
    chmod -R +w ./sources/repos/shmashmexa/target
    rm -rd ./sources/repos/shmashmexa/target

    #Necessary if using custom unpack hook
    runHook postUnpack
  '';
  
  sourceRoot = "/build/sources/repos/shmashmexa";

  #Generate with
  #cargo update && diff -u /dev/null ./Cargo.toml > ./nix/add-Cargo.lock.patch
  #Doesn't work!
  #cargoPatches = [ ./add-Cargo.lock.patch ];


  #Not needed, will build all crates in the workspace.
  #cargoBuildFlags=["-p shmashmexa-backend"];
  #buildAndTestSubdir = "backend";

  # If using a workspace lockfile, this has to be the main one.
  cargoLock.lockFile = ../Cargo.lock; 
  #cargoHash = "sha256-v8kU+mLUV+qLGwhDPCqi/uDLWRWuhwe0UPg0r/vMaQI=";

  #cargoLock.outputHashes = {
  #   "hyper-services" = "sha256-hash...";
  #};
}

# try with nix-build ./nix/cross-compile.nix in repo root