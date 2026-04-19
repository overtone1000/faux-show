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
nix --extra-experimental-features nix-command store sign --recursive --key-file ./deploy/nix/nix-store-binary-cache-key-secret $(readlink -f result)
nix --extra-experimental-features nix-command copy --to ssh://$SERVER_IP $(readlink -f result)
```

### Deploying Frontent

The frontend is simply stored in `/var/www/html/faux_show`