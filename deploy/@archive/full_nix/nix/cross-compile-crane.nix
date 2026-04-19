# for cross compiling backend
let
  commons = import ./commons.nix;

  craneSource = builtins.fetchTarball {
    url = "https://github.com/ipetkov/crane/archive/refs/tags/v0.23.3.tar.gz";
    sha256 = "sha256:0kdjzp7ljik9hpw3x67phx2kp1v30j7xy5h98lskmnb4hcdbsj1b";
  };

  craneLib = import craneSource {
    inherit (commons.cross_pkgs);
  };

  #paths
  source=./../..;
  trm_rust_libs=./../../../trm-rust-libs; #Uses two modules from this

  #strings
  sources_dir_faux_show="repos/faux_show";
  sources_dir_trm_rust_libs="repos/trm-rust-libs";

  trm_rust_lib_deps = 
  {
    src = craneLib.cleanCargoSource trm_rust_libs;
    strictDeps = true;
    #These are needed by crane
    pname = "trm-rust-lib"; #Name of the package of interest
    version = "0.3.0"; #Package version
  };

  lib_build_artifacts = craneLib.buildDepsOnly commonArgs;

  commonArgs = {
    src = craneLib.cleanCargoSource source;
    strictDeps = true;
    #These are needed by crane
    pname = "faux-show-backend"; #Name of the package of interest
    version = "0.3.0"; #Package version

    buildInputs = [
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
  };

  # Build *just* the cargo dependencies, so we can reuse
  # all of that work (e.g. via cachix) when running in CI
  main_build_artifacts = craneLib.buildDepsOnly (commonArgs // {
    cargoArtifacts=lib_build_artifacts;
  });
  
  build = craneLib.buildPackage ({
    cargoArtifacts=main_build_artifacts;
  }//commonArgs);

in

build

# try with nix-build ./deploy/nix/cross-compile.nix in repo root
# Caching doesn't seem to work for some reason. Always get "cargoArtifacts not set, will not reuse any cargo artifacts" and then it completely rebuilds everything.
# Probably because of the cross compile approach in commons.nix