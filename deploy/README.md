## Nix Deployment

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
nix-build ./deploy/nix/cross-compile.nix
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