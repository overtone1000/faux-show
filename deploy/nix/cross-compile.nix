# for cross compiling backend
let
  commons = import ./commons.nix;

  #paths
  source=./../..;
  trm_rust_libs=./../../../trm-rust-libs; #Uses two modules from this

  #strings
  sources_dir_faux_show="repos/faux_show";
  sources_dir_trm_rust_libs="repos/trm-rust-libs";
in

commons.cross_pkgs.rustPlatform.buildRustPackage {
  name = "faux-show";
  pname = "faux-show-backend"; #Name of the package of interest
  version = "0.3.0"; #Package version

  srcs = [
    source
    trm_rust_libs
  ];

  unpackPhase = ''
    #Necessary if using custom unpack hook
    runHook preUnpack

    echo Unpacking
    
    mkdir sources
    mkdir -p sources/${sources_dir_faux_show}
    mkdir -p sources/${sources_dir_trm_rust_libs}
    cp -r ${source}/** sources/${sources_dir_faux_show}
    cp -r ${trm_rust_libs}/** sources/${sources_dir_trm_rust_libs}

    #ls -lash .
    #ls -lash ./sources
    #ls -lash ./sources/${sources_dir_faux_show}
    #ls -lash ./sources/${sources_dir_trm_rust_libs}

    #Make sure any pre-existing build artifacts are removed
    chmod -R +w ./sources/${sources_dir_faux_show}/target
    rm -rd ./sources/${sources_dir_faux_show}/target

    #Necessary if using custom unpack hook
    runHook postUnpack
  '';
  
  sourceRoot = "/build/sources/${sources_dir_faux_show}";

  # If using a workspace, must reference the lock file for the whole workspace.
  cargoLock.lockFile = source+"/Cargo.lock"; 
}

# try with nix-build ./deploy/nix/cross-compile.nix in repo root