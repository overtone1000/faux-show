# for cross compiling backend
let
  commons = import ./commons.nix;
in

commons.cross_pkgs.rustPlatform.buildRustPackage {
  name = "faux_show";
  pname = "faux-show-backend"; #Name of the package of interest
  version = "0.3.0"; #Package version

  srcs = [
    ./..
    ./../../trm-rust-libs #Uses two modules from this
  ];

  unpackPhase = ''
    #Necessary if using custom unpack hook
    runHook preUnpack

    echo Unpacking
    
    mkdir sources
    mkdir -p sources/repos/faux_show
    mkdir -p sources/repos/trm-rust-libs
    cp -r ${./..}/** sources/repos/faux_show
    cp -r ${./../../trm-rust-libs}/** sources/repos/trm-rust-libs

    #ls -lash .
    #ls -lash ./sources
    #ls -lash ./sources/repos/faux_show
    #ls -lash ..

    #Make sure any pre-existing build artifacts are removed
    chmod -R +w ./sources/repos/faux_show/target
    rm -rd ./sources/repos/faux_show/target

    #Necessary if using custom unpack hook
    runHook postUnpack
  '';
  
  sourceRoot = "/build/sources/repos/faux_show";

  # If using a workspace, must reference the lock file for the whole workspace.
  cargoLock.lockFile = ../Cargo.lock; 
}

# try with nix-build ./nix/cross-compile.nix in repo root