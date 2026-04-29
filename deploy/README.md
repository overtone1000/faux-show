## Nix Deployment

This build approach does the following:
1. Cross compile and package the backend for Nix using Crane via `flake.nix`.
2. Copy the resulting package to the device with `nix copy`.
3. Build the frontend.
4. Copy the frontend to the device's `/var/lib/www/` directory for use by the backend.

Running the backend as a systemd service is in the _config_ of the device. This is also where pertinent environment variables are set.


### Initialization

The build machine must be configured as a nix cache. First, generate a key pair for signing packages:
```
nix-store --generate-binary-cache-key builder ./deploy/nix/nix-store-binary-cache-key-{secret,public}
```

On the remote host, the configurations must be modified to include the public key as a trusted cache.
```
nix.settings.trusted-public-keys = [
  <content of nix-store-binary-cache-key-public>
];
```

### Deploying Backend

After cross compiling, the build result can be signed and copied
```
nix-build ./deploy/nix/cross-compile-buildRustPackage.nix
NIX_STORE_DIR=$(readlink -f result)
nix --extra-experimental-features nix-command store sign --recursive --key-file ./deploy/nix/nix-store-binary-cache-key-secret $NIX_STORE_DIR
nix --extra-experimental-features nix-command copy --to ssh://$SERVER_IP $NIX_STORE_DIR
ssh $SERVER_IP "sed -i \"s|nix_store_dir=\\\".*\\\";|nix_store_dir=\\\"$NIX_STORE_DIR\\\";|\" /etc/nixos/trm_nixos/devices/raspberry_pi_kiosk/imports/faux-show-backend.nix"
```

The package can be added to the NixOS config by adding the store path directly in system packages.
```
let
  nix_store_dir="NIX_STORE_DIR from above"
in
{ pkgs, ... }:{
  environment.systemPackages = [
    nix_store_dir
  ];
}
```

### Deploying Frontent

The frontend is simply stored in `/var/www/html/faux_show`